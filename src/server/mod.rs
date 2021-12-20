use std::net::SocketAddr;
use std::str::from_utf8;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use log::{debug, error, info, trace};
use regex::Regex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{watch, Mutex};

use crate::storage::Storage;
use crate::FnRes;

enum ProcessDataResult {
    Ok,
    Close,
}

pub struct Server {
    socket_addr: String,
    dashboard_content: String,
    storage: Arc<Mutex<Storage>>,
    connection_counter: Arc<AtomicU64>,
}

impl Server {
    pub fn new(
        socket_addr: String,
        dashboard_content: String,
        storage: Arc<Mutex<Storage>>,
        connection_counter: Arc<AtomicU64>,
    ) -> Self {
        Server {
            socket_addr,
            dashboard_content,
            storage,
            connection_counter,
        }
    }

    pub async fn start(&mut self, shutdown_rx: watch::Receiver<()>) -> FnRes<()> {
        let listener = TcpListener::bind(&self.socket_addr).await?;
        /*
           We get local address from listener instead of use from &self
            because of we can pass local address with zero port which can be selected randomly.
        */
        let local_addr = listener.local_addr()?;
        info!("socket starts at : {}", local_addr);

        let re = Regex::new(r"\{\{port}}").unwrap();
        self.dashboard_content = re
            .replace(&self.dashboard_content, &local_addr.port().to_string())
            .to_string();

        let mut _shutdown_rx = shutdown_rx.clone();
        tokio::select! {
            res = self.accept(listener, shutdown_rx.clone()) => { res }
            _ = _shutdown_rx.changed() => {
                debug!("terminating accept new clients loop");
                Ok(())
            }
        }
    }

    async fn accept(&self, listener: TcpListener, shutdown_rx: watch::Receiver<()>) -> FnRes<()> {
        loop {
            let (socket, socket_addr) = listener.accept().await?;
            info!("new client; ip : {}", socket_addr);
            self.connection_counter.fetch_add(1, Ordering::Relaxed);

            let mut connection = Connection::new(
                socket,
                socket_addr,
                shutdown_rx.clone(),
                self.dashboard_content.clone(),
                self.storage.clone(),
                self.connection_counter.clone(),
            );
            tokio::spawn(async move {
                trace!("spawn thread for {} client", socket_addr);
                connection.process_socket().await;
                trace!(
                    "moving from spawn in accept loop for {} client",
                    socket_addr
                );
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
    storage: Arc<Mutex<Storage>>,
    connection_counter: Arc<AtomicU64>,
}

impl Connection {
    fn new(
        socket: TcpStream,
        socket_addr: SocketAddr,
        shutdown_rx: watch::Receiver<()>,
        dashboard_content: String,
        storage: Arc<Mutex<Storage>>,
        connection_counter: Arc<AtomicU64>,
    ) -> Self {
        Connection {
            socket,
            socket_addr,
            shutdown_rx,
            dashboard_content,
            storage,
            connection_counter,
        }
    }

    async fn process_socket(&mut self) {
        let mut _shutdown_rx = self.shutdown_rx.clone();
        tokio::select! {
            res = self.read_data() => {
                match res {
                    Ok(()) => debug!("connection with {} client closed successfully", self.socket_addr),
                    Err(e) => error!("failed to read data from socket; client : {}; : {}", self.socket_addr, e)
                }
            }
            _ = _shutdown_rx.changed() => {
                trace!("terminating read data loop; client : {}", self.socket_addr);
            }
        }

        match self.socket.shutdown().await {
            Ok(()) => trace!("successfully shutdown {} socket", self.socket_addr),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotConnected => debug!(
                    "cannot shutdown {} socket because it is already disconnected : {}",
                    self.socket_addr, e
                ),
                _ => error!(
                    "failed to shutdown socket with {} address : {}",
                    self.socket_addr, e
                ),
            },
        }

        trace!("we have moved from select in Connection.process_socket");
    }

    async fn read_data(&mut self) -> FnRes<()> {
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
                    trace!(
                        "data from {} client successfully proceeded",
                        self.socket_addr
                    );
                    // Read next frame from socket.
                    continue;
                }
                Ok(ProcessDataResult::Close) => {
                    trace!("close connection with {} client", self.socket_addr);
                    // Go away from read loop to close connection with client.
                    return Ok(());
                }
                Err(e) => {
                    error!(
                        "failed to process data from {} client: {}",
                        self.socket_addr, e
                    );
                    return Err(e.to_string().into());
                }
            };
        }
    }

    async fn process_data(&mut self, n: usize, buf: &[u8]) -> FnRes<ProcessDataResult> {
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
                // TODO: May be we should move it to log handler only?
                // We use n-2 to remove /r/n at the end of data.
                let truncated_buf: &[u8] = &buf[0..n - 2];

                if truncated_buf.starts_with(b"GET / HTTP/1.1") {
                    return self
                        .handle_dashboard()
                        .await
                        .map(|_| Ok(ProcessDataResult::Close))?;
                }
                if truncated_buf.starts_with(b"GET /sse HTTP/1.1") {
                    return self
                        .handle_sse()
                        .await
                        .map(|_| Ok(ProcessDataResult::Close))?;
                }
                self.handle_log(truncated_buf)
                    .await
                    .map(|_| Ok(ProcessDataResult::Ok))?
            }
        }
    }

    async fn handle_dashboard(&mut self) -> FnRes<()> {
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

    async fn handle_log(&self, buf: &[u8]) -> FnRes<()> {
        // Convert bytes to string.
        let data = from_utf8(buf)?;
        info!(
            "new data received from {} client : {:?}",
            self.socket_addr, data
        );
        self.storage.lock().await.store(buf)?;
        Ok(())
    }

    async fn handle_sse(&mut self) -> FnRes<()> {
        let response = "HTTP/1.1 200 OK
Connection: keep-alive
Content-Type: text/event-stream
Cache-Control: no-cache
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET";
        self.socket.write_all(response.as_bytes()).await?;
        self.socket.flush().await?;

        self.socket.write_all(b"retry: 10000\n").await?;
        self.socket.flush().await?;
        self.socket.write_all(b"event: data\n").await?;
        self.socket.flush().await?;

        let mut _shutdown_rx = self.shutdown_rx.clone();
        tokio::select! {
            res = self.send_sse_data() => { res },
            _ = _shutdown_rx.changed() => {
                trace!("terminating sse send data loop; client : {}", self.socket_addr);
                Ok(())
            }
        }
    }

    async fn send_sse_data(&mut self) -> FnRes<()> {
        loop {
            match self.socket.write_all(b"data\n\n").await {
                Ok(()) => Ok(()),
                Err(e) => match e.kind() {
                    std::io::ErrorKind::BrokenPipe => {
                        debug!(
                            "cannot send sse data; looks like {} client has disconnected",
                            self.socket_addr
                        );
                        return Ok(());
                    }
                    _ => Err(e),
                },
            }?;
            self.socket.flush().await?;
            trace!("sent sse data to {} client", self.socket_addr);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
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