use anyhow::Context;

use openh264::{OpenH264API, encoder::{BitRate, Encoder, EncoderConfig, FrameRate, RateControlMode, UsageType}};

use crate::{avcc::AvccConverter, pipeline::{EncodedFrame, RawFrame}, yuv::YUVBuffer};

pub struct OpenH264Encoder {
    encoder: Encoder,
    converter: AvccConverter,
    yuv_buffer: YUVBuffer,
    h264_buffer: Vec<u8>
}

impl OpenH264Encoder {

    const H264_BUFFER_CAPACITY: usize = 1024 * 1024;

    pub fn new(fps: f32) -> anyhow::Result<Self> {
        Ok(Self {
            encoder: create_encoder(fps)?,
            converter: AvccConverter::new(),
            yuv_buffer: YUVBuffer::default(),
            h264_buffer: Vec::with_capacity(Self::H264_BUFFER_CAPACITY)
        })
    }

    pub fn encode(&mut self, raw_frame: RawFrame) -> Option<EncodedFrame> {

        self.h264_buffer.clear();

        let width = raw_frame.width;
        let height = raw_frame.height;
        let timestamp_ticks = raw_frame.timestamp_ticks;

        if width == 0 || height == 0 { return None }

        raw_frame.convert_to_yuv(&mut self.yuv_buffer);

        let bitstream = self.encoder.encode(&self.yuv_buffer).ok()?;

        bitstream.write_vec(&mut self.h264_buffer);

        self.converter.convert(&self.h264_buffer);

        let is_keyframe = matches!(bitstream.frame_type(), openh264::encoder::FrameType::IDR);

        if self.converter.avcc_data.is_empty() { return None }

        let data = self.converter.take_avcc_data();
        let sps = self.converter.take_sps();
        let pps = self.converter.take_pps();

        Some(EncodedFrame { data, width, height, timestamp_ticks, is_keyframe, sps, pps })
    }
}

fn create_encoder(fps: f32) -> anyhow::Result<Encoder> {

    let config = EncoderConfig::new()
        .usage_type(UsageType::ScreenContentRealTime)
        .max_frame_rate(FrameRate::from_hz(fps))
        .rate_control_mode(RateControlMode::Quality)
        .bitrate(BitRate::from_bps(8_000_000));

    let api = OpenH264API::from_source();

    Encoder::with_api_config(api, config).context("Unable to initizialize the encoder.")
}
