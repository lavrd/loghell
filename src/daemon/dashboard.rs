use std::error::Error;
use std::fs;
use std::net::SocketAddr;

use log::info;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;

use crate::daemon::handler::Handler;

pub struct Dashboard {
    socket_addr: SocketAddr,
    writer: OwnedWriteHalf,
}

impl Dashboard {
    pub fn new(socket_addr: SocketAddr, writer: OwnedWriteHalf) -> Self {
        Dashboard {
            socket_addr,
            writer,
        }
    }
}

impl Handler for Dashboard {
    fn handle(&mut self, _: &[u8]) -> Option<Box<dyn Error>> {
        let _socket_addr = self.socket_addr.clone();
        // TODO: How to check errors from async func?
        futures::executor::block_on(async move {
            let contents = fs::read_to_string("./dashboard/index.html").unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                contents.len(),
                contents
            );
            self.writer.write_all(response.as_bytes()).await.unwrap();
            self.writer.flush().await.unwrap();
        });
        info!("send dashboard for {} client", _socket_addr);
        None
    }
}
