use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use crate::shared::FnRes;

mod server;
mod shared;
mod storage;

const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:0";
const DEFAULT_STORAGE: &str = "nonsense";

const ENV_SOCKET_ADDR: &str = "SOCKET_ADDR";
const ENV_STORAGE: &str = "STORAGE";

#[repr(u8)]
enum ExitCode {
    Ok = 0,
    FailedToStartDaemon = 201,
    FailedToStopDaemon = 202,
}

#[tokio::main]
async fn main() -> FnRes<()> {
    let filter = Targets::new()
        .with_target("loghell", tracing::Level::TRACE)
        .with_default(LevelFilter::OFF);
    let terminal_subscriber = fmt::Layer::new().with_writer(std::io::stdout);
    let subscriber = tracing_subscriber::registry().with(filter).with(terminal_subscriber);
    tracing::subscriber::set_global_default(subscriber).expect("failed to set global subscriber");

    let socket_addr = env::var(ENV_SOCKET_ADDR).unwrap_or_else(|_| DEFAULT_SOCKET_ADDR.to_string());
    let storage_name = env::var(ENV_STORAGE).unwrap_or_else(|_| DEFAULT_STORAGE.to_string());
    let dashboard_content = include_str!("../dashboard/index.html");

    let storage = storage::new_storage(&storage_name)?;
    let shared_storage = Arc::new(Mutex::new(storage));
    let connection_counter = Arc::new(AtomicU64::new(0));
    let mut server = server::Server::new(
        socket_addr,
        dashboard_content.to_string(),
        shared_storage,
        connection_counter.clone(),
    );
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(());

    tokio::spawn(async move {
        match server.start(shutdown_rx).await {
            Ok(()) => debug!("server has been stopped successfully"),
            Err(e) => {
                error!("failed to start server : {}", e);
                std::process::exit(ExitCode::FailedToStartDaemon as i32);
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
            std::process::exit(ExitCode::FailedToStopDaemon as i32);
        }
        _ = shutdown_tx.closed() => {
            debug!("server successfully stopped");
        }
    }

    std::process::exit(ExitCode::Ok as i32);
}
