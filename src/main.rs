use clap::Parser;
use futures::future::join_all;
use notify::event::ModifyKind;
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::time::Duration;
use tokio::time;
use tokio::time::Instant;

#[derive(Parser)]
#[clap(name = "Mesa", version = "0.1.0", author = "")]
struct Cli {
    #[arg(short, long, default_value = ".")] // default to current directory
    dir: String,
}

struct ToolConfig {
    python_tools: Vec<Tool>,
}

enum Tool {
    Ruff,
}

struct Mesa {
    dir: String,
    tools: ToolConfig,
}

impl Mesa {
    fn new(dir: String) -> Result<Self> {
        Ok(Self {
            dir,
            tools: ToolConfig {
                python_tools: vec![Tool::Ruff],
            },
        })
    }

    async fn watch(&self) -> Result<()> {
        let (tx, rx) = mpsc::channel::<Result<Event>>();
        let mut watcher = recommended_watcher(tx)?;
        watcher.watch(Path::new(&self.dir), RecursiveMode::Recursive)?;

        let mut last_event: Option<(PathBuf, Instant)> = None;

        for res in rx {
            match res {
                Ok(event) => match event.kind {
                    EventKind::Modify(ModifyKind::Data(_)) => {
                        let current_path = &event.paths[0];

                        // Debounce events
                        // Check if the event is a duplicate
                        if let Some((last_path, last_time)) = &last_event {
                            if last_path == current_path
                                && last_time.elapsed() < Duration::from_millis(100)
                            {
                                // Ignore duplicate event
                                continue;
                            }
                        }

                        // Update the last event
                        last_event = Some((current_path.clone(), time::Instant::now()));
                        self.process_file(&event.paths[0]).await?
                    }
                    _ => continue,
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }

        Ok(())
    }

    async fn process_file(&self, path: &Path) -> Result<()> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("py") => {
                println!("Changed: {}", path.display());
                let tools = &self.tools.python_tools;
                let results = join_all(tools.iter().map(|tool| self.run_tool(tool, path))).await;
                self.display_results(results);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn run_tool(&self, tool: &Tool, path: &Path) -> Result<Output> {
        match tool {
            Tool::Ruff => {
                print!("Running ruff...");
                // get current time
                let now = Instant::now();
                let output = Command::new("ruff").arg("format").arg(path).output()?;
                let elapsed = now.elapsed();
                println!(" ✓ ({:.2?})", elapsed);
                return Ok(output);
            }
            _ => return Err(notify::Error::generic("Tool not implemented".into())),
        }
    }

    fn display_results(&self, results: Vec<Result<Output>>) {
        for result in results {
            match result {
                Ok(output) => {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mesa = Mesa::new(cli.dir)?;
    futures::executor::block_on(mesa.watch())?;
    Ok(())
}
