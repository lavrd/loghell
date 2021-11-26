use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::str::from_utf8;

use log::{debug, error, info, trace};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;

enum ProcessDataResult {
    Ok,
    Close,
}

pub struct Server {
    socket_addr: String,
}

impl Server {
    pub fn new(socket_addr: String) -> Self {
        Server { socket_addr }
    }

    pub async fn start(&self, shutdown_rx: watch::Receiver<()>) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.socket_addr).await?;
        /*
           We get local address from listener instead of use from &self
            because of we can pass local address with zero port which can be selected randomly.
        */
        let local_addr = listener.local_addr()?;
        info!("socket starts at : {}", local_addr);

        let mut _shutdown_rx = shutdown_rx.clone();
        tokio::select! {
            res = self.accept(listener, shutdown_rx.clone()) => { res }
            _ = _shutdown_rx.changed() => {
                debug!("terminating accept new clients loop");
                Ok(())
            }
        }
    }

    async fn accept(
        &self,
        listener: TcpListener,
        shutdown_rx: watch::Receiver<()>,
    ) -> Result<(), Box<dyn Error>> {
        loop {
            let (socket, socket_addr) = listener.accept().await?;
            info!("new client; ip : {}", socket_addr);

            let _shutdown_rx = shutdown_rx.clone();
            tokio::spawn(async move {
                trace!("spawn thread for {} client", socket_addr);
                // TODO: Try to move outside spawn.
                let mut connection = Connection::new(socket, socket_addr, _shutdown_rx);
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
        trace!("dropping Daemon")
    }
}

struct Connection {
    socket: TcpStream,
    socket_addr: SocketAddr,
    shutdown_rx: watch::Receiver<()>,
}

impl Connection {
    fn new(socket: TcpStream, socket_addr: SocketAddr, shutdown_rx: watch::Receiver<()>) -> Self {
        Connection {
            socket,
            socket_addr,
            shutdown_rx,
        }
    }

    async fn process_socket(&mut self) {
        let mut _shutdown_rx = self.shutdown_rx.clone();
        tokio::select! {
            res = self.read_data() => {
                match res {
                    None => debug!("connection with {} client closed successfully", self.socket_addr),
                    Some(e) => error!("failed to read data from socket; client : {}; : {}", self.socket_addr, e)
                }
            }
            _ = _shutdown_rx.changed() => {
                trace!("terminating read data loop; client : {}", self.socket_addr);
            }
        }

        match self.socket.shutdown().await {
            Ok(()) => trace!("successfully shutdown {} socket", self.socket_addr),
            Err(e) => {
                error!(
                    "failed to shutdown socket with {} address : {}",
                    self.socket_addr, e
                )
            }
        }

        trace!("we have moved from select in Connection.process_socket");
    }

    async fn read_data(&mut self) -> Option<Box<dyn Error>> {
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
                    return Some(e.to_string().into());
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
                    return None;
                }
                Err(e) => {
                    error!(
                        "failed to process data from {} client: {}",
                        self.socket_addr, e
                    );
                    return Some(e);
                }
            };
        }
    }

    async fn process_data(
        &mut self,
        n: usize,
        buf: &[u8],
    ) -> Result<ProcessDataResult, Box<dyn Error>> {
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
                // TODO: This is tested only with telnet client.
                // TODO: May be we should move it to tcp handler only?
                // We use n-2 to remove /r/n.
                let truncated_buf: &[u8] = &buf[0..n - 2];

                if truncated_buf.len() >= 14 && &truncated_buf[0..14] == b"GET / HTTP/1.1" {
                    return self
                        .handle_dashboard()
                        .await
                        .map(|_| Ok(ProcessDataResult::Close))?;
                }
                self.handle_log(truncated_buf)
                    .await
                    .map(|_| Ok(ProcessDataResult::Ok))?
            }
        }
    }

    async fn handle_dashboard(&mut self) -> Result<(), Box<dyn Error>> {
        let contents = fs::read_to_string("./dashboard/index.html").unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );
        self.socket.write_all(response.as_bytes()).await?;
        self.socket.flush().await?;
        info!("send dashboard for {} client", self.socket_addr);
        Ok(())
    }

    async fn handle_log(&self, buf: &[u8]) -> Result<(), Box<dyn Error>> {
        // Convert bytes to string.
        let data = from_utf8(buf)?;
        info!(
            "new data received from {} client : {:?}",
            self.socket_addr, data
        );
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        trace!("dropping Connection for {} client", self.socket_addr)
    }
}
