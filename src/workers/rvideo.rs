use crate::prelude::*;
use roboplc::controller::*;
use roboplc_derive::WorkerOpts;
#[derive(WorkerOpts)]
#[worker_opts(cpu = 2, priority = 80, scheduling = "fifo", blocking = true)]
pub struct RvideoSrv {}

impl Worker<WorkerMessage, Variables> for RvideoSrv {
    fn run(&mut self, _context: &Context<WorkerMessage, Variables>) -> WResult { roboplc::serve_rvideo().map_err(Into::into) }
}
