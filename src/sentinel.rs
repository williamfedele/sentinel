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

    pub async fn watch(
        &mut self,
        mut stop_receiver: Option<tokio::sync::mpsc::Receiver<()>>,
    ) -> Result<()> {
        let (tx, rx) = mpsc::channel::<Result<Event>>();
        let mut watcher = recommended_watcher(tx)?;
        watcher.watch(Path::new(&self.dir), RecursiveMode::Recursive)?;
        self.watcher = Some(watcher);

        let (file_tx, mut file_rx) = tokio::sync::mpsc::channel::<PathBuf>(100);

        let _ = tokio::task::spawn(async move {
            let mut last_event: Option<(PathBuf, Instant)> = None;

            while let Ok(res) = rx.recv() {
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
                            let _ = file_tx.send(current_path.clone()).await;
                        }
                        _ => continue,
                    },
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        });

        println!("Watching for changes...");

        if let Some(stop_receiver) = &mut stop_receiver {
            loop {
                tokio::select! {
                    Some(path) = file_rx.recv() => {
                        self.process_file(&path).await?;
                    }
                    _ = stop_receiver.recv() => {
                        println!("Stopping watcher...");
                        break;
                    }
                }
            }
        } else {
            loop {
                if let Some(path) = file_rx.recv().await {
                    self.process_file(&path).await?;
                }
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio::fs;

    use super::*;

    #[tokio::test]
    async fn test_sentinel_new() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir = temp_dir.path().to_string_lossy().to_string();
        let config = Config {
            commands: HashMap::new(),
        };
        let sentinel = Sentinel::new(dir, config)?;
        assert_eq!(sentinel.dir, temp_dir.path().to_string_lossy());
        Ok(())
    }

    #[tokio::test]
    async fn test_sentinel_watch() -> Result<()> {
        // TODO: Create temp file while watching
        Ok(())
    }

    #[tokio::test]
    async fn test_process_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir = temp_dir.path().to_string_lossy().to_string();
        let mut commands = HashMap::new();
        commands.insert("txt".to_string(), vec!["echo {file}".to_string()]);
        let config = Config { commands };
        let mut sentinel = Sentinel::new(dir, config)?;

        let file_path = temp_dir.path().join("test.txt");
        let _ = fs::write(&file_path, "test content").await;

        let result = sentinel.process_file(file_path.as_path()).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_process_file_no_commands() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir = temp_dir.path().to_string_lossy().to_string();
        let config = Config {
            commands: HashMap::new(),
        };
        let mut sentinel = Sentinel::new(dir, config)?;

        let file_path = temp_dir.path().join("test.txt");
        let _ = fs::write(&file_path, "test content").await;

        let result = sentinel.process_file(file_path.as_path()).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_disable_enable_watch() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir = temp_dir.path().to_string_lossy().to_string();
        let config = Config {
            commands: HashMap::new(),
        };
        let mut sentinel = Sentinel::new(dir.clone(), config)?;

        // Initialize watcher
        let (tx, _rx) = mpsc::channel::<Result<Event>>();
        let mut watcher = recommended_watcher(tx)?;
        watcher.watch(Path::new(&dir), RecursiveMode::Recursive)?;
        sentinel.watcher = Some(watcher);

        // Disable watch
        let disable_result = sentinel.disable_watch();
        assert!(disable_result.is_ok());

        // Enable watch
        let enable_result = sentinel.enable_watch();
        assert!(enable_result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_disable_enable_watch_no_watcher() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir = temp_dir.path().to_string_lossy().to_string();
        let config = Config {
            commands: HashMap::new(),
        };
        let mut sentinel = Sentinel::new(dir.clone(), config)?;

        // Disable watch when no watcher is initialized
        let disable_result = sentinel.disable_watch();
        assert!(disable_result.is_ok());

        // Enable watch when no watcher is initialized
        let enable_result = sentinel.enable_watch();
        assert!(enable_result.is_ok());

        Ok(())
    }
}
