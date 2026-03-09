use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, StatefulWidget, Widget, Wrap};
use std::borrow::Cow;

pub struct ActionConfirm {
    question: Cow<'static, str>,
    content: Cow<'static, str>,
}

impl ActionConfirm {
    pub fn new<S, T>(question: S, content: T) -> Self
    where
        S: Into<Cow<'static, str>>,
        T: Into<Cow<'static, str>>,
    {
        Self {
            question: question.into(),
            content: content.into(),
        }
    }
}

pub struct ConfirmWidget;

impl StatefulWidget for ConfirmWidget {
    type State = ActionConfirm;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // clear the area to ensure we are on the top
        let clear = Clear;
        clear.render(area, buf);
        let block = Block::bordered().title(&*state.question);
        let layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(block.inner(area));
        block.render(area, buf);
        let content_area = layout[0];
        let line_area = layout[1];
        let option_area = layout[2];
        let content_para = Paragraph::new(&*state.content).wrap(Wrap { trim: true });
        content_para.render(content_area, buf);
        let line = Block::default().borders(Borders::BOTTOM);
        line.render(line_area, buf);
        let option_layout =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(option_area);
        let yes = "(Y)es";
        let no = "(N)o";
        yes.render(option_layout[0], buf);
        no.render(option_layout[1], buf);
    }
}
