mod capturer;
mod encoder;
mod writer;

use std::{thread, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use anyhow::anyhow;
use crossbeam_channel::bounded;
use scap::capturer::Resolution;

use capturer::capture_worker;
use encoder::encode_worker;
use writer::writer_worker;

#[cfg(target_os = "macos")]
use scap::capturer::engine::mac::PixelBuffer;

use crate::yuv::YUVBuffer;

pub enum RawFrameData {

    #[cfg(target_os = "macos")] PixelBuffer(PixelBuffer),

    #[cfg(not(target_os = "macos"))] Owned(Vec<u8>),
}

pub const VIDEO_TIMESCALE: u32 = 90_000;

pub struct RawFrame { data: RawFrameData, pub width: usize, pub height: usize, pub timestamp_ticks: u64 }

impl RawFrame {

    pub fn convert_to_yuv(&self, yuv_buffer: &mut YUVBuffer) {

        yuv_buffer.resize(self.width, self.height);

        match &self.data {

            #[cfg(target_os = "macos")]
            RawFrameData::PixelBuffer(pb) => yuv_buffer.from_bgra(&*pb.data(), pb.bytes_per_row()),

            #[cfg(not(target_os = "macos"))]
            RawFrameData::Owned(data) => yuv_buffer.from_bgra(data, self.width * 4)
        }
    }
}

pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub timestamp_ticks: u64,
    pub is_keyframe: bool,
    pub sps: Option<Vec<u8>>,
    pub pps: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct RecordConfig { pub fps: u32, pub resolution: Resolution, pub filename: String }

impl RecordConfig {

    pub fn _720p30(filename: String) -> Self { Self { fps: 30, resolution: Resolution::_720p, filename } }

    pub fn _720p60(filename: String) -> Self { Self { fps: 60, resolution: Resolution::_720p, filename } }

    pub fn _1080p30(filename: String) -> Self { Self { fps: 30, resolution: Resolution::_1080p, filename } }

    pub fn _1080p60(filename: String) -> Self { Self { fps: 60, resolution: Resolution::_1080p, filename } }
}

struct AutoStopGuard(Arc<AtomicBool>);

impl Drop for AutoStopGuard {

    fn drop(&mut self) { self.0.store(false, Ordering::SeqCst); }
}

pub fn record_pipeline(config: RecordConfig, running: Arc<AtomicBool>) -> anyhow::Result<()> {

    let (capture_tx, capture_rx) = bounded(2);

    let (encode_tx, encode_rx) = bounded(30);

    let _guard = AutoStopGuard(running.clone());

    let running_capture = running.clone();
    let capture_handle = thread::spawn(move || capture_worker(config.fps, config.resolution, capture_tx, running_capture));

    let encode_handle = thread::spawn(move || encode_worker(config.fps as f32, capture_rx, encode_tx));

    let writer_result = writer_worker(config, encode_rx);

    running.store(false, Ordering::SeqCst);

    let encode_result = encode_handle
        .join()
        .map_err(|_| anyhow!("Encode thread panicked."))?;

    let capture_result = capture_handle
        .join()
        .map_err(|_| anyhow!("Capture thread panicked."))?;

    capture_result?;
    encode_result?;
    writer_result?;

    Ok(())
}
