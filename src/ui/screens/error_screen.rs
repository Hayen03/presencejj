use std::sync::Arc;

use ratatui::{style::{Style, Stylize}, text::{Line, Text, ToText}, widgets::{Paragraph, Widget, WidgetRef}};

use crate::ui::{AppState, Screen};

#[derive(Debug)]
enum ErrorScreenContent<'a> {
	Text(Text<'a>),
	Error(Box<dyn std::error::Error>),
}
impl ErrorScreenContent<'_> {
	fn text(&'_ self) -> Text<'_> {
		match self {
			ErrorScreenContent::Text(text) => text.clone(),
			ErrorScreenContent::Error(err) => err.to_text(),
		}
	}
}

#[derive(Debug)]
pub struct ErrorScreen<'a> {
	_message: ErrorScreenContent<'a>,
	scroll: u16,
}
impl<'a> ErrorScreen<'a> {
	pub fn fromstr(value: &'a str) -> Self {
		let lines = value.lines().map(Line::from).collect::<Vec<_>>();
		Self { _message: ErrorScreenContent::Text(Text::from(lines).red()), scroll: 0 }
	}
	pub fn from_text(value: Text<'a>) -> Self {
		Self { _message: ErrorScreenContent::Text(value), scroll: 0 }
	}
	pub fn from_error(value: Box<dyn std::error::Error>) -> Self {
		Self {
			_message: ErrorScreenContent::Error(value),
			scroll: 0,
		}
	}
}
impl WidgetRef for ErrorScreen<'_> {
	fn render_ref(&self, area:ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
		// find the dims of the box we want to render the error message in, with some padding
		let box_width = crate::ui::Theme::DARK.max_error_box_width.min(area.width);
		let box_height = crate::ui::Theme::DARK.max_error_box_height.min(area.height);
		let box_area = area.centered(ratatui::layout::Constraint::Length(box_width), ratatui::layout::Constraint::Length(box_height));
		let title = Line::from(" Une erreur est survenue! ").centered().red();
		let instruction = Line::from(" Appuyez sur Entrée pour continuer. ").centered().gray();
		let block = ratatui::widgets::Block::bordered()
			.title(title)
			.title_bottom(instruction)
			.border_set(ratatui::symbols::border::THICK)
			.border_style(Style::new().red())
			.bg(ratatui::style::Color::Black);
		let error_text = Paragraph::new(self._message.text())
			.block(block)
			.wrap(ratatui::widgets::Wrap { trim: false })
			.scroll((0, self.scroll));
		ratatui::widgets::Clear.render(box_area, buf);
		error_text.render(box_area, buf);
	}
}
impl Screen for ErrorScreen<'_> {
	fn handle_event(&mut self, event: crate::ui::event::Event, _state: Arc<AppState>) -> Result<crate::ui::UpdateAction, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				match key.code {
					crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Enter => { Ok(crate::ui::UpdateAction::Pop) },
					crossterm::event::KeyCode::Up => {
						if self.scroll > 0 {
							self.scroll -= 1;
						}
						Ok(crate::ui::UpdateAction::Continue)
					},
					crossterm::event::KeyCode::Down => {
						self.scroll += 1;
						Ok(crate::ui::UpdateAction::Continue)
					},
					_ => { Ok(crate::ui::UpdateAction::Continue) },
				}
			},
			_ => { Ok(crate::ui::UpdateAction::Continue) },
		}
	}
}