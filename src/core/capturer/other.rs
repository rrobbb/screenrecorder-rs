use scap::{capturer::Capturer, frame::Frame};

use std::time::Instant;

use super::{RawFrame, RawFrameData, get_timestamp_ticks};

pub fn get_next_frame(capturer: &mut Capturer, start_time: Instant) -> Option<RawFrame> {

    let Ok(Frame::BGRA(mut frame)) = capturer.get_next_frame() else { return None };

    let width = (frame.width as usize) & !1;
    let height = (frame.height as usize) & !1;

    if width == 0 || height == 0 { return None }

    let expected_bytes = width * height * 4;

    if frame.data.len() < expected_bytes { return None }

    frame.data.truncate(expected_bytes);

    let timestamp_ticks = get_timestamp_ticks(start_time);

    Some(RawFrame { data: RawFrameData::Owned(frame.data), width, height, timestamp_ticks })
}
