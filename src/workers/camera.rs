use crate::prelude::*;
use roboplc::prelude::*;
use roboplc::rvideo;
use roboplc_derive::WorkerOpts;
use rscam::{Camera, Config};
use serde::de::StdError;
use std::time::Instant;
use tracing::{debug, info};

#[derive(WorkerOpts)]
#[worker_opts(cpu = 3, priority = 80, scheduling = "fifo", blocking = true)]
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
        let config = Config {
            interval: variables.interval,
            resolution: (variables.width as u32, variables.height as u32),
            format: &variables.fourcc,
            nbuffers: variables.buf_size,
            ..Default::default()
        };

        camera.start(&config)?;
        info!(dev_idx, "Camera started.");

        // @todo add validation for the format
        for control in camera.formats() {
            let format = control.unwrap();

            info!("format: {:?}", format);
            camera.resolutions(&format.format).iter().for_each(|control| {
                info!("resolution: {:?}", control);
            });
        }

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
                debug!("Average FPS: {:.2}", average_fps);
                debug!("Elapsed: {:.2}", elapsed.as_secs_f64());
                debug!("MB processed: {:.2}", mb_processed);
            }
        }
    }
}
