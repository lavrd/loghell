use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, trace};
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

mod config;
mod index;
mod log_storage;
mod server;
mod storage;

#[repr(u8)]
enum ExitCode {
    Ok = 0,
    FailedToStartDaemon = 201,
    FailedToStopDaemon = 202,
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(value: ExitCode) -> Self {
        std::process::ExitCode::from(value as u8)
    }
}

#[tokio::main]
async fn main() -> Result<std::process::ExitCode, Box<dyn std::error::Error>> {
    let dashboard_content = include_str!("../dashboard/index.html");

    let cfg = config::Config::new();

    let filter =
        Targets::new().with_target("loghell", tracing::Level::TRACE).with_default(LevelFilter::OFF);
    let terminal_subscriber = fmt::Layer::new().with_writer(std::io::stdout);
    let subscriber = tracing_subscriber::registry().with(filter).with(terminal_subscriber);
    tracing::subscriber::set_global_default(subscriber).expect("failed to set global subscriber");

    let log_storage =
        Arc::new(Mutex::new(log_storage::LogStorage::new(&cfg.index_name, &cfg.storage_name)?));

    let connection_counter = Arc::new(AtomicU64::new(0));
    let server =
        server::Server::new(dashboard_content.to_string(), connection_counter.clone(), log_storage);
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(());

    let socket_addr = cfg.socket_addr;
    let res: JoinHandle<ExitCode> = tokio::spawn(async move {
        match server.start(&socket_addr, shutdown_rx).await {
            Ok(()) => {
                debug!("server has been stopped successfully");
                ExitCode::Ok
            }
            Err(e) => {
                error!("failed to start server : {}", e);
                ExitCode::FailedToStartDaemon
            }
        }
    });

    tokio::signal::ctrl_c().await?;
    info!("ctrl+c signal has been received");

    trace!("server open connections : {}", connection_counter.load(Ordering::Relaxed));
    shutdown_tx.send(())?;
    let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(1));
    tokio::pin!(timeout);
    tokio::select! {
        _ = &mut timeout => {
            error!("server stopping is timed out");
            return Ok(ExitCode::FailedToStopDaemon.into())
        }
        _ = shutdown_tx.closed() => {
            debug!("server successfully stopped");
        }
    }

    // todo: check that it is work
    let start_exit_code: std::process::ExitCode = res.await?.into();
    eprintln!("{:?}", start_exit_code);

    Ok(ExitCode::Ok.into())
}
