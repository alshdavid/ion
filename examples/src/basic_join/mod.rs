use std::sync::Arc;
use std::time::Duration;

use ion::*;
use serde::Serialize;

use crate::_utils::thread_id;

#[derive(Serialize)]
struct Report {
    thread: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    js_context: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_loop: Option<bool>,
    message: String,
}

impl Report {
    fn print(
        js_context: Option<usize>,
        event_loop: Option<bool>,
        message: &str,
    ) {
        let thread = if event_loop.is_none() {
            thread_id::thread_id()
        } else {
            1000
        };
        println!(
            "{}",
            serde_json::to_string(&Report {
                thread,
                js_context: js_context,
                event_loop: event_loop,
                message: message.to_string(),
            })
            .unwrap()
        )
    }
}

pub fn main() -> anyhow::Result<()> {
    let case = std::env::args()
        .collect::<Vec<String>>()
        .get(2)
        .cloned()
        .expect("No code provided");

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions::default())?;

    Report::print(None, None, "start");
    #[rustfmt::skip]
    match case.as_str() {
        "should_cancel_when_dropped" => should_cancel_when_dropped(runtime),
        "should_cancel_when_dropped_multiple" => should_cancel_when_dropped_multiple(runtime),
        "should_cancel_blocking_when_dropped" => should_cancel_blocking_when_dropped(runtime),
        "should_cancel_blocking_when_dropped_multiple" => should_cancel_blocking_when_dropped_multiple(runtime),
        "should_wait_for_code_to_finish" => should_wait_for_code_to_finish(runtime),
        "should_wait_for_code_to_finish_multiple" => should_wait_for_code_to_finish_multiple(runtime),
        "should_wait_for_code_to_finish_blocking" => should_wait_for_code_to_finish_blocking(runtime),
        "should_wait_for_code_to_finish_worker" => should_wait_for_code_to_finish_worker(runtime),
        "should_wait_for_code_to_finish_worker_blocking" => should_wait_for_code_to_finish_worker_blocking(runtime),
        "should_wait_for_code_to_finish_context" => should_wait_for_code_to_finish_context(runtime),
        "should_wait_for_code_to_finish_context_blocking" => should_wait_for_code_to_finish_context_blocking(runtime),
        _ => panic!("No Case Selected"),
    }?;

    Report::print(None, None, "end");
    Ok(())
}

fn non_blocking_exec(context: usize) -> Box<dyn Send + Fn(&Env) -> ion::Result<()>> {
    return Box::new(move |env| {
        env.inc_ref();
        Report::print(Some(context), None, "start");

        env.spawn_background({
            let env = env.as_async();

            async move {
                Report::print(Some(context), Some(true), "start");
                tokio::time::sleep(Duration::from_millis(1000)).await;
                Report::print(Some(context), Some(true), "end");

                env.exec_async(move |env| {
                    env.dec_ref();
                    Report::print(Some(context), None, "resolved");
                    Ok(())
                })
                .await
            }
        })?;

        Report::print(Some(context), None, "end");
        Ok(())
    });
}

fn should_cancel_when_dropped(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec(non_blocking_exec(0))?;
    Ok(())
}

fn should_cancel_when_dropped_multiple(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec(non_blocking_exec(0))?;
    c0.exec(non_blocking_exec(0))?;

    Ok(())
}

fn should_cancel_blocking_when_dropped(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec_blocking(non_blocking_exec(0))?;

    Ok(())
}

fn should_cancel_blocking_when_dropped_multiple(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec_blocking(non_blocking_exec(0))?;
    c0.exec_blocking(non_blocking_exec(0))?;

    Ok(())
}

fn should_wait_for_code_to_finish(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec(non_blocking_exec(0))?;

    c0.join_blocking()?;
    w0.join_blocking()?;

    Ok(())
}

fn should_wait_for_code_to_finish_multiple(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec(non_blocking_exec(0))?;
    c0.exec(non_blocking_exec(0))?;

    c0.join_blocking()?;
    w0.join_blocking()?;

    Ok(())
}

fn should_wait_for_code_to_finish_blocking(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec_blocking(non_blocking_exec(0))?;

    c0.join_blocking()?;
    w0.join_blocking()?;

    Ok(())
}

fn should_wait_for_code_to_finish_worker(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec(non_blocking_exec(0))?;

    w0.join_blocking()?;

    Ok(())
}

fn should_wait_for_code_to_finish_worker_blocking(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec_blocking(non_blocking_exec(0))?;

    w0.join_blocking()?;

    Ok(())
}

fn should_wait_for_code_to_finish_context(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec(non_blocking_exec(0))?;

    c0.join_blocking()?;

    Ok(())
}

fn should_wait_for_code_to_finish_context_blocking(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    let w0 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let c0 = w0.create_context()?;

    c0.exec_blocking(non_blocking_exec(0))?;

    c0.join_blocking()?;

    Ok(())
}
