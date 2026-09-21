pub struct AvccConverter {
    pub avcc_data: Vec<u8>,
    sps: Option<Vec<u8>>,
    pps: Option<Vec<u8>>,
    pub is_keyframe: bool,
}

impl AvccConverter {

    pub fn new() -> Self {
        Self {
            avcc_data: Vec::with_capacity(256 * 1024),
            sps: None,
            pps: None,
            is_keyframe: false,
        }
    }

    pub fn convert(&mut self, raw_h264: &[u8]) -> bool {

        self.avcc_data.clear();

        self.is_keyframe = false;

        let mut i = 0;
        let len = raw_h264.len();
        let mut last_nal_start: Option<usize> = None;

        while i < len {

            let remainder = &raw_h264[i..];

            let header_len = if remainder.starts_with(&[0, 0, 0, 1]) {
                4
            } else if remainder.starts_with(&[0, 0, 1]) {
                3
            } else {
                i += 1;
                continue;
            };

            if let Some(prev_start) = last_nal_start {
                let nal = &raw_h264[prev_start..i];
                self.process_nal(nal);
            }

            last_nal_start = Some(i + header_len);
            i += header_len;
        }

        if let Some(prev_start) = last_nal_start {
            let nal = &raw_h264[prev_start..len];
            self.process_nal(nal);
        }

        self.is_keyframe
    }

    fn process_nal(&mut self, nal: &[u8]) {

        if nal.is_empty() { return }

        let nal_type = nal[0] & 0x1F;

        match nal_type {
            7 => { self.sps = Some(nal.to_vec()); return }
            8 => { self.pps = Some(nal.to_vec()); return }
            5 => { self.is_keyframe = true; }
            _ => {}
        }

        let nal_len = nal.len() as u32;

        self.avcc_data.extend_from_slice(&nal_len.to_be_bytes());
        self.avcc_data.extend_from_slice(nal);
    }

    pub fn take_avcc_data(&mut self) -> Vec<u8> { std::mem::replace(&mut self.avcc_data, Vec::with_capacity(256 * 1024)) }

    pub fn take_sps(&mut self) -> Option<Vec<u8>> { self.sps.take() }

    pub fn take_pps(&mut self) -> Option<Vec<u8>> { self.pps.take() }
}
