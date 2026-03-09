use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, Clear, Paragraph, StatefulWidget, Widget, Wrap};
use std::time::{Duration, Instant};

pub struct Noti {
    title: String,
    content: String,
    instant: Option<Instant>,
    duration: Duration,
    should_disappear: bool,
}

pub struct NotificationWidget;

impl Noti {
    pub fn new(title: String, content: String) -> Self {
        Self {
            title,
            content,
            instant: None,
            duration: Duration::from_secs(3),
            should_disappear: false,
        }
    }
    pub fn is_appear(&self) -> bool {
        self.instant.is_some()
    }
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
    pub fn should_disappear(&self) -> bool {
        self.should_disappear
    }
}

impl StatefulWidget for NotificationWidget {
    type State = Noti;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        match state.instant {
            Some(instant) => {
                let age = instant.elapsed();
                if age > state.duration {
                    state.should_disappear = true;
                    return;
                }
            }
            None => {
                state.instant = Some(Instant::now());
            }
        }
        let vertical =
            Layout::vertical([Constraint::Fill(1), Constraint::Percentage(30)]).split(area)[1];
        let area = Layout::horizontal([Constraint::Fill(1), Constraint::Percentage(40)])
            .split(vertical)[1];
        let block = Block::bordered().title(state.title.as_str());
        // clear the area to ensure we are on the top
        let clear = Clear;
        clear.render(area, buf);
        let para = Paragraph::new(state.content.as_str())
            .block(block)
            .wrap(Wrap { trim: true });
        para.render(area, buf);
    }
}
