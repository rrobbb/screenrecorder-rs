use yuvutils_rs::{bgra_to_yuv420, YuvRange, YuvStandardMatrix};
use openh264::formats::YUVSource;

/// Needed to convert a BGRA frame into a YUV frame
/// H264 encoding needs a YUV frame

pub struct YUVBuffer {
    y: Vec<u8>,
    u: Vec<u8>,
    v: Vec<u8>,
    width: usize,
    height: usize,
}

impl YUVBuffer {

    pub fn new(width: usize, height: usize) -> Self {

        let y_len = width * height;

        let uv_len = (width / 2) * (height / 2);

        Self { y: vec![0; y_len], u: vec![0; uv_len], v: vec![0; uv_len], width, height }
    }

    /// Converts BGRA to YUV420p using CPU SIMD without allocating new memory
    pub fn from_bgra(&mut self, bgra: &[u8], bgra_stride: usize) {
        bgra_to_yuv420(
            &mut self.y,
            self.width as u32,
            &mut self.u,
            (self.width / 2) as u32,
            &mut self.v,
            (self.width / 2) as u32,
            bgra,
            bgra_stride as u32,
            self.width as u32,
            self.height as u32,
            YuvRange::Full,
            YuvStandardMatrix::Bt601,
        );
    }
}

impl YUVSource for YUVBuffer {

    fn dimensions(&self) -> (usize, usize) { (self.width, self.height) }

    fn y(&self) -> &[u8] { &self.y }

    fn u(&self) -> &[u8] { &self.u }

    fn v(&self) -> &[u8] { &self.v }

    fn strides(&self) -> (usize, usize, usize) { (self.width, self.width / 2, self.width / 2) }
}
