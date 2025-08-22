use clap::Parser;

#[derive(Debug, Parser)]
pub struct EvalCommand {
    /// Code to evaluate
    pub code: String,
}

pub fn main(command: EvalCommand) -> anyhow::Result<()> {
    let runtime = ion::platform::initialize_once()?;

    let worker = runtime.spawn_worker()?;
    let ctx = worker.create_context()?;

    ctx.exec_blocking(|env| {
        ion::exts::define_console(&env);
        ion::exts::define_set_timeout(&env);
        ion::exts::define_set_interval(&env);

        env.eval_script(command.code)?;
        Ok(())
    })?;

    Ok(())
}
