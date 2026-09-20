mod capturer;
mod encoder;
mod writer;

use std::thread;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crossbeam_channel::bounded;

use crate::config::RecordConfig;

use capturer::capture_worker;
use encoder::encode_worker;
use writer::writer_worker;

#[cfg(target_os = "macos")]
use scap::capturer::engine::mac::PixelBuffer;

pub enum RawFrameData {

    #[cfg(target_os = "macos")]
    PixelBuffer(PixelBuffer),

    #[cfg(not(target_os = "macos"))]
    Owned(Vec<u8>),
}


pub struct RawFrame { pub data: RawFrameData, pub width: u16, pub height: u16, pub timestamp_ticks: u64 }

impl RawFrame {

    pub fn dimensions(&self) -> (u16, u16) { (self.width, self.height) }
}

pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub timestamp_ticks: u64,
    pub is_keyframe: bool,
    pub sps: Option<Vec<u8>>,
    pub pps: Option<Vec<u8>>,
}

impl EncodedFrame {

    pub fn dimensions(&self) -> (u16, u16) { (self.width, self.height) }
}

pub fn record_pipeline(config: RecordConfig, running: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let (capture_tx, capture_rx) = bounded(60);
    let (encode_tx, encode_rx) = bounded(60);

    let running_capture = running.clone();

    let capture_handle = thread::spawn(move || capture_worker(config.fps, capture_tx, running_capture));
    let encode_handle = thread::spawn(move || encode_worker(capture_rx, encode_tx));

    let writer_result = writer_worker(config, encode_rx);

    let encode_result = match encode_handle.join() {

        Ok(result) => result,

        Err(_) => return Err("Encode thread panicked.".into())
    };

    if capture_handle.join().is_err() { return Err("Capture thread panicked.".into()); }

    encode_result?;
    writer_result?;

    Ok(())
}
