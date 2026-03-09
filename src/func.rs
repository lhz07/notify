use crate::proto::{self, Level};
use owo_colors::OwoColorize;

pub fn status() {
    println!();
    let Some(mut dir) = dirs::data_local_dir() else {
        eprintln!("Can not get data local dir");
        return;
    };
    dir.push("notify");
    match proto::get_notifications(&dir) {
        Ok(notifications) => {
            let count = notifications.len();
            if count == 0 {
                println!("{}\n", "No unread notifications".bold());
                return;
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
            println!("\nRun `notify view` for full list.\n");
        }
        Err(e) => eprintln!("Can not read notifications, error: {e}\n"),
    }
}
