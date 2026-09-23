use std::time::Instant;

use crate::{avcc::AvccConverter, pipeline::{EncodedFrame, RawFrame}, yuv::YUVBuffer};

use anyhow::Context;

use openh264::{OpenH264API, encoder::{BitRate, Encoder, EncoderConfig, FrameRate, RateControlMode, UsageType}};

pub struct OpenH264Encoder {
    encoder: Encoder,
    converter: AvccConverter,
    yuv_buffer: YUVBuffer,
    h264_buffer: Vec<u8>
}

impl OpenH264Encoder {

    pub fn new(fps: f32) -> anyhow::Result<Self> {
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

        let timestamp_ticks = raw_frame.timestamp_ticks;


        let start = Instant::now();

        raw_frame.convert_to_yuv(&mut self.yuv_buffer);

        let yuv_time = start.elapsed();


        // YUV -> BitStream (Annex-B)

        let start = Instant::now();

        let bitstream = self.encoder.encode(&self.yuv_buffer).ok()?;

        let encode_time = start.elapsed();


        // BitStream (Annex-B) -> BitStream (AVCC)

        let start = Instant::now();

        bitstream.write_vec(&mut self.h264_buffer);

        let write_time = start.elapsed();


        // BitStream (AVCC) -> H264

        let start = Instant::now();

        self.converter.convert(&self.h264_buffer);

        let avcc_time = start.elapsed();


        let is_keyframe = matches!(bitstream.frame_type(), openh264::encoder::FrameType::IDR);

        if self.converter.avcc_data.is_empty() { return None }

        println!(
            "YUV: {:>6.2} ms | H264: {:>6.2} ms | write: {:>6.2} ms | AVCC: {:>6.2} ms",
            yuv_time.as_secs_f64() * 1000.0,
            encode_time.as_secs_f64() * 1000.0,
            write_time.as_secs_f64() * 1000.0,
            avcc_time.as_secs_f64() * 1000.0,
        );

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

    Encoder::with_api_config(api, config)
        .context("Unable to initizialize the encoder.")
}
