mod software;
mod macos;

use software::SoftwareEncoder;

use crossbeam_channel::{Sender, Receiver};

use super::{RawFrame, EncodedFrame};

pub fn encode_worker(fps: f32, rx: Receiver<RawFrame>, tx: Sender<EncodedFrame>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let mut encoder = SoftwareEncoder::new(fps)?;

    println!("Using software encoding.");

    while let Ok(raw_frame) = rx.recv() {

        if let Some(encoded_frame) = encoder.encode(raw_frame) {

            if tx.send(encoded_frame).is_err() { break }
        }
    }

    Ok(())
}
