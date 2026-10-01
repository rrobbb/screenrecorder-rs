mod yuv;
mod avcc;

pub mod frame;
pub mod config;

mod capturer;
mod encoder;

use config::RecordConfig;

use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use anyhow::anyhow;
use crossbeam_channel::bounded;

pub const VIDEO_TIMESCALE: u32 = 90_000;

struct AutoStopGuard(Arc<AtomicBool>);

impl Drop for AutoStopGuard {

    fn drop(&mut self) { self.0.store(false, Ordering::SeqCst); }
}

pub fn record_pipeline(config: RecordConfig, running: Arc<AtomicBool>) -> anyhow::Result<()> {

    let (capture_tx, capture_rx) = bounded(2);

    let (encode_tx, encode_rx) = bounded(4);

    let _guard = AutoStopGuard(running.clone());

    let running_capture = running.clone();
    let capture_handle = std::thread::spawn(move || capturer::worker(config.fps, config.resolution, capture_tx, running_capture));

    let encode_handle = std::thread::spawn(move || encoder::worker(config.fps as f32, capture_rx, encode_tx));

    let writer_result = crate::container::mp4::writer_worker(&config.filename, config.fps, encode_rx);

    running.store(false, Ordering::SeqCst);

    let encode_result = encode_handle.join().map_err(|_| anyhow!("Encode thread panicked."))?;

    let capture_result = capture_handle.join().map_err(|_| anyhow!("Capture thread panicked."))?;

    capture_result?;
    encode_result?;
    writer_result?;

    Ok(())
}
