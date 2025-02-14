use clap::Parser;
use notify::Result;
use sentinel::Sentinel;

mod config;
mod sentinel;
mod tool;
mod utils;

#[derive(Parser)]
#[clap(name = "Sentinel", version = "0.1.0", author = "")]
struct Cli {
    #[arg(short, long, default_value = ".")] // default to current directory
    dir: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sentinel = Sentinel::new(cli.dir)?;
    futures::executor::block_on(sentinel.watch())?;
    Ok(())
}
