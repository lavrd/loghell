use std::net::SocketAddr;
use std::str::from_utf8;

use async_trait::async_trait;
use log::{error, info};

use crate::daemon::handler::Handler;

// TODO: Rename. Something with logs handler.
pub struct TCP {
    socket_addr: SocketAddr,
}

impl TCP {
    pub fn new(socket_addr: SocketAddr) -> Self {
        TCP { socket_addr }
    }
}

#[async_trait]
impl Handler for TCP {
    async fn handle(&mut self, buf: &[u8]) -> Option<Box<dyn std::error::Error>> {
        // Convert bytes to string.
        let data = match from_utf8(buf) {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "failed to convert incoming data to string; err : {}; data : {:?}",
                    e, buf
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
