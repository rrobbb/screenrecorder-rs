mod yuv;
mod avcc;
mod core;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    if let Err(error) = can_record() {
        eprintln!("Unable to record: {}", error);
        return Ok(());
    }

    let config = core::RecordConfig::native30("record");

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

    let running_clone = running.clone();

    let handle = std::thread::spawn(move || core::record_pipeline(config, running_clone));

    println!("Record started.");

    let mut input = String::new();

    std::io::stdin().read_line(&mut input)?;

    running.store(false, std::sync::atomic::Ordering::Relaxed);

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
