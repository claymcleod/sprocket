//! Sprocket server binary entry point.

use anyhow::Context;
use clap::Parser;
use clap_verbosity_flag::InfoLevel;
use clap_verbosity_flag::Verbosity;
use sprocket_server::Config;

/// REST API server for executing WDL workflows.
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Configuration file path.
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

async fn inner() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(args.verbose)
        .init();

    let config = if let Some(path) = args.config {
        Config::from_file(&path).context("failed to load configuration from file")?
    } else {
        Config::default()
    };

    sprocket_server::run(config).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    match inner().await {
        Ok(_) => {}
        Err(err) => eprintln!("error: {err}"),
    }
}
