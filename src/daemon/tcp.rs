use log::{error, info};
use std::net::SocketAddr;
use std::str::from_utf8;

pub struct TCPHandler<'a> {
    buf: &'a [u8],
    socket_addr: SocketAddr,
}

impl<'a> TCPHandler<'a> {
    pub fn new(buf: &[u8], socket_addr: SocketAddr) -> TCPHandler {
        TCPHandler { buf, socket_addr }
    }

    pub fn handle(&self) -> Option<Box<dyn std::error::Error>> {
        // Convert bytes to string.
        let data = match from_utf8(self.buf) {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "failed to convert incoming data to string; err : {}; data : {:?}",
                    e, self.buf
                );
                return Some(e.to_string().into());
            }
        };
        info!(
            "new data received from {} client : {:?}",
            self.socket_addr, data
        );
        None
    }
}
