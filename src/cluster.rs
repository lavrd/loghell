use std::time::Duration;

use thiserror::Error;
use tracing::{debug, error, trace};

#[derive(Default, Debug)]
pub(crate) struct ClusterState {
    pub(crate) asd: u64,
}

pub(crate) type ClusterStateReader = tokio::sync::watch::Receiver<ClusterState>;
pub(crate) type ClusterStateTransmitter = tokio::sync::watch::Sender<ClusterState>;

pub(crate) struct Cluster {
    addrs: String,
    cst: ClusterStateTransmitter, // cluster state transmitter
}

impl Cluster {
    pub(crate) fn new(addrs: String) -> (Self, ClusterStateReader) {
        let (tx, rx) = tokio::sync::watch::channel(ClusterState::default());
        (Self { addrs, cst: tx }, rx)
    }

    pub(crate) async fn start(&self, mut shutdown_rx: tokio::sync::watch::Receiver<()>) {
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    debug!("received shutdown signal; stop routine");
                    return;
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
            trace!(?self.addrs, "there are no receivers to transmit state");
        }
        self.cst
            .send(ClusterState::default())
            .map_err(|e| Error::TransmitClusterStateError(e.to_string()))?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("failed to transmit cluster state: {0}")]
    TransmitClusterStateError(String),
}
