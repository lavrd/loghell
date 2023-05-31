use std::net::SocketAddr;
use std::str::from_utf8;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tracing::{debug, error, info, trace};

use crate::cluster::Message;
use crate::cluster::{self, NEW_LOG_MESSAGE_TYPE};
use crate::log_storage::LogStoragePointer;
use crate::shared::now_as_nanos_u64;

enum ProcessDataResult {
    Ok,
    Close,
}

pub(crate) struct Server {
    dashboard_content: String,
    connection_counter: Arc<AtomicU64>,
    log_storage: LogStoragePointer,
    csr: cluster::Transmitter,
}

impl Server {
    pub(crate) fn new(
        dashboard_content: String,
        connection_counter: Arc<AtomicU64>,
        log_storage: LogStoragePointer,
        csr: cluster::Transmitter,
    ) -> Self {
        Server {
            dashboard_content,
            connection_counter,
            log_storage,
            csr,
        }
    }

    pub(crate) async fn start(
        &self,
        socket_addr: &str,
        shutdown_rx: watch::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(socket_addr).await?;
        /*
           We get local address from listener instead of use from &self
            because we can pass local address with zero port which can be selected randomly.
        */
        let local_addr = listener.local_addr()?;
        info!("socket starts at: {}", local_addr);

        let mut shutdown_rx_ = shutdown_rx.clone();
        tokio::select! {
            res = self.accept(listener, shutdown_rx.clone()) => { res }
            _ = shutdown_rx_.changed() => {
                debug!("terminating accept new clients loop");
                Ok(())
            }
        }
    }

    async fn accept(
        &self,
        listener: TcpListener,
        shutdown_rx: watch::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (socket, socket_addr) = listener.accept().await?;
            info!("new client; ip: {}", socket_addr);
            self.connection_counter.fetch_add(1, Ordering::Relaxed);

            let mut connection = Connection::new(
                socket,
                socket_addr,
                shutdown_rx.clone(),
                self.dashboard_content.clone(),
                self.connection_counter.clone(),
                self.log_storage.clone(),
                self.csr.subscribe(),
            );
            tokio::spawn(async move {
                trace!("spawn thread for {} client", socket_addr);
                connection.process_socket().await;
                trace!("moving from spawn in accept loop for {} client", socket_addr);
            });
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        trace!("dropping Server")
    }
}

struct Connection {
    socket: TcpStream,
    socket_addr: SocketAddr,
    shutdown_rx: watch::Receiver<()>,
    dashboard_content: String,
    connection_counter: Arc<AtomicU64>,
    log_storage: LogStoragePointer,
    csr: cluster::Reader,
}

impl Connection {
    fn new(
        socket: TcpStream,
        socket_addr: SocketAddr,
        shutdown_rx: watch::Receiver<()>,
        dashboard_content: String,
        connection_counter: Arc<AtomicU64>,
        log_storage: LogStoragePointer,
        csr: cluster::Reader,
    ) -> Self {
        Connection {
            socket,
            socket_addr,
            shutdown_rx,
            dashboard_content,
            connection_counter,
            log_storage,
            csr,
        }
    }

    async fn process_socket(&mut self) {
        let mut shutdown_rx_ = self.shutdown_rx.clone();
        tokio::select! {
            res = self.read_data() => {
                match res {
                    Ok(()) => debug!("connection with {} client closed successfully", self.socket_addr),
                    Err(e) => error!("failed to read data from socket; client: {}: {}", self.socket_addr, e)
                }
            }
            _ = shutdown_rx_.changed() => {
                trace!("terminating read data loop; client: {}", self.socket_addr);
            }
        }

        match self.socket.shutdown().await {
            Ok(()) => trace!("successfully shutdown {} socket", self.socket_addr),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotConnected => debug!(
                    "cannot shutdown {} socket because it is already disconnected: {}",
                    self.socket_addr, e
                ),
                _ => error!("failed to shutdown socket with {} address: {}", self.socket_addr, e),
            },
        }

        trace!("we have moved from select in Connection.process_socket");
    }

    async fn read_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut buf = [0; 1024];
            // Read data from socket.
            let n = match self.socket.read(&mut buf).await {
                Ok(n) => {
                    trace!("read {} bytes from {} client", n, self.socket_addr);
                    n
                }
                Err(e) => {
                    error!("failed to read data from {} client", self.socket_addr);
                    return Err(e.to_string().into());
                }
            };
            match self.process_data(n, &buf).await {
                Ok(ProcessDataResult::Ok) => {
                    trace!("data from {} client successfully proceeded", self.socket_addr);
                    // Read next frame from socket.
                    continue;
                }
                Ok(ProcessDataResult::Close) => {
                    trace!("close connection with {} client", self.socket_addr);
                    // Go away from read loop to close connection with client.
                    return Ok(());
                }
                Err(e) => {
                    error!("failed to process data from {} client: {}", self.socket_addr, e);
                    return Err(e.to_string().into());
                }
            };
        }
    }

    async fn process_data(
        &mut self,
        n: usize,
        buf: &[u8],
    ) -> Result<ProcessDataResult, Box<dyn std::error::Error>> {
        match n {
            n if n == 0 => {
                trace!("connection with {} client closed", self.socket_addr);
                Ok(ProcessDataResult::Close)
            }
            n if n == 5 && buf[0..n] == [255, 244, 255, 253, 6] => {
                trace!(
                    "connection with {} client closed (ctrl+c by telnet client)",
                    self.socket_addr
                );
                Ok(ProcessDataResult::Close)
            }
            n => {
                if buf.starts_with(b"GET / HTTP/1.1") {
                    return self.handle_dashboard().await.map(|_| Ok(ProcessDataResult::Close))?;
                }
                if buf.starts_with(b"GET /events HTTP/1.1") {
                    return self.handle_sse().await.map(|_| Ok(ProcessDataResult::Close))?;
                }
                if buf.starts_with(b"GET /health HTTP/1.1") {
                    return self.handle_health().await.map(|_| Ok(ProcessDataResult::Close))?;
                }
                if buf.starts_with(b"cluster>") {
                    return self.handle_cluster().await.map(|_| Ok(ProcessDataResult::Close))?;
                }
                self.handle_log(buf, n).await.map(|_| Ok(ProcessDataResult::Ok))?
            }
        }
    }

    async fn handle_dashboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            self.dashboard_content.len(),
            self.dashboard_content
        );
        self.socket.write_all(response.as_bytes()).await?;
        self.socket.flush().await?;
        info!("sent dashboard for {} client", self.socket_addr);
        Ok(())
    }

    async fn handle_log(&self, buf: &[u8], n: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut truncated_buf = &buf[0..n];
        // Remove \n at the end of the data.
        if truncated_buf.ends_with(&[10]) {
            truncated_buf = &truncated_buf[0..n - 1]
        }
        info!(
            "new data received from {} client : {:?}",
            self.socket_addr,
            from_utf8(truncated_buf)?
        );
        self.log_storage.lock().await.store(truncated_buf).await
    }

    async fn handle_sse(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let response = "HTTP/1.1 200 OK
Connection: keep-alive
Content-Type: text/event-stream
Cache-Control: no-cache";
        self.socket.write_all(response.as_bytes()).await?;
        self.socket.flush().await?;

        self.socket.write_all(b"retry: 10000\n").await?;
        self.socket.flush().await?;
        self.socket.write_all(b"event: data\n").await?;
        self.socket.flush().await?;

        let mut shutdown_rx_ = self.shutdown_rx.clone();
        tokio::select! {
            res = self.send_sse_data() => { res },
            _ = shutdown_rx_.changed() => {
                trace!("terminating sse send data loop; client: {}", self.socket_addr);
                Ok(())
            }
        }
    }

    async fn send_sse_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut start_from = 0;
        loop {
            let mut logs = self.log_storage.lock().await.find("level:debug", start_from).await?;
            start_from = now_as_nanos_u64()?;
            // We need to send at leat one message at time to check that connection is still open.
            logs.push(b"check".to_vec());
            for log in &mut logs {
                log.push(10); // add new line
                let is_stop = write(&mut self.socket, &self.socket_addr, log).await?;
                if is_stop {
                    return Ok(());
                }
            }
            self.socket.flush().await?;
            trace!("sent sse (logs) data to {} client", self.socket_addr);
            // It means we sent only check command.
            if logs.len() == 1 {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }

    async fn handle_health(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let response = "HTTP/1.1 200 OK
Connection: close\n\n";
        self.socket.write_all(response.as_bytes()).await?;
        self.socket.flush().await?;
        Ok(())
    }

    async fn handle_cluster(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // As we want to add meta data to our message when transmit it to other cluster members
        // we need to add two more bytes with it. It is message type and new line.
        const DATA_OVERHEAD_LENGTH: usize = 2;
        loop {
            let msg = self.csr.recv().await?;
            let data = match msg {
                Message::NewLog(new_log) => {
                    let mut data: Vec<u8> =
                        Vec::with_capacity(new_log.len() + DATA_OVERHEAD_LENGTH);
                    data.insert(0, NEW_LOG_MESSAGE_TYPE); // add message type
                    data.extend(new_log.iter());
                    data.push(10); // add new line
                    data
                }
            };
            let is_stop = write(&mut self.socket, &self.socket_addr, &data).await?;
            if is_stop {
                return Ok(());
            }
        }
    }
}

/// If function returns true -> we need to stop read/write loop.
async fn write(
    socket: &mut TcpStream,
    socket_addr: &SocketAddr,
    data: &[u8],
) -> Result<bool, Box<dyn std::error::Error>> {
    match socket.write_all(data).await {
        Ok(()) => Ok(false),
        Err(e) => match e.kind() {
            std::io::ErrorKind::BrokenPipe => {
                debug!("cannot send data; looks like {} client disconnected", socket_addr);
                Ok(true)
            }
            _ => Err(e.to_string().into()),
        },
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        trace!("dropping Connection for {} client", self.socket_addr);
        self.connection_counter
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(x - 1))
            .unwrap();
    }
}
