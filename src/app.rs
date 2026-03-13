use crate::{
    confirm_widget::{ActionConfirm, ConfirmWidget},
    flags::LevelFlag,
    id::{Id, IdGenerator},
    notification_widget::{Noti, NotificationWidget},
    proto::{self, Level, Notification, StrSplit},
};
use indexmap::IndexMap;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Flex, Layout},
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
    current_selected: Option<Id>,
    list_state: ListState,
    level_flag: LevelFlag,
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
            level_flag: LevelFlag::default(),
            current_selected: None,
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
        if !key.modifiers.is_empty() && key.modifiers != KeyModifiers::SHIFT {
            return false;
        }

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
                    KeyCode::Char('d') | KeyCode::Enter => {
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
                        if let Some(id) = self.current_selected {
                            self.current_popup = Some(Popup::DeleteSingle(id));
                        }
                    }
                    KeyCode::Char('C') => {
                        let confirm = ActionConfirm::new(
                            "Clear all notifications?",
                            "All notifications will be permanently deleted!",
                        );
                        self.current_popup = Some(Popup::DeleteAll(confirm))
                    }
                    KeyCode::Char('i') | KeyCode::Char('1') => {
                        self.level_flag ^= LevelFlag::Info;
                    }
                    KeyCode::Char('n') | KeyCode::Char('2') => {
                        self.level_flag ^= LevelFlag::Notice;
                    }
                    KeyCode::Char('w') | KeyCode::Char('3') => {
                        self.level_flag ^= LevelFlag::Warning;
                    }
                    KeyCode::Char('c') | KeyCode::Char('4') => {
                        self.level_flag ^= LevelFlag::Critical;
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

        let notifications = self
            .notifications
            .iter()
            .filter(|(_, noti)| self.level_flag.contains(noti.notify.level.into()));

        self.terminal.draw(|f| {
            let main_layout = Layout::vertical([
                Constraint::Min(1),
                Constraint::Min(3),
                Constraint::Percentage(100),
            ])
            .split(f.area());
            let level_area = main_layout[1];
            let tab_layout =
                Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                    .split(level_area);
            let level_area = tab_layout[0];
            let hint_area = tab_layout[1];
            let content_area = main_layout[2];
            let title = "Notification Center".bold().into_centered_line();
            f.render_widget(title, main_layout[0]);
            let horizontal_layout =
                Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                    .split(content_area);
            let list_area = horizontal_layout[0];
            let detail_area = horizontal_layout[1];
            let item_width = list_area.width.saturating_sub(5);
            let list_items = {
                let notifications = notifications.clone();

                if let Some(Popup::DeleteSingle(selected)) = &self.current_popup {
                    notifications
                        .map(|(id, noti)| {
                            if id == selected {
                                const SPACE: &str = "        │ ";
                                vec![
                                    Line::from(vec![
                                        "D".underlined(),
                                        "elete? │ ".into(),
                                        noti.notify.program.as_str().bold(),
                                    ]),
                                    Line::from(vec![
                                        SPACE.into(),
                                        noti.notify.title.as_str().into(),
                                    ]),
                                    Line::from(vec![
                                        SPACE.into(),
                                        noti.notify.level.as_tui_color(),
                                        Span::from("  "),
                                        Span::from(noti.time_str()),
                                    ]),
                                    Line::from(SPACE),
                                ]
                            } else {
                                render_noti_lines(noti, item_width as usize)
                            }
                        })
                        .collect()
                } else {
                    notifications
                        .map(|(_, noti)| render_noti_lines(noti, item_width as usize))
                        .collect::<Vec<_>>()
                }
            };

            let level_block = Block::default().borders(Borders::ALL);
            let inner = level_block.inner(level_area);
            f.render_widget(level_block, level_area);
            const LEVEL_LEN: u16 = 5;
            let chunks = Layout::horizontal([
                Constraint::Length(LEVEL_LEN),
                Constraint::Length(LEVEL_LEN),
                Constraint::Length(LEVEL_LEN),
                Constraint::Length(LEVEL_LEN),
            ])
            .flex(Flex::SpaceEvenly)
            .split(inner);

            for (level, area) in Level::LIST.into_iter().zip(chunks.iter()) {
                let level_str = level.as_tui_color_short();
                let level_str = if self.level_flag.contains(level.into()) {
                    level_str.bold()
                } else {
                    level_str
                };
                f.render_widget(level_str, *area);
            }

            // render key hint
            let key_block = Block::default().borders(Borders::ALL);
            let inner = key_block.inner(hint_area);
            f.render_widget(key_block, hint_area);
            let chunks = Layout::horizontal([
                Constraint::Length(9),
                Constraint::Length(12),
                Constraint::Length(14),
            ])
            .flex(Flex::SpaceEvenly)
            .split(inner);
            f.render_widget("<q> Quit".bold(), chunks[0]);
            f.render_widget("<d> Delete".bold(), chunks[1]);
            f.render_widget("<C> Clear All".bold(), chunks[2]);

            if list_items.is_empty() {
                let title = Line::from("Notification");
                let block = Block::default().title(title).borders(Borders::ALL);
                let inner = block.inner(list_area);
                f.render_widget(block, list_area);
                let layout = Layout::vertical([
                    Constraint::Fill(7),
                    Constraint::Length(1),
                    Constraint::Fill(8),
                ])
                .flex(Flex::Center)
                .split(inner);
                f.render_widget(Line::from("No notification").centered().bold(), layout[1]);
            } else {
                let title = if list_items.len() == 1 {
                    Span::from("Notification (1)")
                } else {
                    format!("Notifications ({})", list_items.len()).into()
                };
                let list = List::new(list_items)
                    .block(Block::default().title(title).borders(Borders::ALL))
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
                    let mut notifications = notifications;
                    let (id, noti) = notifications.nth(index).expect("index is always valid");
                    self.current_selected = Some(*id);
                    let detail_block = Block::default()
                        .title("Detail")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::LightBlue));
                    let mut lines = Vec::with_capacity(2);
                    lines.push(Line::from(Span::from(noti.notify.title.as_str()).bold()));
                    lines.push(Line::default());
                    let body = Text::from(noti.notify.body.as_str());
                    lines.extend_from_slice(&body.lines);
                    let detail_paragraph = Paragraph::new(lines)
                        .block(detail_block)
                        .wrap(Wrap { trim: false });
                    f.render_widget(detail_paragraph, detail_area);
                } else {
                    self.current_selected = None;
                }
            };
            if let Some(Popup::DeleteAll(confirm)) = &mut self.current_popup {
                let vertical_layout = Layout::vertical([
                    Constraint::Percentage(10),
                    Constraint::Percentage(70),
                    Constraint::Percentage(20),
                ])
                .split(content_area);
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
                f.render_stateful_widget(NotificationWidget, content_area, noti);
                if noti.should_disappear() {
                    self.notification_queue.pop_front();
                    // render next one
                    if let Some(noti) = self.notification_queue.front_mut() {
                        f.render_stateful_widget(NotificationWidget, content_area, noti);
                    }
                }
            }
        })?;
        Ok(())
    }
}
