use inst_upd::prelude::*;
use inst_upd::workers::*;
use roboplc::controller::*;
use roboplc::rvideo;
use std::time::Duration;

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);


fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    roboplc::setup_panic();
    roboplc::configure_logger(
        dotenv::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string())
            .parse()
            .unwrap(),
    );

    let variables = init_config_by_env(dotenv::vars().collect());

    // @todo move to the worker as function
    let rvideo_stream = rvideo::add_stream(
        rvideo::Format::MJpeg,
        variables.camera_config.width,
        variables.camera_config.height,
    )?;

    let detector_video = DetectorVideo::new_with_rvideo(rvideo_stream);

    let mut controller: Controller<WorkerMessage, Variables> = Controller::new_with_variables(variables);

    controller.spawn_worker(RvideoSrv {})?;
    controller.spawn_worker(detector_video)?;
    controller.spawn_worker(BotWorker {})?;
    controller.spawn_worker(WebSocketWorker {})?;
    // register SIGINT and SIGTERM signals with max shutdown timeout
    controller.register_signals(SHUTDOWN_TIMEOUT)?;
    // blocks the main thread while the controller is online and the workers are running
    controller.block();
    Ok(())
}
