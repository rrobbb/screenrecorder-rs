mod utils;
mod yuv;
mod avcc;

use yuv::YUVBuffer;

use mp4::{Mp4Config, Mp4Sample, Mp4Writer, TrackConfig, TrackType};
use openh264::encoder::Encoder;
use scap::{
    capturer::{Capturer, Options, Resolution},
    frame::Frame,
};

use std::thread;
use std::fs::File;
use std::io::BufWriter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::Bytes;
use utils::*;

use crate::avcc::AvccConverter;


const VIDEO_TIMESCALE: u32 = 90_000;
const BUF_WRITER_CAPACITY: usize = 1024 * 1024; // 1 MB


pub struct RecordConfig {
    pub fps: u32,
    pub filename: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    if let Err(error) = can_record() {
        eprintln!("Unable to record: {}", error);
        return Ok(());
    }

    let config = RecordConfig {
        fps: 30,
        filename: "record".to_string(),
    };

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Il thread riceve solo la configurazione e il flag atomico
    let handle = thread::spawn(move || record_worker(config, running_clone));

    println!("Registrazione avviata. Premere Invio per fermare...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    running.store(false, Ordering::Relaxed);
    let _ = handle.join();

    println!("Registrazione completata.");
    Ok(())
}


fn create_capturer(fps: u32) -> Capturer {

    let options = Options {
        fps,
        show_cursor: true,
        output_type: scap::frame::FrameType::BGRAFrame,
        output_resolution: Resolution::Captured,
        ..Default::default()
    };

    Capturer::build(options).expect("Unable to create the capturer.")
}

fn create_mp4_writer(filename: &str) -> Mp4Writer<BufWriter<File>>{

    let path = format!("{}.mp4", filename);

    let file = File::create(&path).expect("Unable to create the file.");

    let writer = BufWriter::with_capacity(BUF_WRITER_CAPACITY, file);

    let mp4_config = Mp4Config {
        major_brand: str::parse("isom").unwrap(),
        minor_version: 512,
        compatible_brands: vec![str::parse("isom").unwrap(), str::parse("iso2").unwrap(), str::parse("mp41").unwrap()],
        timescale: 90000
    };

    Mp4Writer::write_start(writer, &mp4_config).expect("Unable to initialize the MP4 Writer.")
}

fn record_worker(config: RecordConfig, running: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let mut capturer = create_capturer(config.fps);
    let mut encoder = Encoder::new()?;
    let mut mp4_writer = create_mp4_writer(&config.filename);
    let mut converter = AvccConverter::new();

    // let mut yuv_buffer = YUVBuffer::new(config.width as usize, config.height as usize);
    let mut yuv_buffer: Option<YUVBuffer> = None;

    capturer.start_capture();

    let mut frame_count = 0;
    let frame_duration_ticks = 90000 / config.fps;
    let mut track_added = false;
    let track_id: u32 = 1;

    let fallback_duration = VIDEO_TIMESCALE / config.fps;
    let start_time = Instant::now();
    let mut last_frame_ticks: u64 = 0;

    // 2. Ciclo di acquisizione
    while running.load(Ordering::Relaxed) {

        if let Ok(Frame::BGRA(frame)) = capturer.get_next_frame() {

            let elapsed_secs = start_time.elapsed().as_secs_f64();
            let current_ticks = (elapsed_secs * VIDEO_TIMESCALE as f64) as u64;

            let duration = if last_frame_ticks > 0 && current_ticks > last_frame_ticks {
                (current_ticks - last_frame_ticks) as u32
            } else {
                fallback_duration
            };

            let width = frame.width as u16;
            let height = frame.height as u16;

            let buffer = yuv_buffer.get_or_insert_with(|| { YUVBuffer::new(width as usize, height as usize) });

            buffer.from_bgra(&frame.data);

            let bitstream = encoder.encode(buffer)?;
            let raw_h264 = bitstream.to_vec();

            let is_keyframe = converter.convert(&raw_h264);
            if converter.avcc_data.is_empty() { continue; }

            if !track_added {

                let Some((sps, pps)) = converter.take_sps_pps() else { continue };

                let track_config = TrackConfig {
                    track_type: TrackType::Video,
                    timescale: VIDEO_TIMESCALE,
                    language: "und".to_string(),
                    media_conf: mp4::MediaConfig::AvcConfig(mp4::AvcConfig {
                        width: width,
                        height: height,
                        seq_param_set: sps,
                        pic_param_set: pps,
                    }),
                };

                mp4_writer.add_track(&track_config)?;
                track_added = true;
            }

            let sample = Mp4Sample {
                start_time: (frame_count * frame_duration_ticks) as u64,
                duration,
                rendering_offset: 0,
                is_sync: is_keyframe,
                bytes: Bytes::copy_from_slice(&converter.avcc_data),
            };

            mp4_writer.write_sample(track_id, &sample)?;
            last_frame_ticks = current_ticks;

            frame_count += 1;
            print!("\rFrame acquisiti e codificati: {}", frame_count);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        } else {
            thread::sleep(Duration::from_millis(1));
        }
    }

    capturer.stop_capture();
    mp4_writer.write_end()?;

    println!("\nTempo totale: {:.2?}", start_time.elapsed());
    Ok(())
}
