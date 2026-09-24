mod yuv;
mod avcc;
mod pipeline;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use pipeline::{record_pipeline, RecordConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    if let Err(error) = can_record() {
        eprintln!("Unable to record: {}", error);
        return Ok(());
    }

    let config = RecordConfig::_1080p30("record".to_string());
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

pub fn can_record() -> Result<(), String> {

    if !scap::is_supported() { return Err("Platform not supported.".to_string()); }

    if !scap::has_permission() {
        println!("Requesting permission...");
        if !scap::request_permission() {
            return Err("Permission denied.".to_string());
        }
    }

    Ok(())
}
