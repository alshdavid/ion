use ion::*;

pub fn main() -> anyhow::Result<()> {
    let code = std::env::args()
        .collect::<Vec<String>>()
        .get(2)
        .cloned()
        .expect("No code provided");

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions {
        extensions: vec![
            ion::extensions::console(),
            ion::extensions::set_interval(),
            ion::extensions::set_timeout(),
        ],
        ..Default::default()
    })?;

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;
    let ctx = worker.create_context()?;

    ctx.eval(code)?;
    Ok(())
}
