use notify::Result;
use std::process::Output;
use std::time::{SystemTime, UNIX_EPOCH};

// get current time in HH:MM:SS format
pub fn get_current_time() -> (u64, u64, u64) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let seconds = now.as_secs();

    let second = seconds % 60;
    let minute = (seconds % 3600) / 60;
    let hour = (seconds / 3600) % 24;

    return (hour, minute, second);
}

pub fn display_results(results: Result<Output>) {
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
