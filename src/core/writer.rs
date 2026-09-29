use anyhow::Context;
use mp4::{Mp4Config, Mp4Sample, Mp4Writer, MediaConfig, TrackConfig, TrackType};

use std::fs::File;
use std::io::{BufWriter};
use std::time::Instant;

use crossbeam_channel::Receiver;

use bytes::Bytes;

use super::{EncodedFrame, RecordConfig, VIDEO_TIMESCALE};

const BUF_WRITER_CAPACITY: usize = 1024 * 1024;

fn create_mp4_writer(filename: &str) -> anyhow::Result<Mp4Writer<BufWriter<File>>> {

    let path = format!("{filename}.mp4");

    let file = File::create(&path)?;

    let writer = BufWriter::with_capacity(BUF_WRITER_CAPACITY, file);

    let mp4_config = Mp4Config {
        major_brand: "isom".parse()?,
        minor_version: 512,
        compatible_brands: vec!["isom".parse()?, "iso2".parse()?, "mp41".parse()?],
        timescale: VIDEO_TIMESCALE
    };

    Mp4Writer::write_start(writer, &mp4_config).context("Unable to initialize MP4 Writer.")
}

pub fn writer_worker(config: RecordConfig, rx: Receiver<EncodedFrame>) -> anyhow::Result<()> {

    let mut mp4_writer = create_mp4_writer(&config.filename)?;

    let mut track_added = false;
    let track_id: u32 = 1;

    let fallback_duration = VIDEO_TIMESCALE / config.fps;
    let mut last_frame_ticks = 0u64;

    let start_time = Instant::now();

    let mut track_config = TrackConfig {
        track_type: TrackType::Video,
        timescale: VIDEO_TIMESCALE,
        language: "und".to_string(),
        media_conf: MediaConfig::AvcConfig(mp4::AvcConfig::default())
    };

    println!("Writer worker started.");

    while let Ok(encoded_frame) = rx.recv() {

        let width = encoded_frame.width as u16;
        let height = encoded_frame.height as u16;
        let current_ticks = encoded_frame.timestamp_ticks;

        if !track_added {

            let (seq_param_set, pic_param_set) = match (encoded_frame.sps, encoded_frame.pps) {

                (Some(sps), Some(pps)) => (sps, pps),

                _ => continue,
            };

            track_config.media_conf = MediaConfig::AvcConfig(mp4::AvcConfig { width, height, seq_param_set, pic_param_set });

            mp4_writer.add_track(&track_config)?;

            track_added = true;
        }

        let duration = if last_frame_ticks > 0 && current_ticks > last_frame_ticks {
            (current_ticks - last_frame_ticks) as u32
        } else {
            fallback_duration
        };

        let sample = Mp4Sample {
            start_time: current_ticks,
            duration,
            rendering_offset: 0,
            is_sync: encoded_frame.is_keyframe,
            bytes: Bytes::from(encoded_frame.data),
        };

        mp4_writer.write_sample(track_id, &sample)?;

        last_frame_ticks = current_ticks;
    }

    mp4_writer.write_end()?;

    println!("\nTotal time: {:.2?}", start_time.elapsed());

    Ok(())
}
