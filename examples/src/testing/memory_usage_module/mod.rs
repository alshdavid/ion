use std::thread;
use std::time::Duration;

use ion::*;

use crate::testing::MemoryUsageCounter;

pub fn main() -> anyhow::Result<()> {
    let memu = MemoryUsageCounter::default();
    println!("[0] {:?}", memu);

    let runtime = JsRuntime::initialize_debug()?;
    println!("[1] {:?}", memu);

    let worker = runtime.spawn_worker()?;

    for i in 2..50 {
        let ctx = worker.create_context()?;

        for _ in 2..1000 {
            ctx.exec_blocking(|env| {
                env.eval_module("export {}")?;
                Ok(())
            })?;
        }

        worker.run_garbage_collection_for_testing()?;
        println!("[{}] {:?}", i, memu);
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
