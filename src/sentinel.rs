use console::style;
use notify::event::ModifyKind;
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;
use tokio::time;
use tokio::time::Instant;

use crate::config::Config;
use crate::utils::{display_results, get_current_time};

pub struct Sentinel {
    watcher: Option<notify::RecommendedWatcher>,
    dir: String,
    config: Config,
}

impl Sentinel {
    pub fn new(dir: String, config: Config) -> Result<Self> {
        Ok(Self {
            watcher: None,
            dir: dir.clone(),
            config,
        })
    }

    pub async fn watch(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel::<Result<Event>>();
        let mut watcher = recommended_watcher(tx)?;
        watcher.watch(Path::new(&self.dir), RecursiveMode::Recursive)?;
        self.watcher = Some(watcher);

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
                                && last_time.elapsed() < Duration::from_millis(500)
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

    async fn process_file(&mut self, path: &Path) -> Result<()> {
        let (hour, minute, second) = get_current_time();
        let ext = path.extension().and_then(|e| e.to_str());
        let path = path.to_str().expect("Invalid path");

        if let Some(commands) = self.config.commands.get(ext.unwrap_or("")) {
            println!(
                "[{}] - File changed: {}",
                style(format!("{:0>2}:{:0>2}:{:0>2}", hour, minute, second))
                    .bold()
                    .magenta(),
                style(path).bold().cyan(),
            );
            let commands = commands.clone();
            for command_template in commands {
                let command_str = command_template.replace("{file}", path);
                let mut parts = command_str.split_whitespace();
                if let Some(program) = parts.next() {
                    let args: Vec<&str> = parts.collect();

                    print!(
                        "Running command: {} {}",
                        style(program).bold().cyan(),
                        style(args.join(" ")).bold().yellow(),
                    );

                    // Disable watching to avoid infinite loops if the command modifies the file
                    self.disable_watch()?;

                    let now = Instant::now();
                    let output = Command::new(program).args(args).output()?;
                    let elapsed = now.elapsed();

                    // Start watching again after command has ran
                    self.enable_watch()?;

                    println!(" ✓ ({:.2?})", elapsed);

                    display_results(Ok(output));
                }
            }
            Ok(())
        } else {
            // no commands for this file type
            Ok(())
        }
    }

    // Disable watching to avoid infinite loops
    fn disable_watch(&mut self) -> Result<()> {
        if let Some(watcher) = &mut self.watcher {
            watcher.unwatch(Path::new(&self.dir))?;
        }
        Ok(())
    }

    // Re-enable watching after command has ran
    fn enable_watch(&mut self) -> Result<()> {
        if let Some(watcher) = &mut self.watcher {
            watcher.watch(Path::new(&self.dir), RecursiveMode::Recursive)?;
        }
        Ok(())
    }
}
