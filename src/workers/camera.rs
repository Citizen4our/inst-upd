use crate::prelude::*;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution};
use roboplc::prelude::*;
use roboplc::rvideo;
use roboplc_derive::WorkerOpts;
use serde::de::StdError;
use std::str::FromStr;
use std::time::Instant;
use tracing::{debug, info};

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
        let dev_idx = variables.dev_idx;
        info!(dev_idx, "Opening camera device");
        let camera = CameraIndex::Index(dev_idx as u32);
        // let mut camera = Camera::new(("/dev/video".to_string() + &dev_idx).as_str())?;
        //@todo add frame rate
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
            Resolution::new(variables.width, variables.height),
            FrameFormat::from_str(std::str::from_utf8(&variables.fourcc).unwrap_or("MJPEG")).unwrap(),
            60,
        )));

        let mut camera = nokhwa::Camera::new(camera, requested)?;

        camera.open_stream()?;
        info!("Camera opened, stream started");

        // @todo add validation for the format
        // for control in camera.formats() {
        //     let format = control.unwrap();
        //
        //     info!("format: {:?}", format);
        //     camera.resolutions(&format.format).iter().for_each(|control| {
        //         info!("resolution: {:?}", control);
        //     });
        // }

        let start_time = Instant::now();
        let mut frame_count = 0;
        let mut total_bytes = 0;

        loop {
            let frame = camera.frame()?;
            frame_count += 1;
            total_bytes += frame.buffer().len();


            let frame_data = frame.buffer().to_vec();
            // let decoded_image = frame.decode_image::<RgbFormat>().unwrap();


            if let Some(ref mut stream) = self.stream {
                stream.send_frame(rvideo::Frame::from(frame_data.clone()))?;
            }

            context.hub().send(WorkerMessage::Frame(frame_data.clone()));

            if frame_count % 120 == 0 {
                let elapsed = start_time.elapsed();
                let mb_processed = total_bytes as f64 / (1024.0 * 1024.0);
                let average_fps = frame_count as f64 / elapsed.as_secs_f64();
                debug!("camera: Average FPS: {:.2}", average_fps);
                debug!("camera: Elapsed: {:.2}", elapsed.as_secs_f64());
                debug!("camera: MB processed: {:.2}", mb_processed);
            }
        }

        info!(dev_idx, "Camera stopped.");
    }
}
