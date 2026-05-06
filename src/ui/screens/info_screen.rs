use lazy_static::lazy_static;
use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style, Stylize}, symbols::border, text::{Line, Text}, widgets::{Block, Clear, Paragraph, Widget, WidgetRef, Wrap}};
// use unicode_width::UnicodeWidthStr;

use crate::ui::{Screen, ScreenSize, line_width};

lazy_static!{
	pub static ref INFO_SCREEN_INSTRUCTIONS: Line<'static> = Line::from(vec![
		" Appuyez sur ".gray(),
		"Entrée".light_blue().bold(),
		" pour continuer. ".gray(),
	]).centered();
	pub static ref INFO_SCREEN_INSTRUCTION_WIDTH: u16 = (line_width(&INFO_SCREEN_INSTRUCTIONS) as u16).saturating_add(2);
	pub static ref INFO_SCREEN_BLOCK: Block<'static> = Block::bordered()
		.title_bottom(INFO_SCREEN_INSTRUCTIONS.clone())
		.border_set(border::THICK)
		.border_style(Style::new().yellow())
		.bg(Color::Black);
}

#[derive(Debug)]
pub struct InfoScreen<'a> {
	title: Line<'a>,
	title_width: u16,
	text: Text<'a>,
	scroll: u16,
	size: (ScreenSize, ScreenSize),
	prefered_width: u16,
}
impl<'a> InfoScreen<'a> {
	pub fn new(title: String, text: Text<'a>) -> Self {
		let title = Line::from(format!(" {} ", title)).centered().yellow();
		let title_width = (line_width(&title) as u16).saturating_add(2);
		let prefered_width = (text.lines.iter().map(|line| line_width(line)).max().unwrap_or(0) as u16).saturating_add(2);
		InfoScreen { title, text, scroll: 0, size: (ScreenSize::Fill, ScreenSize::Fill), prefered_width, title_width }
	}
	pub fn with_size(mut self, width: ScreenSize, height: ScreenSize) -> Self {
		self.size = (width, height);
		self
	}
}
impl<'a> WidgetRef for InfoScreen<'a> {
	fn render_ref(&self, rect: Rect, buf: &mut Buffer) {
		let max_width = self.size.0.resolve(rect.width, (self.prefered_width.saturating_add(2)).max(self.title_width));
		let tmp_area = rect.centered(ratatui::layout::Constraint::Max(max_width), ratatui::layout::Constraint::Length(rect.height));

		let block = INFO_SCREEN_BLOCK.clone().title_top(self.title.clone());
		let tmp_inner = block.inner(tmp_area);

		let paragraph = Paragraph::new(self.text.clone())
			.wrap(Wrap { trim: false });
		let h = paragraph.line_count(tmp_inner.width);

		let prefered_height = if h > u16::MAX as usize {
			u16::MAX
		} else {
			(h as u16).saturating_add(2)
		};
		let max_height = self.size.1.resolve(rect.height, prefered_height);
		let area = rect.centered(ratatui::layout::Constraint::Max(max_width), ratatui::layout::Constraint::Max(max_height));
		let inner = block.inner(area);
		Clear.render(area, buf);
		block.render(area, buf);

		let max_scroll = h.saturating_sub(inner.height as usize);
		let scroll = self.scroll.min(max_scroll as u16);
		paragraph.scroll((scroll, 0)).render(inner, buf);
	}
}
impl<'a> Screen for InfoScreen<'a> {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: std::sync::Arc<crate::ui::AppState>) -> Result<crate::ui::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event::KeyCode;
				match key.code {
					KeyCode::Enter => { Ok(crate::ui::UpdateAction::Pop.one()) },
					KeyCode::Esc => { Ok(crate::ui::UpdateAction::Pop.one()) },
					KeyCode::Up => {
						self.scroll = self.scroll.saturating_sub(1);
						Ok(crate::ui::UpdateAction::Continue.one())
					},
					KeyCode::Down => {
						self.scroll = self.scroll.saturating_add(1);
						Ok(crate::ui::UpdateAction::Continue.one())
					},
					_ => { Ok(crate::ui::UpdateAction::Continue.one()) },
				}
			},
			crate::ui::event::Event::Mouse(mouse) => {
				use crossterm::event::MouseEventKind;
				match mouse.kind {
					MouseEventKind::ScrollUp => {
						self.scroll = self.scroll.saturating_sub(1);
						Ok(crate::ui::UpdateAction::Continue.one())
					},
					MouseEventKind::ScrollDown => {
						self.scroll = self.scroll.saturating_add(1);
						Ok(crate::ui::UpdateAction::Continue.one())
					},
					_ => { Ok(crate::ui::UpdateAction::Continue.one()) },
				}
			},
			_ => { Ok(crate::ui::UpdateAction::Continue.one()) },
		}
	}
}