use ion::*;

pub fn main() -> anyhow::Result<()> {
    let runtime = JsRuntime::initialize_once(JsRuntimeOptions::default())?;

    let wrk1 = runtime.spawn_worker()?;
    let wrk2 = runtime.spawn_worker()?;
    let wrk3 = runtime.spawn_worker()?;

    drop(wrk1);
    drop(wrk2);
    drop(wrk3);

    Ok(())
}
