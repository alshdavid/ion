use ion::*;

use crate::_utils::memory_blob::blob_100kb;
use crate::_utils::memory_usage::MemoryUsageCounter;

pub fn main() -> anyhow::Result<()> {
    let memu = MemoryUsageCounter::default();
    println!("{}", memu.kilobytes().json());

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions::debug(JsRuntimeOptions {
        ..Default::default()
    }))?;

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;

    for _ in 0..100 {
        worker.run_garbage_collection_for_testing()?;

        let ctx = worker.create_context()?;
        println!("{}", memu.kilobytes().json());

        for _ in 0..10 {
            println!("{}", memu.kilobytes().json());
            let data = blob_100kb();

            ctx.exec_blocking(move |env| {
                let external_js = JsExternal::new(env, data)?;
                env.global_this()?
                    .set_named_property("external", external_js)?;

                Ok(())
            })?;

            // TODO
            // Currently GC only seems to fire when the context is dropped
            ctx.exec_blocking(|env| env.global_this()?.delete_named_property("external"))?;
            worker.run_garbage_collection_for_testing()?;
            // TODO-END

            println!("{}", memu.kilobytes().json());
        }

        println!("{}", memu.kilobytes().json());
    }

    worker.run_garbage_collection_for_testing()?;
    println!("{}", memu.kilobytes().json());

    Ok(())
}
