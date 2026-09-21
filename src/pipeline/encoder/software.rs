use crate::{avcc::AvccConverter, pipeline::{EncodedFrame, RawFrame, RawFrameData}, yuv::YUVBuffer};

use openh264::{OpenH264API, encoder::{BitRate, Encoder, EncoderConfig, FrameRate, RateControlMode, UsageType}, formats::YUVSource};

/// BGRA (Raw Frame) -> YUV -> BitStream (Annex-B) -> BitStream (AVCC) -> H264 (Encoded Frame)
pub struct SoftwareEncoder {
    encoder: Encoder,
    converter: AvccConverter,
    yuv_buffer: YUVBuffer,
    h264_buffer: Vec<u8>
}

impl SoftwareEncoder {

    pub fn new(fps: f32) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            encoder: create_encoder(fps)?,
            converter: AvccConverter::new(),
            yuv_buffer: YUVBuffer::new(0, 0),
            h264_buffer: Vec::with_capacity(1024 * 1024)
        })
    }

    pub fn encode(&mut self, raw_frame: RawFrame) -> Option<EncodedFrame> {

        self.h264_buffer.clear();

        let (width, height) = raw_frame.dimensions();

        if width == 0 || height == 0 { return None }

        let current_ticks = raw_frame.timestamp_ticks;

        if self.yuv_buffer.dimensions() != (width as usize, height as usize) {
            self.yuv_buffer.resize(width as usize, height as usize);
        }

        let (pixel_data, stride) = match &raw_frame.data {

            #[cfg(target_os = "macos")]
            RawFrameData::PixelBuffer(pb) => (pb.data(), pb.bytes_per_row()),

            #[cfg(not(target_os = "macos"))]
            RawFrameData::Owned(data) => (data.as_slice(), width as usize * 4),
        };

        self.yuv_buffer.from_bgra(&pixel_data, stride);

        let bitstream = self.encoder.encode(&self.yuv_buffer).ok()?;
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

fn create_encoder(fps: f32) -> Result<Encoder, Box<dyn std::error::Error + Send + Sync>> {

    let config = EncoderConfig::new()
        .usage_type(UsageType::ScreenContentRealTime)
        .max_frame_rate(FrameRate::from_hz(fps))
        .rate_control_mode(RateControlMode::Quality)
        .bitrate(BitRate::from_bps(2_500_000));

    let api = OpenH264API::from_source();

    let encoder = Encoder::with_api_config(api, config)?;

    Ok(encoder)
}
