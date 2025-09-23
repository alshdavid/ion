use std::thread;
use std::time::Duration;

use ion::*;

use crate::testing::MemoryUsageCounter;

pub fn main() -> anyhow::Result<()> {
    let memu = MemoryUsageCounter::default();
    println!("[0] {:?}", memu);

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions::default())?;
    println!("[1] {:?}", memu);

    for i in 2..50 {
        {
            let worker = runtime.spawn_worker()?;
            worker.run_garbage_collection_for_testing()?;
            drop(worker);
        };

        println!("[{}] {:?}", i, memu);
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
