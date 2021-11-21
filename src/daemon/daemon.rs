use log::{debug, error, info, trace};
use std::net::SocketAddr;
use std::str::from_utf8;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;

enum ProcessDataResult {
    Ok,
    Close,
}

pub struct Daemon {
    socket_addr: String,
}

impl Daemon {
    pub fn new(socket_addr: String) -> Self {
        Daemon { socket_addr }
    }

    pub async fn start(
        &self,
        shutdown_rx: watch::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (socket, socket_addr) = listener.accept().await?;
            info!("new client; ip : {}", socket_addr);

            let _shutdown_rx = shutdown_rx.clone();
            tokio::spawn(async move {
                trace!("spawn thread for {} client", socket_addr);
                let mut handler = Handler {
                    socket,
                    socket_addr,
                    shutdown_rx: _shutdown_rx.clone(),
                };
                handler.process_socket().await;
                trace!(
                    "moving from spawn in accept loop for {} client",
                    socket_addr
                );
            });
        }
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        trace!("dropping Daemon")
    }
}

struct Handler {
    socket: TcpStream,
    socket_addr: SocketAddr,
    shutdown_rx: watch::Receiver<()>,
}

impl Handler {
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

        trace!("we have moved from select in Handler.process_socket");
    }

    async fn read_data(&mut self) -> Option<Box<dyn std::error::Error>> {
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
            match self.process_data(n, &buf) {
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

    fn process_data(
        &self,
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
                // TODO: This is tested only with telnet client.
                // We use n-2 to remove /r/n.
                let truncated_buf: &[u8] = &buf[0..n - 2];
                // Convert bytes to string.
                let data = match from_utf8(truncated_buf) {
                    Ok(data) => data,
                    Err(e) => {
                        error!(
                            "failed to convert incoming data to string; err : {}; data : {:?}",
                            e, truncated_buf
                        );
                        return Err(e.to_string().into());
                    }
                };
                info!(
                    "new data received from {} client : {:?}",
                    self.socket_addr, data
                );
                Ok(ProcessDataResult::Ok)
            }
        }
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        trace!("dropping Handler for {} client", self.socket_addr)
    }
}
