use scap::capturer::Capturer;
use scap::engine::mac::PixelBuffer;

use std::time::Instant;

use super::{RawFrame, RawFrameData, get_timestamp_ticks};

pub fn get_next_frame(capturer: &mut Capturer, start_time: Instant) -> Option<RawFrame> {

    let pixel_buffer = capturer.raw().get_next_pixel_buffer().ok()?;

    let (width, height) = get_dimensions(&pixel_buffer);

    if width == 0 || height == 0 { return None }

    let expected_bytes = pixel_buffer.bytes_per_row() * height as usize;

    if expected_bytes == 0 { return None }

    let timestamp_ticks = get_timestamp_ticks(start_time);
    let data = RawFrameData::PixelBuffer(pixel_buffer);

    Some(RawFrame { data, width, height, timestamp_ticks })
}

fn get_dimensions(pixel_buffer: &PixelBuffer) -> (u16, u16) {
    ((pixel_buffer.width() as u16) & !1, (pixel_buffer.height() as u16) & !1)
}
