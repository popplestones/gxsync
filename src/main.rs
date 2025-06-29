use clap::Parser;
use gxsync::cli::CliArgs;
use gxsync::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = CliArgs::parse();
    run(args).await
}
