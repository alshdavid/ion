use std::path::PathBuf;

use clap::Parser;
use normalize_path::NormalizePath;

#[derive(Debug, Parser)]
pub struct RunCommand {
    /// Target get file to run
    pub path: PathBuf,
}

pub fn main(command: RunCommand) -> anyhow::Result<()> {
    let entry = if command.path.is_absolute() {
        command.path
    } else {
        let Ok(cwd) = std::env::current_dir() else {
            return Err(anyhow::anyhow!("Unable to get cwd"));
        };
        cwd.join(&command.path).normalize()
    }
    .normalize();

    let entry = std::fs::read_to_string(entry)?;
    let runtime = ion::platform::initialize_once()?;

    let worker = runtime.spawn_worker()?;
    let ctx = worker.create_context()?;

    ctx.exec_blocking(|env| {
        // println!("hello");
        ion::exts::define_console(&env);
        ion::exts::define_set_timeout(&env);
        ion::exts::define_set_interval(&env);

        env.eval_script(entry)?;

        Ok(())
    })?;

    Ok(())
}
