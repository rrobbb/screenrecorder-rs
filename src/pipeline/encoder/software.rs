use crate::{avcc::AvccConverter, pipeline::{EncodedFrame, RawFrame, RawFrameData}, yuv::YUVBuffer};
use openh264::encoder::Encoder;

pub struct SoftwareEncoder {
    encoder: Encoder,
    converter: AvccConverter,
    yuv_buffer: Option<YUVBuffer>,
    h264_buffer: Vec<u8>,
    dimensions: (u16, u16),
}

impl SoftwareEncoder {

    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            encoder: Encoder::new()?,
            converter: AvccConverter::new(),
            yuv_buffer: None,
            h264_buffer: Vec::with_capacity(1024 * 1024),
            dimensions: (0, 0),
        })
    }

    pub fn encode(&mut self, raw_frame: RawFrame) -> Option<EncodedFrame> {

        self.h264_buffer.clear();

        let (width, height) = raw_frame.dimensions();
        let current_ticks = raw_frame.timestamp_ticks;

        if self.yuv_buffer.is_none() || self.dimensions != (width, height) {
            self.yuv_buffer = Some(YUVBuffer::new(width as usize, height as usize));
            self.dimensions = (width, height);
        }

        let yuv_buffer = self.yuv_buffer.as_mut().unwrap();

        let (pixel_data, stride) = match &raw_frame.data {
            #[cfg(target_os = "macos")]
            RawFrameData::PixelBuffer(pb) => (pb.data(), pb.bytes_per_row()),
            #[cfg(not(target_os = "macos"))]
            RawFrameData::Owned(data) => (data.as_slice(), width as usize * 4),
        };

        yuv_buffer.from_bgra(&pixel_data, stride);

        let bitstream = self.encoder.encode(yuv_buffer).ok()?;
        bitstream.write_vec(&mut self.h264_buffer);

        let is_keyframe = matches!(bitstream.frame_type(), openh264::encoder::FrameType::IDR);

        self.converter.convert(&self.h264_buffer);

        if self.converter.avcc_data.is_empty() { return None }

        Some(EncodedFrame {
            data: self.converter.take_avcc_data(),
            width,
            height,
            timestamp_ticks: current_ticks,
            is_keyframe,
            sps: self.converter.take_sps(),
            pps: self.converter.take_pps(),
        })
    }
}
