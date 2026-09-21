use apple_cf::{cm::{CMTime, CMSampleBuffer}, cv::CVPixelBuffer};

use videotoolbox::{compression::CompressionSession, Codec};

pub struct VideoToolboxEncoder { session: CompressionSession, width: u16, height: u16, fps: u32 }

impl VideoToolboxEncoder {

    pub fn new(width: u16, height: u16, fps: u32, bitrate: i32) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {

        let session = CompressionSession::builder(width as i32, height as i32, Codec::H264)
            .with_real_time(true)
            .with_allow_frame_reordering(false)
            .with_expected_frame_rate(fps as f64)
            .with_average_bit_rate(bitrate)
            .build()?;

        Ok(Self { session, width, height, fps })
    }

    pub async fn encode(&self, pixel_buffer: CVPixelBuffer, presentation_time: CMTime) -> Result<CMSampleBuffer, Box<dyn std::error::Error + Send + Sync>> {

        let duration = CMTime::new(1, self.fps as i32);

        let sample_buffer = self.session.encode_frame_async(pixel_buffer, presentation_time, duration, None).await?;

        Ok(sample_buffer)
    }
}
