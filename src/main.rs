mod utils;
mod yuv;
mod avcc;
mod pipeline;
mod config;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use utils::*;
use config::RecordConfig;
use pipeline::record_pipeline;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    if let Err(error) = can_record() {
        eprintln!("Unable to record: {}", error);
        return Ok(());
    }

    let config = RecordConfig::default();
    let running = Arc::new(AtomicBool::new(true));

    let running_clone = running.clone();

    let handle = thread::spawn(move || record_pipeline(config, running_clone));

    println!("Record started.");

    let mut input = String::new();

    std::io::stdin().read_line(&mut input)?;

    running.store(false, Ordering::Relaxed);

    let _ = handle.join();

    Ok(())
}
