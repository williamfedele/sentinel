use clap::Parser;
use notify::Result;
use sentinel::Sentinel;

mod config;
mod sentinel;
mod utils;

#[derive(Parser)]
#[clap(name = "Sentinel", version = "0.1.0", author = "")]
struct Cli {
    #[arg(short, long, default_value = ".")] // default to current directory
    dir: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = match config::Config::load_config(cli.dir.clone()) {
        Some(config) => config,
        None => {
            eprintln!("No config file found");
            eprintln!("Please create a .sentinel.yaml file in the project root");
            if cfg!(windows) {
                eprintln!("Or create a global config file at: %APPDATA%/sentinel/global.yaml");
            } else if cfg!(target_os = "macos") {
                eprintln!("Or create a global config file at: $HOME/Library/Application Support/sentinel/global.yaml");
            } else {
                eprintln!(
                    "Or create a global config file at: $XDG_CONFIG_HOME/sentinel/global.yaml"
                );
            }
            return Ok(());
        }
    };

    let mut sentinel = Sentinel::new(cli.dir, config)?;
    futures::executor::block_on(sentinel.watch())?;
    Ok(())
}
