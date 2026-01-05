use ion::*;

use crate::_utils::memory_usage::MemoryUsageCounter;

pub fn main() -> anyhow::Result<()> {
    let memu = MemoryUsageCounter::default();
    println!("{}", memu.kilobytes().json());

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions::default())?;
    println!("{}", memu.kilobytes().json());

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;
    let ctx = worker.create_context()?;

    for _ in 0..200 {
        for _ in 0..3000 {
            let _value = ctx.exec_blocking(|env| {
                let value = env.eval_script::<JsNumber>("1 + 1")?;
                value.get_u32()
            })?;
        }

        println!("{}", memu.kilobytes().json());
    }

    Ok(())
}
