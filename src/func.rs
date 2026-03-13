use crate::proto::{self, Level};
use owo_colors::OwoColorize;
use std::borrow::Cow;

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
