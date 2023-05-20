use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{Args, Parser, Subcommand};
use hyper::{Body, Client, Method, Request, StatusCode};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[derive(Debug, Parser)]
#[clap(name = "loghellctl")]
#[clap(about = "Command line utility to interact with Loghell", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    #[clap(flatten)]
    args: GlobalArgs,
}

#[derive(Debug, Args)]
struct GlobalArgs {
    /// Setup Loghell endpoint
    #[clap(short, long, default_value = "127.0.0.1:6669")]
    endpoint: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Check Loghell health status
    Health,
    /// Simulate sending logs to Loghell
    Simulate,
}

#[tokio::main]
async fn main() {
    match do_main().await {
        Ok(_) => std::process::exit(0),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

async fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let endpoint = cli.args.endpoint;
    match cli.command {
        Commands::Health => health(&endpoint).await?,
        Commands::Simulate => simulation(&endpoint).await?,
    }
    Ok(())
}

async fn health(endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let builder = Request::builder().method(Method::GET).uri(format!("http://{}/health", endpoint));
    let req = builder.body(Body::empty())?;
    let res = client.request(req).await.map_err(|e| format!("failed to send request: {}", e))?;
    if res.status().as_u16() != StatusCode::OK {
        return Err(format!("incorrect response status code: {}", res.status().as_u16()).into());
    }
    Ok(())
}

async fn simulation(endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(endpoint).await?;
    for _ in 0..100 {
        // {"level":"debug","component":"example","time":"1684607842880484000","message":"example debug log"}
        let mut data = String::new();
        data.push_str(r#"{"level":"debug","component":"example","time":""#);
        data.push_str(&now_as_nanos_u64()?.to_string());
        data.push_str(r#"","message":"example debug log"}"#);
        stream.write_all(data.as_bytes()).await?;
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Ok(())
}

pub(crate) fn now_as_nanos_u64() -> Result<u64, Box<dyn std::error::Error>> {
    let now_as_nanos_u128 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let now_as_nanos_u64 = u64::try_from(now_as_nanos_u128)?;
    Ok(now_as_nanos_u64)
}
