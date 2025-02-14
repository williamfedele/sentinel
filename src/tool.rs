use async_trait::async_trait;
use notify::Result;
use std::path::Path;
use std::process::{Command, Output};
use tokio::time::Instant;

#[async_trait]
pub trait Tool {
    fn name(&self) -> &'static str;
    async fn run(&self, path: &Path) -> Result<Output>;
}

pub struct RuffFormat;

#[async_trait]
impl Tool for RuffFormat {
    fn name(&self) -> &'static str {
        "ruff format"
    }

    async fn run(&self, path: &Path) -> Result<Output> {
        print!("Running {}...", self.name());
        // get current time
        let now = Instant::now();
        let output = Command::new("ruff").arg("format").arg(path).output()?;
        let elapsed = now.elapsed();
        println!(" ✓ ({:.2?})", elapsed);
        Ok(output)
    }
}

pub struct RuffCheck;

#[async_trait]
impl Tool for RuffCheck {
    fn name(&self) -> &'static str {
        "ruff check"
    }

    async fn run(&self, path: &Path) -> Result<Output> {
        print!("Running {}...", self.name());
        // get current time
        let now = Instant::now();
        let output = Command::new("ruff").arg("check").arg(path).output()?;
        let elapsed = now.elapsed();
        println!(" ✓ ({:.2?})", elapsed);
        Ok(output)
    }
}
