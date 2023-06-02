use std::sync::Arc;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use tracing::{debug, error};

use crate::{
    log_storage::{LogStoragePointer, Notifier},
    server, shared,
};

pub(crate) const NEW_LOG_MESSAGE_TYPE: u8 = 1;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    NewLog(Arc<Vec<u8>>),
}

pub(crate) type Transmitter = tokio::sync::broadcast::Sender<Message>;
pub(crate) type Reader = tokio::sync::broadcast::Receiver<Message>;

pub(crate) struct Cluster {
    cst: Transmitter, // cluster state transmitter
    // We need to store it in order to not close transmitter channel.
    _csr: Reader,
}

impl Cluster {
    pub(crate) fn new() -> (Self, Transmitter) {
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
        log_storage: LogStoragePointer,
        mut lsn: Notifier,
        mut shutdown_rx: tokio::sync::watch::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.connect(&addrs, log_storage).await?;
        // At the moment we don't want to implement dynamic cluster changing.
        if !addrs.is_empty() {
            return Ok(());
        }
        loop {
            tokio::select! {
                new_log = lsn.recv() => {
                    let new_log = new_log?;
                    shared::broadcast(&self.cst, Message::NewLog(new_log))?;
                }
                _ = shutdown_rx.changed() => {
                    debug!("received shutdown signal; stop routine");
                    return Ok(());
                }
            }
        }
    }

    async fn connect(
        &self,
        addrs: &str,
        log_storage: LogStoragePointer,
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
    log_storage: LogStoragePointer,
) -> Result<(), Box<dyn std::error::Error>> {
    // Notify TCP server that it is cluster connection.
    stream.write_all(server::CMD_CLUSTER.as_bytes()).await?;
    let mut reader = BufReader::new(stream);
    loop {
        let mut buf: Vec<u8> = Vec::new();
        reader.read_until(10, &mut buf).await?;
        if buf.is_empty() {
            break;
        }
        // Get and remove message type.
        let message_type = buf.remove(0);
        // Delete new line.
        buf.pop();
        match message_type {
            NEW_LOG_MESSAGE_TYPE => log_storage.lock().await.store(buf).await?,
            _ => error!("unknown first byte on cluster message: {}", buf[0]),
        }
    }
    Ok(())
}
