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

    for _ in 0..300 {
        let worker = runtime.spawn_worker(JsWorkerOptions::default())?;

        {
            let ctx0 = worker.create_context()?;
            let ctx1 = worker.create_context()?;

            ctx0.eval("globalThis.value = []")?;
            for i in 0..1 {
                ctx0.eval(format!("globalThis.value.push({})", i))?;
            }

            ctx1.eval("globalThis.value = []")?;
            for i in 0..100 {
                ctx1.eval(format!("globalThis.value.push({})", i))?;
            }

            ctx0.join()?;
            ctx1.join()?;
        };

        worker.run_garbage_collection_for_testing()?;
        worker.join()?;

        println!("{}", memu.megabytes().json());
    }

    println!("{}", memu.megabytes().json());

    Ok(())
}
