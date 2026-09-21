use yuvutils_rs::{bgra_to_yuv420, YuvRange, YuvStandardMatrix};
use openh264::formats::YUVSource;

/// Converts a BGRA frame into a YUV frame

pub struct YUVBuffer { y: Vec<u8>, u: Vec<u8>, v: Vec<u8>, width: usize, height: usize }

impl YUVBuffer {

    pub fn new(width: usize, height: usize) -> Self {

        let (y_size, uv_size) = get_y_and_uv_size(width, height);

        Self { y: vec![0; y_size], u: vec![0; uv_size], v: vec![0; uv_size], width, height }
    }

    /// Converts BGRA to YUV420p using CPU SIMD without allocating new memory
    pub fn from_bgra(&mut self, bgra: &[u8], bgra_stride: usize) {

        if self.width == 0 || self.height == 0 { return }

        let matrix = if self.height >= 720 { YuvStandardMatrix::Bt709 } else { YuvStandardMatrix::Bt601 };

        let (y_stride, u_stride, v_stride) = self.strides();

        bgra_to_yuv420(
            &mut self.y, y_stride as u32,
            &mut self.u, u_stride as u32,
            &mut self.v, v_stride as u32,
            bgra, bgra_stride as u32,
            self.width as u32, self.height as u32,
            YuvRange::TV, matrix
        );
    }

    pub fn resize(&mut self, width: usize, height: usize) {

        if self.width == width && self.height == height { return }

        self.width = width;
        self.height = height;

        let (y_size, uv_size) = get_y_and_uv_size(width, height);

        self.y.resize(y_size, 0);
        self.u.resize(uv_size, 0);
        self.v.resize(uv_size, 0);
    }
}

impl YUVSource for YUVBuffer {

    fn dimensions(&self) -> (usize, usize) { (self.width, self.height) }

    fn y(&self) -> &[u8] { &self.y }

    fn u(&self) -> &[u8] { &self.u }

    fn v(&self) -> &[u8] { &self.v }

    fn strides(&self) -> (usize, usize, usize) { (self.width, self.width / 2, self.width / 2) }
}

fn get_y_and_uv_size(width: usize, height: usize) -> (usize, usize) {

    let y_size = width * height;
    let uv_size = y_size / 4;

    (y_size, uv_size)
}
