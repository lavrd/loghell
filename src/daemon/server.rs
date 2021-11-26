use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, error, info, trace};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::{Mutex, watch};

use crate::daemon::dashboard::Dashboard;
use crate::daemon::handler::Handler;
use crate::daemon::tcp::TCP;

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
    reader: OwnedReadHalf,
    socket_addr: SocketAddr,
    shutdown_rx: watch::Receiver<()>,
    http_handler: Arc<Mutex<Dashboard>>,
    tcp_handler: Arc<Mutex<TCP>>,
}

impl Connection {
    fn new(socket: TcpStream, socket_addr: SocketAddr, shutdown_rx: watch::Receiver<()>) -> Self {
        let (reader, writer) = socket.into_split();
        let _shutdown_rx = shutdown_rx.clone();
        Connection {
            reader,
            socket_addr,
            shutdown_rx: _shutdown_rx,
            http_handler: Arc::new(Mutex::new(Dashboard::new(socket_addr, writer))),
            tcp_handler: Arc::new(Mutex::new(TCP::new(socket_addr))),
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

        // TODO: How to shutdown obviously?
        // match self.socket.shutdown().await {
        //     Ok(()) => trace!("successfully shutdown {} socket", self.socket_addr),
        //     Err(e) => {
        //         error!(
        //             "failed to shutdown socket with {} address : {}",
        //             self.socket_addr, e
        //         )
        //     }
        // }

        trace!("we have moved from select in Connection.process_socket");
    }

    async fn read_data(&mut self) -> Option<Box<dyn std::error::Error>> {
        loop {
            let mut buf = [0; 1024];
            // Read data from socket.
            let n = match self.reader.read(&mut buf).await {
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
                // TODO: May be we should move it to tcp handler only?
                // We use n-2 to remove /r/n.
                let truncated_buf: &[u8] = &buf[0..n - 2];

                let handler: Arc<Mutex<dyn Handler>> = match truncated_buf {
                    truncated_buf
                    if truncated_buf.len() >= 14
                        && &truncated_buf[0..14] == b"GET / HTTP/1.1" =>
                        {
                            self.http_handler.clone()
                        }
                    _ => self.tcp_handler.clone(),
                };

                let mut bh = handler.try_lock().unwrap();

                match bh.handle(truncated_buf).await {
                    None => Ok(ProcessDataResult::Ok),
                    Some(e) => Err(e.to_string().into()),
                }
            }
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        trace!("dropping Connection for {} client", self.socket_addr)
    }
}
