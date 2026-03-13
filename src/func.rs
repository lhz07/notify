use crate::{
    app::App,
    command::{CliArgs, ShellArgs, prepare_path},
    proto::{self, Level},
};
use clap::CommandFactory;
use owo_colors::OwoColorize;
use std::{borrow::Cow, io::BufWriter};

pub fn status() -> Result<(), Cow<'static, str>> {
    let mut dir = dirs::data_local_dir().ok_or("Can not get data local dir")?;
    dir.push("notify");
    let notifications =
        proto::get_notifications(&dir).map_err(|e| format!("Can not read notifications, {e}\n"))?;
    let count = notifications.len();
    if count == 0 {
        println!("{}", "No unread notifications".bold());
        return Ok(());
    }
    println!("{}", format!("{count} unread notifications\n").bold());
    if count <= 10 {
        for noti in notifications {
            println!("{}", noti);
        }
    } else {
        // just show brief info here
        let mut list = [
            (Level::Info, 0),
            (Level::Notice, 0),
            (Level::Warning, 0),
            (Level::Critical, 0),
        ];
        for n in notifications.iter() {
            list[n.notify.level as usize].1 += 1;
        }
        for (level, count) in list {
            println!("{}: {}", level, count);
        }

        println!();
        // only display the most important notifications
        for (level, count) in list.iter().rev() {
            if *count > 0 {
                println!("{}", format!("Recent {}: ", level.as_ref()).bold());
                for n in notifications
                    .iter()
                    .filter(|n| n.notify.level == *level)
                    .take(5)
                {
                    println!("{}", n);
                }
                break;
            }
        }
    }
    println!("\nRun `notify view` for full list.");
    Ok(())
}

pub fn view() -> Result<(), Cow<'static, str>> {
    let mut dir = dirs::data_local_dir().ok_or("Can not get data local dir")?;
    dir.push("notify");
    let terminal = ratatui::init();
    let mut app =
        App::init(terminal, dir).map_err(|e| format!("Can not initialize app, error: {e}"))?;
    let res = app.run();
    ratatui::restore();
    res.map_err(|e| format!("app error: {e}"))?;
    Ok(())
}

pub fn completions(shell_args: ShellArgs) -> Result<(), Cow<'static, str>> {
    let mut cmd = CliArgs::command();
    let bin_name = cmd.get_name().to_string();
    if shell_args.install {
        let path = prepare_path(shell_args.shell)?;
        let file = std::fs::File::create(&path).map_err(|e| format!("{e}: {}", path.display()))?;
        let mut content = BufWriter::new(file);
        clap_complete::generate(shell_args.shell, &mut cmd, bin_name, &mut content);
        println!("Successfully installed {} completions!", shell_args.shell)
    } else {
        clap_complete::generate(shell_args.shell, &mut cmd, bin_name, &mut std::io::stdout());
    }
    Ok(())
}
