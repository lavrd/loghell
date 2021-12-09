use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use env_logger::{Builder, Env};
use log::{debug, error, info};
use tokio::sync::Mutex;

use crate::storage::Storage;

mod config;
mod server;
mod storage;

const DEFAULT_LOG_LEVEL: &str = "TRACE";
const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:0";
const DEFAULT_STORAGE: &str = "tantivy";
const DEFAULT_CONFIG_PATH: &str = "./contrib/config.yaml";

const ENV_SOCKET_ADDR: &str = "SOCKET_ADDR";
const ENV_STORAGE: &str = "STORAGE";
const ENV_CONFIG_PATH: &str = "CONFIG_PATH";

#[repr(u8)]
enum ExitCode {
    Ok = 0,
    FailedToStartDaemon = 201,
    FailedToStopDaemon = 202,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Builder::from_env(Env::default().default_filter_or(DEFAULT_LOG_LEVEL)).init();

    let socket_addr = env::var(ENV_SOCKET_ADDR).unwrap_or_else(|_| DEFAULT_SOCKET_ADDR.to_string());
    let storage_name = env::var(ENV_STORAGE).unwrap_or_else(|_| DEFAULT_STORAGE.to_string());
    let config_path = env::var(ENV_CONFIG_PATH).unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
    let dashboard_content = include_str!("../dashboard/index.html");

    let config = config::Config::new(&config_path)?;
    let storage = Arc::new(Mutex::new(storage::new_storage(
        &storage_name,
        config.storage,
    )?));
    let connection_counter = Arc::new(AtomicU64::new(0));
    let mut server = server::Server::new(
        socket_addr,
        dashboard_content.to_string(),
        storage,
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

    println!(
        "server open connections : {}",
        connection_counter.load(Ordering::Relaxed)
    );
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
