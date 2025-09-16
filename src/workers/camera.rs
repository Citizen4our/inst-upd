use crate::prelude::*;
use roboplc::prelude::*;
use roboplc::rvideo;
use roboplc_derive::WorkerOpts;
use rscam::{Camera, Config};
use serde::de::StdError;
use std::time::Instant;
use tracing::info;

#[derive(WorkerOpts)]
#[worker_opts(cpu = 3, priority = 90, scheduling = "fifo", blocking = true)]
pub struct DetectorVideo {
    stream: Option<rvideo::Stream>,
}

impl DetectorVideo {
    pub fn new_with_rvideo(stream: rvideo::Stream) -> Self { Self { stream: Some(stream) } }

    pub fn new() -> Self { Self { stream: None } }
}
impl Worker<WorkerMessage, Variables> for DetectorVideo {
    fn run(&mut self, context: &Context<WorkerMessage, Variables>) -> Result<(), Box<(dyn StdError + Send + Sync + 'static)>> {
        let variables = &context.variables().camera_config;
        let dev_idx = variables.dev_idx.to_string();
        info!(dev_idx, "Opening camera device");
        let mut camera = Camera::new(("/dev/video".to_string() + &dev_idx).as_str())?;

        // First, check available formats and validate the requested format
        let mut available_formats = Vec::new();
        let mut requested_format_supported = false;

        // Convert 5-byte format to 4-byte format for comparison
        let requested_format_4byte: [u8; 4] = [
            variables.fourcc[0],
            variables.fourcc[1],
            variables.fourcc[2],
            variables.fourcc[3],
        ];

        info!(dev_idx, "Checking available camera formats");
        for control in camera.formats() {
            let format = control.unwrap();
            available_formats.push(format.format.clone());
            info!("Available format: {:?}", format.format);

            // Check if our requested format is supported
            if format.format == requested_format_4byte {
                requested_format_supported = true;
                info!("Requested format {:?} is supported", requested_format_4byte);
            }

            camera.resolutions(&format.format).iter().for_each(|control| {
                info!("Available resolution for {:?}: {:?}", format.format, control);
            });
        }

        // If requested format is not supported, try to find a suitable alternative
        let format_to_use = if requested_format_supported {
            requested_format_4byte
        } else {
            // Try common formats in order of preference
            let fallback_formats = [
                [b'M', b'J', b'P', b'G'], // MJPEG
                [b'Y', b'U', b'Y', b'V'], // YUYV
                [b'R', b'G', b'B', b'3'], // RGB3
                [b'B', b'G', b'R', b'3'], // BGR3
            ];
            let mut selected_format = None;

            for fallback in &fallback_formats {
                if available_formats.contains(fallback) {
                    selected_format = Some(*fallback);
                    info!("Using fallback format: {:?}", fallback);
                    break;
                }
            }

            match selected_format {
                Some(format) => format,
                None => {
                    return Err(format!(
                        "No supported format found. Available formats: {:?}, Requested: {:?}",
                        available_formats, requested_format_4byte
                    )
                    .into());
                }
            }
        };

        let config = Config {
            interval: variables.interval,
            resolution: (variables.width, variables.height),
            format: &format_to_use,
            nbuffers: variables.buf_size,
            ..Default::default()
        };

        info!(dev_idx, "Starting camera with format: {:?}", format_to_use);
        camera.start(&config)?;
        info!(dev_idx, "Camera started successfully.");

        let start_time = Instant::now();
        let mut frame_count = 0;
        let mut total_bytes = 0;

        loop {
            let frame = camera.capture()?;
            frame_count += 1;
            total_bytes += frame.len();

            let frame_data = frame.to_vec();

            if let Some(ref mut stream) = self.stream {
                stream.send_frame(rvideo::Frame::from(frame_data.clone()))?;
            }

            context.hub().send(WorkerMessage::Frame(frame_data.clone()));

            if frame_count % 120 == 0 {
                let elapsed = start_time.elapsed();
                let mb_processed = total_bytes as f64 / (1024.0 * 1024.0);
                let average_fps = frame_count as f64 / elapsed.as_secs_f64();
                info!("camera: Average FPS: {:.2}", average_fps);
                info!("camera: Elapsed: {:.2}", elapsed.as_secs_f64());
                info!("camera: MB processed: {:.2}", mb_processed);
            }
        }
    }
}
