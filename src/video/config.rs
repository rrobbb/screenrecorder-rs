use scap::capturer::Resolution;

macro_rules! record_config {
    ($name:ident, $fps:expr, $resolution:expr) => {
        pub fn $name(filename: &str) -> Self {
            Self { fps: $fps, resolution: $resolution, filename: filename.to_string() }
        }
    };
}

#[derive(Debug, Clone)]
pub struct RecordConfig { pub fps: u32, pub resolution: Resolution, pub filename: String }

impl RecordConfig {

    record_config!(_720p30, 30, Resolution::_720p);
    record_config!(_720p60, 60, Resolution::_720p);

    record_config!(_1080p30, 30, Resolution::_1080p);
    record_config!(_1080p60, 60, Resolution::_1080p);

    record_config!(native30, 30, Resolution::Captured);
    record_config!(native60, 60, Resolution::Captured);
}
