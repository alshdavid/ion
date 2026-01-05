use ion::*;

use crate::_utils::memory_usage::MemoryUsageCounter;

pub fn main() -> anyhow::Result<()> {
    let memu = MemoryUsageCounter::default();
    println!("{}", memu.kilobytes().json());

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

    println!("{}", memu.kilobytes().json());

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;

    for _ in 2..100 {
        let ctx = worker.create_context()?;

        for _ in 2..1000 {
            ctx.exec_blocking(|env| {
                env.eval_module("export {}")?;
                Ok(())
            })?;
        }

        worker.run_garbage_collection_for_testing()?;
        println!("{}", memu.kilobytes().json());
    }

    Ok(())
}
