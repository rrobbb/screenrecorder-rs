use anyhow::Context;
use crossbeam_channel::Sender;

use scap::{capturer::{Capturer, Options, Resolution}, frame::FrameType};

use std::{time::{Duration, Instant}, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use super::frame::RawFrame;

pub fn worker(fps: u32, resolution: Resolution, tx: Sender<RawFrame>, running: Arc<AtomicBool>) -> anyhow::Result<()> {

    let mut capturer = create_capturer(fps, resolution)?;

    capturer.start_capture();

    let start_time = Instant::now();

    while running.load(Ordering::Relaxed) {

        match RawFrame::get_next(&mut capturer, start_time) {

            Some(raw) => if tx.send(raw).is_err() { break }

            None => std::thread::sleep(Duration::from_millis(1))
        }
    }

    capturer.stop_capture();

    anyhow::Ok(())
}

fn create_capturer(fps: u32, output_resolution: Resolution) -> anyhow::Result<Capturer> {

    let output_type = FrameType::BGRAFrame;

    let options = Options { fps, show_cursor: true, output_type, output_resolution, ..Default::default() };

    Capturer::build(options).context("Unable to create the capturer.")
}
