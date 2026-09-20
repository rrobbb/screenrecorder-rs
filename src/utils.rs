pub fn can_record() -> Result<(), String> {

    if !scap::is_supported() { return Err("Platform not supported.".to_string()); }

    if !scap::has_permission() {
        println!("Requesting permission...");
        if !scap::request_permission() {
            return Err("Permission denied.".to_string());
        }
    }

    Ok(())
}

pub fn _detect_h264_format(data: &[u8]) -> &'static str {

    if data.len() >= 4 && data[0..4] == [0, 0, 0, 1] { return "Annex B (4-byte start code)" }

    if data.len() >= 3 && data[0..3] == [0, 0, 1] { return "Annex B (3-byte start code)"; }

    if data.len() >= 4 {

        let nal_length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

        if nal_length > 0 && nal_length <= data.len() - 4 { return "AVCC (4-byte length prefix)" }
    }

    "Unknown format."
}
