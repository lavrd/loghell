use env_logger::Env;
use log::{debug, error};
use std::env;

mod daemon;

const DEFAULT_LOG_LEVEL: &str = "DEBUG";
const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:0";

const ENV_SOCKET_ADDR: &str = "SOCKET_ADDR";

#[repr(u8)]
enum ExitCode {
    FailedToStartDaemon = 201,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or(DEFAULT_LOG_LEVEL)).init();

    let socket_addr = env::var(ENV_SOCKET_ADDR).unwrap_or_else(|_| DEFAULT_SOCKET_ADDR.to_string());

    tokio::spawn(async move {
        match daemon::Daemon::new(socket_addr).start().await {
            Ok(_) => debug!("daemon stopped successfully"),
            Err(e) => {
                error!("failed to start daemon : {}", e);
                std::process::exit(ExitCode::FailedToStartDaemon as i32);
            }
        }
    });

    tokio::signal::ctrl_c().await?;

    // std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}
