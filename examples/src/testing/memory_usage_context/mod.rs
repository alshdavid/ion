use std::thread;
use std::time::Duration;

use ion::*;

use crate::testing::MemoryUsageCounter;

pub fn main() -> anyhow::Result<()> {
    let memu = MemoryUsageCounter::default();
    println!("[0] {:?}", memu);

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions::default())?;
    println!("[1] {:?}", memu);

    for i in 0..50 {
        {
            let worker = runtime.spawn_worker(JsWorkerOptions::default())?;

            {
                let ctx0 = worker.create_context()?;
                let ctx1 = worker.create_context()?;

                drop(ctx0);
                drop(ctx1);
            };

            worker.run_garbage_collection_for_testing()?;
            drop(worker);
        };

        println!("[{}] {:?}", i, memu);
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
