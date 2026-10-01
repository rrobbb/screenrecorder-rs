#[cfg(target_os = "macos")] use scap::capturer::engine::mac::PixelBuffer;

use scap::capturer::Capturer;

use std::time::Instant;

use super::{yuv::YUVBuffer, VIDEO_TIMESCALE};

pub enum RawFrameData {

    #[cfg(target_os = "macos")] PixelBuffer(PixelBuffer),

    #[cfg(not(target_os = "macos"))] Owned(Vec<u8>)
}

/// A BGRA raw frame
pub struct RawFrame { data: RawFrameData, pub width: usize, pub height: usize, pub timestamp_ticks: u64 }

impl RawFrame {

    #[cfg(target_os = "macos")]
    pub fn get_next(capturer: &mut Capturer, start_time: Instant) -> Option<Self> {

        let pixel_buffer = capturer.raw().get_next_pixel_buffer().ok()?;

        Self::new(pixel_buffer.width(), pixel_buffer.height(), RawFrameData::PixelBuffer(pixel_buffer), start_time)
    }

    #[cfg(not(target_os = "macos"))]
    pub fn get_next(capturer: &mut Capturer, start_time: Instant) -> Option<Self> {

        let frame = capturer.get_next_frame().ok()?;

        Self::new(frame.width, frame.height, RawFrameData::Owned(frame.data), start_time)
    }

    pub fn new(width: usize, height: usize, data: RawFrameData, start_time: Instant) -> Option<Self> {

        // width and height must be even
        let width = width & !1;
        let height = height & !1;

        if width == 0 || height == 0 { return None }

        let valid_data = match data {

            #[cfg(target_os = "macos")]
            RawFrameData::PixelBuffer(pb) => {

                if pb.bytes_per_row() == 0 { return None };

                RawFrameData::PixelBuffer(pb)
            }

            #[cfg(not(target_os = "macos"))]
            RawFrameData::Owned(mut vec) => {

                let expected_bytes = width * height * 4;

                if vec.len() < expected_bytes { return None }

                vec.truncate(expected_bytes);

                RawFrameData::Owned(vec)
            }
        };

        let elapsed = start_time.elapsed();

        let timestamp_ticks = (elapsed.as_nanos() * VIDEO_TIMESCALE as u128 / 1_000_000_000) as u64;

        Some(RawFrame { data: valid_data, width, height, timestamp_ticks })
    }

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

/// A H.264 encoded frame
pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub timestamp_ticks: u64,
    pub is_keyframe: bool,
    pub sps: Option<Vec<u8>>,
    pub pps: Option<Vec<u8>>,
}
