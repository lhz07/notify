use std::borrow::Cow;

use notify::{app::App, func};

fn main() -> Result<(), Cow<'static, str>> {
    let mut args = std::env::args();
    args.next();
    let second = args.next();
    let third = args.next();
    if let Some(s) = &second
        && s == "status"
        && third.is_none()
    {
        func::status()?;
    } else if let Some(s) = &second
        && s == "view"
        && third.is_none()
    {
        let mut dir = dirs::data_local_dir().ok_or("Can not get data local dir")?;
        dir.push("notify");
        let terminal = ratatui::init();
        let mut app =
            App::init(terminal, dir).map_err(|e| format!("Can not initialize app, error: {e}"))?;
        let res = app.run();
        ratatui::restore();
        res.map_err(|e| format!("app error: {e}"))?;
    } else {
        Err("unknown command")?;
    }
    Ok(())
}
