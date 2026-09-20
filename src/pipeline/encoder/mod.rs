mod software;

use software::SoftwareEncoder;

use crossbeam_channel::{Sender, Receiver};

use super::{RawFrame, EncodedFrame};

pub fn encode_worker(rx: Receiver<RawFrame>, tx: Sender<EncodedFrame>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let mut encoder = SoftwareEncoder::new()?;

    while let Ok(raw_frame) = rx.recv() {

        if let Some(encoded_frame) = encoder.encode(raw_frame) {

            if tx.send(encoded_frame).is_err() { break }

        }
    }

    Ok(())
}
