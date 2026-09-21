pub const VIDEO_TIMESCALE: u32 = 90_000;

pub const BUF_WRITER_CAPACITY: usize = 1024 * 1024;

pub struct RecordConfig { pub fps: u32, pub filename: String }

impl Default for RecordConfig {

    fn default() -> Self { RecordConfig { fps: 30, filename: "record".to_string() } }
}
