pub struct AvccConverter { pub avcc_data: Vec<u8>, sps: Option<Vec<u8>>, pps: Option<Vec<u8>> }

impl AvccConverter {

    const BUFFER_CAPACITY: usize = 256 * 1024;

    pub fn new() -> Self {
        Self {
            avcc_data: Vec::with_capacity(Self::BUFFER_CAPACITY),
            sps: None,
            pps: None
        }
    }

    pub fn convert(&mut self, raw_h264: &[u8]) {

        self.avcc_data.clear();

        let mut i = 0;
        let len = raw_h264.len();
        let mut last_nal_start = 0;

        while i + 2 < len {

            if raw_h264[i] != 0 || raw_h264[i + 1] != 0 { i += 1; continue }

            if raw_h264[i + 2] == 1 {

                if i > last_nal_start { self.process_nal(&raw_h264[last_nal_start..i]); }

                last_nal_start = i + 3;
                i += 3;
            }

            else if i + 3 < len && raw_h264[i + 2] == 0 && raw_h264[i + 3] == 1 {

                if i > last_nal_start { self.process_nal(&raw_h264[last_nal_start..i]); }

                last_nal_start = i + 4;
                i += 4;
            }

            else { i += 1; }
        }

        if last_nal_start < len { self.process_nal(&raw_h264[last_nal_start..]); }
    }

    #[inline]
    fn process_nal(&mut self, nal: &[u8]) {

        if nal.is_empty() { return }

        let nal_type = nal[0] & 0x1F;

        match nal_type {
            7 => { self.sps = Some(nal.to_vec()); return }
            8 => { self.pps = Some(nal.to_vec()); return }
            _ => {}
        }

        let nal_len = nal.len() as u32;

        self.avcc_data.extend_from_slice(&nal_len.to_be_bytes());
        self.avcc_data.extend_from_slice(nal);
    }

    pub fn take_avcc_data(&mut self) -> Vec<u8> { std::mem::replace(&mut self.avcc_data, Vec::with_capacity(Self::BUFFER_CAPACITY)) }

    pub fn take_sps(&mut self) -> Option<Vec<u8>> { self.sps.take() }

    pub fn take_pps(&mut self) -> Option<Vec<u8>> { self.pps.take() }
}
