use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use crate::shared::FnRes;

mod config;
mod index;
mod server;
mod shared;
mod storage;

#[repr(u8)]
enum ExitCode {
    Ok = 0,
    FailedToStartDaemon = 201,
    FailedToStopDaemon = 202,
}

#[tokio::main]
async fn main() -> FnRes<()> {
    let dashboard_content = include_str!("../dashboard/index.html");
    let cfg = config::Config::new();

    let filter =
        Targets::new().with_target("loghell", tracing::Level::TRACE).with_default(LevelFilter::OFF);
    let terminal_subscriber = fmt::Layer::new().with_writer(std::io::stdout);
    let subscriber = tracing_subscriber::registry().with(filter).with(terminal_subscriber);
    tracing::subscriber::set_global_default(subscriber).expect("failed to set global subscriber");

    let index = index::new_index(&cfg.index_name)?;
    let storage = storage::new_storage(&cfg.storage_name)?;

    let connection_counter = Arc::new(AtomicU64::new(0));
    let server = server::Server::new(
        &cfg.socket_addr,
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
