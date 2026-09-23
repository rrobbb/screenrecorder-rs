mod capturer;
mod encoder;
mod writer;

use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;

use anyhow::anyhow;
use crossbeam_channel::bounded;

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
pub const BUF_WRITER_CAPACITY: usize = 1024 * 1024;

pub struct RawFrame { data: RawFrameData, width: u16, height: u16, pub timestamp_ticks: u64 }

impl RawFrame {

    #[inline] pub fn dimensions(&self) -> (u16, u16) { (self.width, self.height) }

    pub fn convert_to_yuv(&self, yuv_buffer: &mut YUVBuffer) {

        yuv_buffer.resize(self.width as usize, self.height as usize);

        match &self.data {

            #[cfg(target_os = "macos")]
            RawFrameData::PixelBuffer(pb) => yuv_buffer.from_bgra(&*pb.data(), pb.bytes_per_row()),

            #[cfg(not(target_os = "macos"))]
            RawFrameData::Owned(data) => yuv_buffer.from_bgra(data, self.width as usize * 4)
        }
    }
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

    #[inline] pub fn dimensions(&self) -> (u16, u16) { (self.width, self.height) }
}

#[derive(Debug, Clone)]
pub struct RecordConfig { pub fps: u32, pub filename: String }

impl Default for RecordConfig {

    fn default() -> Self { RecordConfig { fps: 60, filename: "record".to_string() } }
}

struct AutoStopGuard(Arc<AtomicBool>);

impl Drop for AutoStopGuard {

    fn drop(&mut self) { self.0.store(false, Ordering::SeqCst); }
}

pub fn record_pipeline(config: RecordConfig, running: Arc<AtomicBool>) -> anyhow::Result<()> {

    let (capture_tx, capture_rx) = bounded(2);

    let (encode_tx, encode_rx) = bounded(30);

    let _guard = AutoStopGuard(running.clone());

    let capture_fps = config.fps;
    let running_capture = running.clone();
    let capture_handle = thread::spawn(move || capture_worker(capture_fps, capture_tx, running_capture));

    let encode_fps = config.fps as f32;
    let encode_handle = thread::spawn(move || encode_worker(encode_fps, capture_rx, encode_tx));

    let writer_result = writer_worker(config, encode_rx);

    running.store(false, Ordering::SeqCst);

    let encode_result = encode_handle
        .join()
        .map_err(|_| anyhow!("Il thread di Encoding è andato in panico"))?;

    let capture_result = capture_handle
        .join()
        .map_err(|_| anyhow!("Il thread di Cattura è andato in panico"))?;

    capture_result?;
    encode_result?;
    writer_result?;

    Ok(())
}
