use std::env;
use std::error::Error;
use std::sync::Arc;

use env_logger::{Builder, Env};
use log::{debug, error, info};
use tokio::sync::Mutex;

use crate::storage::dummy::Dummy;
use crate::storage::tantivy::Tantivy;
use crate::storage::Storage;
use crate::storage_type::StorageType;

mod daemon;
mod storage;
mod storage_type;

const DEFAULT_LOG_LEVEL: &str = "TRACE";
const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:0";

const ENV_SOCKET_ADDR: &str = "SOCKET_ADDR";
const ENV_STORAGE: &str = "STORAGE";

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
    let storage_name = env::var(ENV_STORAGE).unwrap_or_else(|_| StorageType::Dummy.to_string());
    let dashboard_content = include_str!("../dashboard/index.html");

    let storage = Arc::new(Mutex::new(get_storage(&storage_name)?));

    let server = daemon::Server::new(socket_addr, dashboard_content.to_string(), storage);
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(());

    tokio::spawn(async move {
        match server.start(shutdown_rx).await {
            Ok(()) => debug!("daemon has been stopped successfully"),
            Err(e) => {
                error!("failed to start daemon : {}", e);
                std::process::exit(ExitCode::FailedToStartDaemon as i32);
            }
        }
    });

    tokio::signal::ctrl_c().await?;
    info!("ctrl+c signal has been received");

    shutdown_tx.send(())?;
    let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(1));
    tokio::pin!(timeout);
    tokio::select! {
        _ = &mut timeout => {
            error!("daemon stopping is timed out");
            std::process::exit(ExitCode::FailedToStopDaemon as i32);
        }
        _ = shutdown_tx.closed() => {
            debug!("daemon successfully stopped");
        }
    }

    std::process::exit(ExitCode::Ok as i32);
}

fn get_storage(storage_name: &str) -> Result<Box<dyn Storage + Send>, Box<dyn Error>> {
    let storage_type = storage_name.into();
    println!("{} {}", storage_name, storage_type);
    let storage: Box<dyn Storage + Send> = match storage_type {
        StorageType::Dummy => Box::new(Dummy::new()),
        StorageType::Tantivy => Box::new(Tantivy::new()),
        StorageType::Unknown => {
            return Err(format!("unknown storage type : {}", storage_name).into());
        }
    };
    info!("using {} as a logs storage", storage_type);
    Ok(storage)
}
