use std::thread;
use std::time::Duration;

use ion::*;

use crate::_utils::memory_usage::MemoryUsageCounter;

pub fn main() -> anyhow::Result<()> {
    let memu = MemoryUsageCounter::default();

    println!("{}", memu.megabytes().json());

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions::debug(JsRuntimeOptions {
        transformers: vec![
            ion::transformers::json(),
            ion::transformers::ts(),
            ion::transformers::tsx(),
        ],
        extensions: vec![
            ion::extensions::event_target(),
            ion::extensions::console(),
            ion::extensions::set_timeout(),
            ion::extensions::set_interval(),
            ion::extensions::test(),
            ion::extensions::global_this(),
        ],
        ..Default::default()
    }))?;

    println!("{}", memu.megabytes().json());

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;

    for _ in 2..50 {
        let ctx = worker.create_context()?;
        let mut v = vec![];

        for _ in 2..1000 {
            let tsfn = ctx.exec_blocking(|env| {
                let func = JsFunction::new(env, |_env, ctx| ctx.arg::<JsNumber>(0))?;
                ThreadSafeFunction::new(&func)
            })?;

            tsfn.call_blocking(
                // Map Args
                |_env| Ok(42),
                // Map Ret
                move |_env, ret| ret.cast::<JsNumber>()?.get_u32(),
            )
            .unwrap();

            v.push(tsfn);
        }

        v.clear();
        worker.run_garbage_collection_for_testing()?;

        println!("{}", memu.megabytes().json());
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
