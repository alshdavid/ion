use clap::Parser;
use ion::*;

#[derive(Debug, Parser)]
pub struct EvalCommand {
    /// Code to evaluate
    pub code: String,
}

pub fn main(command: EvalCommand) -> anyhow::Result<()> {
    let runtime = JsRuntime::initialize_once(JsRuntimeOptions {
        v8_args: vec![],
        resolvers: vec![ion::resolvers::relative()],
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
    })?;

    let worker = runtime.spawn_worker()?;
    let ctx = worker.create_context()?;

    ctx.exec_blocking(|env| {
        env.eval_script::<JsUnknown>(command.code)?;
        Ok(())
    })?;

    Ok(())
}
