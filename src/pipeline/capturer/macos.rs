use scap::{capturer::Capturer, engine::mac::PixelBuffer};

use apple_cf::cv::CVPixelBuffer;

use screencapturekit_sys::cm_sample_buffer_ref::CMSampleBufferGetImageBuffer;

use std::time::Instant;

use super::{RawFrame, RawFrameData, get_timestamp_ticks};

pub fn get_next_frame(capturer: &mut Capturer, start_time: Instant) -> Option<RawFrame> {

    let pixel_buffer = capturer.raw().get_next_pixel_buffer().ok()?;

    let (width, height) = get_dimensions(&pixel_buffer);

    if width == 0 || height == 0 { return None }

    let expected_bytes = pixel_buffer.bytes_per_row() * height as usize;

    if expected_bytes == 0 { return None }

    let timestamp_ticks = get_timestamp_ticks(start_time);

    // let data = to_cv_pixel_buffer(&pixel_buffer)?;

    Some(RawFrame { data: RawFrameData::PixelBuffer(pixel_buffer), width, height, timestamp_ticks })
}

fn get_dimensions(pixel_buffer: &PixelBuffer) -> (u16, u16) {
    ((pixel_buffer.width() as u16) & !1, (pixel_buffer.height() as u16) & !1)
}

fn _to_cv_pixel_buffer(pixel_buffer: &PixelBuffer) -> Option<CVPixelBuffer> {

    unsafe {

        let sample_buffer = pixel_buffer.buffer();

        let buffer_ref = &(*sample_buffer.sys_ref);

        let pixel_buffer_ref = CMSampleBufferGetImageBuffer(buffer_ref);

        CVPixelBuffer::from_raw_borrowed(pixel_buffer_ref.cast())
    }
}
