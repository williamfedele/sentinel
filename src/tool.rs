use notify::Result;
use std::path::Path;
use std::process::{Command, Output};
use tokio::time::Instant;

pub enum Tool {
    RuffFormat,
    RuffCheck,
}

pub async fn run_tool(tool: &Tool, path: &Path) -> Result<Output> {
    match tool {
        Tool::RuffFormat => {
            print!("Running ruff format...");
            // get current time
            let now = Instant::now();
            let output = Command::new("ruff").arg("format").arg(path).output()?;
            let elapsed = now.elapsed();
            println!(" ✓ ({:.2?})", elapsed);
            return Ok(output);
        }
        Tool::RuffCheck => {
            print!("Running ruff check...");
            // get current time
            let now = Instant::now();
            let output = Command::new("ruff").arg("check").arg(path).output()?;
            let elapsed = now.elapsed();
            println!(" ✓ ({:.2?})", elapsed);
            return Ok(output);
        }
        _ => return Err(notify::Error::generic("Tool not implemented".into())),
    }
}
