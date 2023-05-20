use clap::{Args, Parser, Subcommand};
use hyper::{Body, Client, Method, Request, StatusCode};

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
