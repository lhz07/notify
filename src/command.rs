use std::{borrow::Cow, fs, path::PathBuf};

use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "notify",
    version = "0.1.0",
    about = "A simple notification center"
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// show brief notifications
    Status,

    /// view all notifications
    View,

    /// generate shell completions
    Completions(ShellArgs),
}

#[derive(Parser)]
pub struct ShellArgs {
    /// The shell to generate completions for
    ///
    /// This command prints the completion script to stdout. You usually need
    /// to redirect it to a file using '>'.
    ///
    /// See `--install` for installing the completion file automatically
    /// into the standard directory for your shell.
    ///
    /// Example:
    ///   notify completions \
    ///     --shell bash > ~/.config/fish/completions/notify.fish
    ///
    /// After installing the file, restart your shell or reload completion.
    #[arg(long_help)]
    #[arg(verbatim_doc_comment)]
    pub shell: Shell,

    /// Write the completion file directly to the default path of the shell
    ///
    /// If this option is specified, the program will attempt to write the
    /// completion script to the standard completion directory automatically.
    ///
    /// Default locations:
    ///
    /// bash: ~/.local/share/bash-completion/completions/notify
    ///
    /// zsh: ~/.local/share/zsh/site-functions/_notify
    ///      (you need to add this directory to $fpath manually)
    ///
    /// fish: ~/.config/fish/completions/notify.fish
    #[arg(verbatim_doc_comment)]
    #[arg(long = "install", long_help)]
    pub install: bool,
}

pub fn prepare_path(shell: Shell) -> Result<PathBuf, Cow<'static, str>> {
    let home = std::env::home_dir().ok_or("Can not get home dir")?;
    home.canonicalize()
        .map_err(|e| format!("{e}, can not visit home: {}", home.display()))?;
    let path = match shell {
        Shell::Bash => ".local/share/bash-completion/completions/notify",
        Shell::Zsh => ".local/share/zsh/site-functions/_notify",
        Shell::Fish => ".config/fish/completions/notify.fish",
        _ => {
            return Err("unsupported shell")?;
        }
    };
    let path = home.join(path);
    let folder = path.parent().expect("parent folder is hard coded");
    fs::create_dir_all(folder)
        .map_err(|e| format!("{e}, can not create dir: {}", folder.display()))?;
    Ok(path)
}
