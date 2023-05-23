use std::time::Duration;

use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use tracing::{debug, error, trace};

#[derive(Default, Debug)]
pub(crate) struct ClusterState {
    pub(crate) _asd: u64,
}

pub(crate) type ClusterStateReader = tokio::sync::watch::Receiver<ClusterState>;
pub(crate) type ClusterStateTransmitter = tokio::sync::watch::Sender<ClusterState>;

pub(crate) struct Cluster {
    cst: ClusterStateTransmitter, // cluster state transmitter
}

impl Cluster {
    pub(crate) fn new() -> (Self, ClusterStateReader) {
        let (tx, rx) = tokio::sync::watch::channel(ClusterState::default());
        (Self { cst: tx }, rx)
    }

    pub(crate) async fn start(
        &self,
        addrs: String,
        mut shutdown_rx: tokio::sync::watch::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.connect(&addrs).await?;
        // At the moment we don't want to implement dynamic cluster changing.
        if !addrs.is_empty() {
            return Ok(());
        }
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    debug!("received shutdown signal; stop routine");
                    return Ok(());
                }
                res = self.tick() => {
                    match res {
                        Ok(_) => (),
                        Err(e) => match e {
                            Error::TransmitClusterStateError(e) => {
                                error!(?e, "failed to transmit cluster state")
                            }
                        }
                    }
                }
            }
        }
    }

    async fn tick(&self) -> Result<(), Error> {
        // We compare with "1" because 1 is a default receiver in server struct.
        // For each connection it is incrementing by 1, so 1 connection = 2 receivers.
        if self.cst.receiver_count() == 1 {
            trace!("there are no receivers to transmit state");
        }
        self.cst
            .send(ClusterState::default())
            .map_err(|e| Error::TransmitClusterStateError(e.to_string()))?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    async fn connect(&self, addrs: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addrs = addrs.split(',');
        for addr in addrs {
            if addr.is_empty() {
                continue;
            }
            let addr = addr.to_string();
            let stream = TcpStream::connect(&addr).await?;
            tokio::spawn(async move {
                match listen(stream).await {
                    Ok(_) => debug!(?addr, "connection stopped"),
                    Err(e) => error!(?e, "failed to listen stream"),
                }
            });
        }
        Ok(())
    }
}

async fn listen(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    stream.write_all(b"cluster>").await?;
    let remote_addr = stream.peer_addr()?;
    let mut reader = BufReader::new(stream);
    loop {
        // todo: i have the same source code in loghellctl
        {
            let mut buf: String = String::new();
            reader.read_line(&mut buf).await?;
            if buf.is_empty() {
                break;
            }
            // Delete new line.
            buf.pop();
            eprintln!("new message in a cluster from {} - {}", remote_addr, buf);
        }
    }
    Ok(())
}

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("failed to transmit cluster state: {0}")]
    TransmitClusterStateError(String),
}
