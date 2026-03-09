use notify::{app::App, func};

fn main() {
    let mut args = std::env::args();
    args.next();
    let second = args.next();
    let third = args.next();
    if let Some(s) = &second
        && s == "status"
        && third.is_none()
    {
        func::status();
    } else if let Some(s) = &second
        && s == "view"
        && third.is_none()
    {
        let Some(mut dir) = dirs::data_local_dir() else {
            eprintln!("Can not get data local dir");
            return;
        };
        dir.push("notify");
        let terminal = ratatui::init();

        let mut app = match App::init(terminal, dir) {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Can not initialize app, error: {e}");
                return;
            }
        };
        let res = app.run();
        ratatui::restore();
        if let Err(e) = res {
            eprintln!("app error: {e}");
        }
    } else {
        eprintln!("unknown command");
    }
}
