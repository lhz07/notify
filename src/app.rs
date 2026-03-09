use crate::{
    confirm_widget::{ActionConfirm, ConfirmWidget},
    id::{Id, IdGenerator},
    notification_widget::{Noti, NotificationWidget},
    proto::{self, Notification, StrSplit},
};
use indexmap::IndexMap;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListState, Paragraph, Wrap},
};
use std::{
    collections::{HashSet, VecDeque},
    io,
    path::PathBuf,
};
use unicode_width::UnicodeWidthStr;

enum Popup {
    DeleteSingle(Id),
    DeleteAll(ActionConfirm),
}

pub struct App {
    terminal: DefaultTerminal,
    notification_queue: VecDeque<Noti>,
    // id_generator: IdGenerator,
    notifications: IndexMap<Id, Notification>,
    current_popup: Option<Popup>,
    list_state: ListState,
}

impl App {
    pub fn init(terminal: DefaultTerminal, dir: PathBuf) -> io::Result<Self> {
        let mut id_generator = IdGenerator::new();
        let mut notifications = IndexMap::new();
        let noti = proto::get_notifications(dir)?;
        for n in noti {
            let id = id_generator.generate();
            notifications.insert(id, n);
        }
        Ok(Self {
            terminal,
            // id_generator,
            notifications,
            current_popup: None,
            notification_queue: VecDeque::new(),
            list_state: ListState::default(),
        })
    }
    pub fn run(&mut self) -> io::Result<()> {
        self.render()?;
        loop {
            let event = event::read()?;
            if self.handle_event(event) {
                break;
            }
            self.render()?;
        }
        Ok(())
    }
    fn handle_event(&mut self, event: Event) -> bool {
        let key = match event {
            Event::Key(key) => key,
            _ => return false,
        };

        if let KeyCode::Esc = key.code
            && let Some(noti) = self.notification_queue.front()
            && noti.is_appear()
        {
            self.notification_queue.pop_front();
            return false;
        }

        match &self.current_popup {
            Some(popup) => match popup {
                Popup::DeleteSingle(id) => match key.code {
                    KeyCode::Esc => self.current_popup = None,
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.current_popup = None;
                        self.list_state.select_next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.current_popup = None;
                        self.list_state.select_previous();
                    }
                    KeyCode::Char('d') => {
                        let id = *id;
                        self.current_popup = None;
                        if let Some(noti) = self.notifications.get(&id) {
                            match noti.delete() {
                                Ok(()) => {
                                    self.notifications.shift_remove(&id);
                                }
                                Err(e) => {
                                    let noti = Noti::new(
                                        "Error".to_string(),
                                        format!(
                                            "Can not delete \"{}\", error: {e}",
                                            noti.notify.title
                                        ),
                                    );
                                    self.notification_queue.push_back(noti);
                                }
                            }
                        }
                    }

                    _ => (),
                },
                Popup::DeleteAll(_) => match key.code {
                    KeyCode::Char('y') => {
                        self.current_popup = None;
                        if self.notifications.is_empty() {
                            return false;
                        }
                        let mut delete_id = HashSet::new();
                        for (id, noti) in &self.notifications {
                            match noti.delete() {
                                Ok(()) => (),
                                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                                Err(e) => {
                                    // show error and return early
                                    let noti = Noti::new(
                                        "Error".to_string(),
                                        format!(
                                            "Can not delete \"{}\", error: {e}",
                                            noti.notify.title
                                        ),
                                    );
                                    self.notification_queue.push_back(noti);
                                    self.notifications.retain(|id, _| !delete_id.contains(id));
                                    return false;
                                }
                            }
                            delete_id.insert(*id);
                        }
                        self.notifications.clear();
                    }
                    KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.current_popup = None
                    }
                    _ => (),
                },
            },
            None => {
                match key.code {
                    KeyCode::Down | KeyCode::Char('j') => self.list_state.select_next(),
                    KeyCode::Up | KeyCode::Char('k') => self.list_state.select_previous(),
                    // exit the app
                    KeyCode::Char('q') => return true,
                    KeyCode::Char('d') => {
                        // delete current notification
                        if let Some(index) = self.list_state.selected() {
                            let (id, _) = self
                                .notifications
                                .get_index(index)
                                .expect("index is always valid");
                            self.current_popup = Some(Popup::DeleteSingle(*id));
                        }
                    }
                    KeyCode::Char('C') => {
                        let confirm = ActionConfirm::new(
                            "Clear all notifications?",
                            "All notifications will be permanently deleted!",
                        );
                        self.current_popup = Some(Popup::DeleteAll(confirm))
                    }
                    _ => (),
                }
            }
        }

        false
    }
    fn render(&mut self) -> io::Result<()> {
        fn render_noti_lines(noti: &Notification, item_width: usize) -> Vec<Line<'_>> {
            let mut lines = Vec::with_capacity(4);

            lines.push(Line::from(noti.notify.program.as_str().bold()));
            let title = if noti.notify.title.width() > item_width {
                let s = noti.notify.title.length_split(item_width);
                vec![s.into(), "…".into()]
            } else {
                vec![noti.notify.title.as_str().into()]
            };
            lines.push(Line::from(title));
            lines.push(Line::from(vec![
                noti.notify.level.as_tui_color(),
                Span::from("  "),
                Span::from(noti.time_str()),
            ]));
            lines.push(Line::default());
            lines
        }

        self.terminal.draw(|f| {
            let main_layout =
                Layout::vertical([Constraint::Min(2), Constraint::Percentage(100)]).split(f.area());
            let title = "Notification Center".bold().into_centered_line();
            f.render_widget(title, main_layout[0]);
            let horizontal_layuout =
                Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                    .split(main_layout[1]);
            let list_area = horizontal_layuout[0];
            let detail_area = horizontal_layuout[1];
            let item_width = list_area.width.saturating_sub(5);
            let list_items = if let Some(Popup::DeleteSingle(selected)) = &self.current_popup {
                self.notifications
                    .iter()
                    .map(|(id, noti)| {
                        if id == selected {
                            #[rustfmt::skip]
                            const SPACE: &str  = "        │ ";
                            const DELETE: &str = "Delete? │ ";
                            let mut lines = Vec::with_capacity(4);
                            lines.push(Line::from(vec![
                                DELETE.into(),
                                noti.notify.program.as_str().bold(),
                            ]));
                            lines.push(Line::from(vec![
                                SPACE.into(),
                                noti.notify.title.as_str().into(),
                            ]));
                            lines.push(Line::from(vec![
                                SPACE.into(),
                                noti.notify.level.as_tui_color(),
                                Span::from("  "),
                                Span::from(noti.time_str()),
                            ]));
                            lines.push(Line::from(SPACE));
                            lines
                        } else {
                            render_noti_lines(noti, item_width as usize)
                        }
                    })
                    .collect()
            } else {
                self.notifications
                    .iter()
                    .map(|(_, noti)| render_noti_lines(noti, item_width as usize))
                    .collect::<Vec<_>>()
            };
            let list = List::new(list_items)
                .block(
                    Block::default()
                        .title("Notifications")
                        .borders(Borders::ALL),
                )
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED),
                )
                .highlight_symbol("› ");
            f.render_stateful_widget(list, list_area, &mut self.list_state);
            // list state will always be corrected after rendering
            if let Some(index) = self.list_state.selected() {
                let detail_block = Block::default()
                    .title("Detail")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::LightBlue));
                let mut lines = Vec::with_capacity(2);
                let noti = &self.notifications[index];
                lines.push(Line::from(Span::from(noti.notify.title.as_str()).bold()));
                lines.push(Line::default());
                let body = Text::from(noti.notify.body.as_str());
                lines.extend_from_slice(&body.lines);
                let detail_paragraph = Paragraph::new(lines)
                    .block(detail_block)
                    .wrap(Wrap { trim: false });
                f.render_widget(detail_paragraph, detail_area);
            }
            if let Some(Popup::DeleteAll(confirm)) = &mut self.current_popup {
                let vertical_layout = Layout::vertical([
                    Constraint::Percentage(10),
                    Constraint::Percentage(70),
                    Constraint::Percentage(20),
                ])
                .split(main_layout[1]);
                let horizontal_layout = Layout::horizontal([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(vertical_layout[1]);
                let popup_area = horizontal_layout[1];
                f.render_stateful_widget(ConfirmWidget, popup_area, confirm);
            }
            if let Some(noti) = self.notification_queue.front_mut() {
                f.render_stateful_widget(NotificationWidget, main_layout[1], noti);
                if noti.should_disappear() {
                    self.notification_queue.pop_front();
                    // render next one
                    if let Some(noti) = self.notification_queue.front_mut() {
                        f.render_stateful_widget(NotificationWidget, main_layout[1], noti);
                    }
                }
            }
        })?;
        Ok(())
    }
}
