use clap::Parser;
use notify::{
    command::{CliArgs, Commands},
    func,
};
use std::borrow::Cow;

fn main() -> Result<(), Cow<'static, str>> {
    let args = CliArgs::parse();
    match args.command {
        Commands::View => {
            func::view()?;
        }
        Commands::Completions(shell_args) => {
            func::completions(shell_args)?;
        }
        Commands::Status => func::status()?,
    }
    Ok(())
}
