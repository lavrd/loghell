use std::error::Error;
use std::net::SocketAddr;

use tokio::io::{AsyncWrite, BufWriter};
use tokio::net::tcp::WriteHalf;
use tokio::net::TcpStream;

use crate::daemon::handler::Handler;

pub struct HTTP {
    socket_addr: SocketAddr,
    writer: Box<dyn AsyncWrite>,
}

impl HTTP {
    pub fn new(
        socket_addr: SocketAddr, writer: Box<dyn AsyncWrite>,
    ) -> Self {
        HTTP { socket_addr, writer }
    }
}

impl Handler for HTTP {
    fn handle(&self, buf: &[u8]) -> Option<Box<dyn Error>> {
        println!("{} : {:?}", self.socket_addr, buf);
        None
    }
}
