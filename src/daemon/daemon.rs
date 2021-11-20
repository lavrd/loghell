use log::{debug, error, info};
use std::str::from_utf8;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub struct Daemon {
    socket_addr: String,
}

impl Daemon {
    pub fn new(socket_addr: String) -> Self { Daemon { socket_addr } }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.socket_addr).await?;
        let local_addr = listener.local_addr()?;
        info!("socket start at {}", local_addr);

        loop {
            let (mut socket, socket_addr) = listener.accept().await?;
            debug!("new client from {}", socket_addr.to_string());

            tokio::spawn(async move {
                let mut buf = [0; 1024];

                loop {
                    let n = match socket.read(&mut buf).await {
                        Ok(n) if n == 0 => {
                            debug!("connection with {} socket closed", socket_addr);
                            return;
                        }
                        Ok(n) if n == 5 && buf[0..n] == [255, 244, 255, 253, 6] => {
                            debug!("telnet connection with {} socket closed", socket_addr);
                            return;
                        }
                        Ok(n) => {
                            // We use n-2 to remove /r/n.
                            let data = match from_utf8(&buf[0..n - 2]) {
                                Ok(data) => data,
                                Err(e) => {
                                    error!("failed to convert incoming data to string : {}", e);
                                    return;
                                }
                            };
                            debug!("new data from {} : {:?}", socket_addr, data);
                            n
                        }
                        Err(e) => {
                            error!("failed to read from socket; err : {:?}", e);
                            0
                        }
                    };

                    if let Err(e) = socket.write_all(&buf[0..n]).await {
                        error!("failed to write to socket; err : {:?}", e);
                        return;
                    }
                };
            });
        }
    }
}
