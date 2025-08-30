mod cmd;

use clap::Parser;
use clap::Subcommand;

#[derive(Debug, Parser)]
struct Command {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Execute a file
    Run(cmd::run::RunCommand),
    /// Evaluate code from commandline
    Eval(cmd::eval::EvalCommand),
}

pub fn main() -> anyhow::Result<()> {
    let command = Command::parse();

    // dbg!(&command);

    match command.command {
        Commands::Run(command) => cmd::run::main(command),
        Commands::Eval(command) => cmd::eval::main(command),
    }
}
