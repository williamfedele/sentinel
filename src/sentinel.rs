use console::style;
use notify::event::ModifyKind;
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tokio::time;
use tokio::time::Instant;

use crate::config::ToolConfig;
use crate::utils::{display_results, get_current_time};

pub struct Sentinel {
    dir: String,
    tools: ToolConfig,
}

impl Sentinel {
    pub fn new(dir: String) -> Result<Self> {
        Ok(Self {
            dir,
            tools: ToolConfig::new(),
        })
    }

    pub async fn watch(&self) -> Result<()> {
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
        let (hour, minute, second) = get_current_time();
        let ext = path.extension().and_then(|e| e.to_str());

        if matches!(ext, Some("py")) {
            println!(
                "[{}] - File changed: {}",
                style(format!("{:0>2}:{:0>2}:{:0>2}", hour, minute, second))
                    .bold()
                    .magenta(),
                path.display()
            );

            match ext {
                Some("py") => {
                    // time and file that changed
                    let tools = &self.tools.python_tools;
                    // Execute tools sequentially
                    for tool in tools {
                        let result = tool.run(path).await;
                        display_results(result); // Display each result immediately
                    }
                    Ok(())
                }
                _ => Ok(()), // Handle other extensions here, or do nothing
            }
        } else {
            Ok(())
        }
    }
}
