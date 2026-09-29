#[cfg(not(target_os = "macos"))] mod other;
#[cfg(not(target_os = "macos"))] use other::get_next_frame;

#[cfg(target_os = "macos")] mod macos;
#[cfg(target_os = "macos")] use macos::get_next_frame;

use anyhow::Context;
use crossbeam_channel::Sender;

use scap::{capturer::{Capturer, Options, Resolution}, frame::FrameType};

use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::{Duration, Instant};

use super::{RawFrame, RawFrameData, VIDEO_TIMESCALE};

pub fn capture_worker(fps: u32, resolution: Resolution, tx: Sender<RawFrame>, running: Arc<AtomicBool>) -> anyhow::Result<()> {

    let mut capturer = create_capturer(fps, resolution)?;

    capturer.start_capture();

    let start_time = Instant::now();

    println!("Capture worker started.");

    while running.load(Ordering::Relaxed) {

        if let Some(raw_frame) = get_next_frame(&mut capturer, start_time) {

            if tx.send(raw_frame).is_err() { break }

        } else { std::thread::sleep(Duration::from_millis(1)); }
    }

    capturer.stop_capture();

    anyhow::Ok(())
}

fn create_capturer(fps: u32, output_resolution: Resolution) -> anyhow::Result<Capturer> {

    let output_type = FrameType::BGRAFrame;

    let options = Options { fps, show_cursor: true, output_type, output_resolution, ..Default::default() };

    Capturer::build(options).context("Unable to create the capturer.")
}

pub fn get_timestamp_ticks(start_time: Instant) -> u64 {

    let elapsed_secs = start_time.elapsed().as_secs_f64();

    (elapsed_secs * VIDEO_TIMESCALE as f64) as u64
}
