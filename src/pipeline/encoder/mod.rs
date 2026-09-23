mod software;

use software::OpenH264Encoder;

use crossbeam_channel::{Sender, Receiver};

use super::{RawFrame, EncodedFrame};

pub fn encode_worker(fps: f32, rx: Receiver<RawFrame>, tx: Sender<EncodedFrame>) -> anyhow::Result<()> {

    let mut encoder = OpenH264Encoder::new(fps)?;

    while let Ok(raw_frame) = rx.recv() {

        if let Some(encoded_frame) = encoder.encode(raw_frame) {

            if tx.send(encoded_frame).is_err() { break }
        }
    }

    Ok(())
}
