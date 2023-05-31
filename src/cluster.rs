use std::{sync::Arc, time::Duration};

use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::Mutex,
};
use tracing::{debug, error, trace};

use crate::log_storage::LogStorage;

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct ClusterState {
    pub(crate) _asd: u64,
}

pub(crate) type ClusterStateReader = tokio::sync::broadcast::Receiver<ClusterState>;
pub(crate) type ClusterStateTransmitter = tokio::sync::broadcast::Sender<ClusterState>;

pub(crate) struct Cluster {
    cst: ClusterStateTransmitter, // cluster state transmitter
    // We need to store it in order to not close transmitter channel.
    _csr: ClusterStateReader,
}

impl Cluster {
    pub(crate) fn new() -> (Self, ClusterStateTransmitter) {
        let (tx, rx) = tokio::sync::broadcast::channel(100);
        (
            Self {
                cst: tx.clone(),
                _csr: rx,
            },
            tx,
        )
    }

    pub(crate) async fn start(
        &self,
        addrs: String,
        log_storage: Arc<Mutex<LogStorage>>,
        mut shutdown_rx: tokio::sync::watch::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.connect(&addrs, log_storage).await?;
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
            tokio::time::sleep(Duration::from_secs(3)).await;
            return Ok(());
        }
        self.cst
            .send(ClusterState::default())
            .map_err(|e| Error::TransmitClusterStateError(e.to_string()))?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    async fn connect(
        &self,
        addrs: &str,
        log_storage: Arc<Mutex<LogStorage>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let addrs = addrs.split(',');
        for addr in addrs {
            if addr.is_empty() {
                continue;
            }
            let addr = addr.to_string();
            let stream = TcpStream::connect(&addr).await?;
            let log_storage_ = log_storage.clone();
            tokio::spawn(async move {
                match listen(stream, log_storage_).await {
                    Ok(_) => debug!(?addr, "connection stopped"),
                    Err(e) => error!(?e, "failed to listen stream"),
                }
            });
        }
        Ok(())
    }
}

async fn listen(
    mut stream: TcpStream,
    log_storage: Arc<Mutex<LogStorage>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Notify TCP server that it is cluster connection.
    stream.write_all(b"cluster>").await?;
    let mut reader = BufReader::new(stream);
    loop {
        {
            let mut buf: String = String::new();
            reader.read_line(&mut buf).await?;
            if buf.is_empty() {
                break;
            }
            // Delete new line.
            buf.pop();
            log_storage.lock().await.store(buf.as_bytes()).await?;
        }
    }
    Ok(())
}

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("failed to transmit cluster state: {0}")]
    TransmitClusterStateError(String),
}
