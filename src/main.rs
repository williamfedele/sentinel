use clap::Parser;
use notify::event::ModifyKind;
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time;
use tokio::time::Instant;

#[derive(Parser)]
#[clap(name = "Sentinel", version = "0.1.0", author = "")]
struct Cli {
    #[arg(short, long, default_value = ".")] // default to current directory
    dir: String,
}

struct ToolConfig {
    python_tools: Vec<Tool>,
}

enum Tool {
    RuffFormat,
    RuffCheck,
}

struct Sentinel {
    dir: String,
    tools: ToolConfig,
}

impl Sentinel {
    fn new(dir: String) -> Result<Self> {
        Ok(Self {
            dir,
            tools: ToolConfig {
                python_tools: vec![Tool::RuffFormat, Tool::RuffCheck],
            },
        })
    }

    async fn watch(&self) -> Result<()> {
        let (tx, rx) = mpsc::channel::<Result<Event>>();
        let mut watcher = recommended_watcher(tx)?;
        watcher.watch(Path::new(&self.dir), RecursiveMode::Recursive)?;

        let mut last_event: Option<(PathBuf, Instant)> = None;

        println!("Watching for changes...");
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
                // time and file that changed
                let (hour, minute, second) = Self::get_current_time();
                println!(
                    "[{:0>2}:{:0>2}:{:0>2}] - File changed: {}",
                    hour,
                    minute,
                    second,
                    path.display()
                );
                let tools = &self.tools.python_tools;
                // Execute tools sequentially
                for tool in tools {
                    let result = self.run_tool(tool, path).await;
                    self.display_results(result); // Display each result immediately
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn run_tool(&self, tool: &Tool, path: &Path) -> Result<Output> {
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

    // get current time in HH:MM:SS format
    fn get_current_time() -> (u64, u64, u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let seconds = now.as_secs();

        let second = seconds % 60;
        let minute = (seconds % 3600) / 60;
        let hour = (seconds / 3600) % 24;

        return (hour, minute, second);
    }

    fn display_results(&self, results: Result<Output>) {
        match results {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sentinel = Sentinel::new(cli.dir)?;
    futures::executor::block_on(sentinel.watch())?;
    Ok(())
}
