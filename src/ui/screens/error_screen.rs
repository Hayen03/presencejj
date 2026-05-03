use std::sync::Arc;

use ratatui::{style::{Style, Stylize}, text::{Line, Text, ToText}, widgets::{Paragraph, Widget, WidgetRef}};

use crate::ui::{AppState, Screen, ScreenSize};

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
	fn prefered_width(&self) -> u16 {
		let text = self.text();
		text.lines.iter().map(|line| line.spans.iter().map(|span| span.width()).sum()).max().unwrap_or(0) as u16 + 2
	}
}

#[derive(Debug)]
pub struct ErrorScreen<'a> {
	_message: ErrorScreenContent<'a>,
	scroll: u16,
	prefered_width: u16,
}
impl<'a> ErrorScreen<'a> {
	pub fn fromstr(value: &'a str) -> Self {
		let lines = value.lines().map(Line::from).collect::<Vec<_>>();
		let err = ErrorScreenContent::Text(Text::from(lines).red());
		let prefered_width = err.prefered_width();
		Self { _message: err, scroll: 0, prefered_width }
	}
	pub fn from_text(value: Text<'a>) -> Self {
		let err = ErrorScreenContent::Text(value);
		let prefered_width = err.prefered_width();
		Self { _message: err, scroll: 0, prefered_width }
	}
	pub fn from_error(value: Box<dyn std::error::Error>) -> Self {
		let err = ErrorScreenContent::Error(value);
		let prefered_width = err.prefered_width();
		Self {
			_message: err,
			scroll: 0,
			prefered_width,
		}
	}
}
impl WidgetRef for ErrorScreen<'_> {
	fn render_ref(&self, area:ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
		// find the dims of the box we want to render the error message in, with some padding
		let box_width = match crate::ui::Theme::DARK.max_error_box_width {
			ScreenSize::Fill => area.width,
			ScreenSize::Length(l) => l,
			ScreenSize::Ratio(r) => (area.width as f32 * r).floor() as u16,
			ScreenSize::Fit {min, max} => {
				self.prefered_width.clamp(min, max)
			},
		};
		let tmp_area = area.centered(ratatui::layout::Constraint::Max(box_width), ratatui::layout::Constraint::Length(area.height));
		//let box_height = crate::ui::Theme::DARK.max_error_box_height.min(area.height);
		//let box_area = area.centered(ratatui::layout::Constraint::Length(box_width), ratatui::layout::Constraint::Length(box_height));
		let title = Line::from(" Une erreur est survenue! ").centered().red();
		let instruction = Line::from(" Appuyez sur Entrée pour continuer. ").centered().gray();
		let block = ratatui::widgets::Block::bordered()
			.title(title)
			.title_bottom(instruction)
			.border_set(ratatui::symbols::border::THICK)
			.border_style(Style::new().red())
			.bg(ratatui::style::Color::Black);
		let tmp_inner = block.inner(tmp_area);

		let error_text = Paragraph::new(self._message.text())
			.wrap(ratatui::widgets::Wrap { trim: false });

		let h = error_text.line_count(tmp_inner.width);
		let box_height = match crate::ui::Theme::DARK.max_error_box_height {
			ScreenSize::Fill => area.width,
			ScreenSize::Length(l) => l,
			ScreenSize::Ratio(r) => (area.width as f32 * r).floor() as u16,
			ScreenSize::Fit {min, max} => {
				(if h > u16::MAX as usize {
					u16::MAX
				} else {
					(h as u16).saturating_add(2)
				}).clamp(min, max)
			},
		};
		let box_area = area.centered(ratatui::layout::Constraint::Max(box_width), ratatui::layout::Constraint::Max(box_height));
		let inner = block.inner(box_area);

		let max_scroll = h.saturating_sub(inner.height as usize);
		let scroll = self.scroll.min(max_scroll as u16);

		ratatui::widgets::Clear.render(box_area, buf);
		block.render(box_area, buf);
		error_text
			.scroll((0, scroll)).render(inner, buf);
	}
}
impl Screen for ErrorScreen<'_> {
	fn handle_event(&mut self, event: crate::ui::event::Event, _state: Arc<AppState>) -> Result<crate::ui::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				match key.code {
					crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Enter => { Ok(crate::ui::UpdateAction::Pop.one()) },
					crossterm::event::KeyCode::Up => {
						if self.scroll > 0 {
							self.scroll -= 1;
						}
						Ok(crate::ui::UpdateAction::Continue.one())
					},
					crossterm::event::KeyCode::Down => {
						self.scroll += 1;
						Ok(crate::ui::UpdateAction::Continue.one())
					},
					_ => { Ok(crate::ui::UpdateAction::Continue.one()) },
				}
			},
			_ => { Ok(crate::ui::UpdateAction::Continue.one()) },
		}
	}
}