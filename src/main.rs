use env_logger::{Builder, Env};
use log::{debug, error, info};
use std::env;

mod daemon;

const DEFAULT_LOG_LEVEL: &str = "TRACE";
const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:0";

const ENV_SOCKET_ADDR: &str = "SOCKET_ADDR";

#[repr(u8)]
enum ExitCode {
    Ok = 0,
    FailedToStartDaemon = 201,
    FailedToStopDaemon = 202,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Builder::from_env(Env::default().default_filter_or(DEFAULT_LOG_LEVEL)).init();

    let socket_addr = env::var(ENV_SOCKET_ADDR).unwrap_or_else(|_| DEFAULT_SOCKET_ADDR.to_string());

    let server = daemon::Server::new(socket_addr);
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
