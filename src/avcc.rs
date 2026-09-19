pub struct AvccConverter {
    pub avcc_data: Vec<u8>,
    nal_starts: Vec<(usize, usize)>,
    sps: Option<Vec<u8>>,
    pps: Option<Vec<u8>>,
}

impl AvccConverter {

    pub fn new() -> Self {
        Self {
            avcc_data: Vec::with_capacity(1024 * 256),
            nal_starts: Vec::with_capacity(16),
            sps: None,
            pps: None,
        }
    }

    /// Converte lo stream Annex B in AVCC in-place.
    /// Restituisce `true` se il frame contiene una Keyframe (IDR).
    pub fn convert(&mut self, raw_h264: &[u8]) -> bool {
        // Reset dei vettori senza liberare la memoria allocata
        self.avcc_data.clear();
        self.nal_starts.clear();

        let mut is_keyframe = false;
        let mut i = 0;

        // start code scan
        while i < raw_h264.len() {

            let remainder = &raw_h264[i..];

            if remainder.starts_with(&[0, 0, 0, 1]) {
                self.nal_starts.push((i, 4));
                i += 4;
            } else if remainder.starts_with(&[0, 0, 1]) {
                self.nal_starts.push((i, 3));
                i += 3;
            } else {
                i += 1;
            }
        }

        // 2. Estrazione e formattazione NAL
        for idx in 0..self.nal_starts.len() {
            let (start_pos, header_len) = self.nal_starts[idx];
            let nal_start = start_pos + header_len;
            let nal_end = if idx + 1 < self.nal_starts.len() {
                self.nal_starts[idx + 1].0
            } else {
                raw_h264.len()
            };

            let nal = &raw_h264[nal_start..nal_end];
            if nal.is_empty() { continue; }

            let nal_type = nal[0] & 0x1F;

            match nal_type {
                7 => {
                    if self.sps.is_none() {
                        self.sps = Some(nal.to_vec());
                    }
                    continue; // Omette SPS dal payload del sample MP4
                }
                8 => {
                    if self.pps.is_none() {
                        self.pps = Some(nal.to_vec());
                    }
                    continue; // Omette PPS dal payload del sample MP4
                }
                5 => {
                    is_keyframe = true;
                }
                _ => {}
            }

            // Scrive prefisso a 4 byte (Big-Endian) e payload
            let len = nal.len() as u32;
            self.avcc_data.extend_from_slice(&len.to_be_bytes());
            self.avcc_data.extend_from_slice(nal);
        }

        is_keyframe
    }

    pub fn take_sps_pps(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        match (&self.sps, &self.pps) {
            (Some(s), Some(p)) => Some((s.clone(), p.clone())),
            _ => None,
        }
    }
}
