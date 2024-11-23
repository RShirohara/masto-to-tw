use std::error::Error;

use worker::{
  event, Context, Env, Request, Response, Result as WorkerResult, ScheduleContext, ScheduledEvent,
};

#[event(fetch)]
pub async fn fetch(_req: Request, env: Env, _ctx: Context) -> WorkerResult<Response> {
  Response::ok("Hello, workers!")
}

#[event(scheduled)]
pub async fn scheduled(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {}

async fn sync_posts(env: &Env) -> Result<(), Box<dyn Error>> {
  Ok(())
}
