use scap::capturer::Capturer;

use std::time::Instant;

use super::{RawFrame, RawFrameData, get_timestamp_ticks};

pub fn get_next_frame(capturer: &mut Capturer, start_time: Instant) -> Option<RawFrame> {

    let pixel_buffer = capturer.raw().get_next_pixel_buffer().ok()?;

    let width = pixel_buffer.width() & !1;
    let height = pixel_buffer.height() & !1;

    if width == 0 || height == 0 { return None }

    let expected_bytes = pixel_buffer.bytes_per_row() * height;

    if expected_bytes == 0 { return None }

    let timestamp_ticks = get_timestamp_ticks(start_time);

    Some(RawFrame { data: RawFrameData::PixelBuffer(pixel_buffer), width, height, timestamp_ticks })
}
