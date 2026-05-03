use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style, Stylize}, symbols::border, text::{Line, Text}, widgets::{Block, Clear, Paragraph, Widget, WidgetRef, Wrap}};
use unicode_segmentation::UnicodeSegmentation;
// use unicode_width::UnicodeWidthStr;

use crate::ui::{Screen, ScreenSize};


#[derive(Debug)]
pub struct InfoScreen<'a> {
	title: String,
	text: Text<'a>,
	scroll: u16,
	size: (ScreenSize, ScreenSize),
	prefered_width: u16,
}
impl<'a> InfoScreen<'a> {
	pub fn new(title: String, text: Text<'a>) -> Self {
		let prefered_width = text.lines.iter().map(|line| line.spans.iter().map(|span| span.width()).sum()).max().unwrap_or(0) as u16 + 2;
		InfoScreen { title, text, scroll: 0, size: (ScreenSize::Fill, ScreenSize::Fill), prefered_width }
	}
	pub fn with_size(mut self, width: ScreenSize, height: ScreenSize) -> Self {
		self.size = (width, height);
		self
	}
}
impl<'a> WidgetRef for InfoScreen<'a> {
	fn render_ref(&self, rect: Rect, buf: &mut Buffer) {
		let max_width = match self.size.0 {
			ScreenSize::Fill => rect.width,
			ScreenSize::Length(l) => l,
			ScreenSize::Ratio(r) => (rect.width as f32 * r).floor() as u16,
			ScreenSize::Fit {min, max} => { (self.prefered_width + 2).clamp(min, max) },
		}.min(rect.width);
		let tmp_area = rect.centered(ratatui::layout::Constraint::Max(max_width), ratatui::layout::Constraint::Length(rect.height));

		let block = Block::bordered()
			.title_top(Line::from(format!(" {} ", self.title)).centered().yellow())
			.title_bottom(Line::from(" Appuyez sur Entrée pour fermer ").centered().gray())
			.border_set(border::THICK)
			.border_style(Style::new().yellow())
			.bg(Color::Black);
		let tmp_inner = block.inner(tmp_area);

		let paragraph = Paragraph::new(self.text.clone())
			.wrap(Wrap { trim: false });
		let h = paragraph.line_count(tmp_inner.width);

		let max_height = match self.size.1 {
			ScreenSize::Fill => rect.height,
			ScreenSize::Length(l) => l,
			ScreenSize::Ratio(r) => (rect.height as f32 * r).floor() as u16,
			ScreenSize::Fit {min, max} => {
				if h > u16::MAX as usize {
					u16::MAX
				} else {
					(h as u16).saturating_add(2).clamp(min, max)
				}
			},
		}.min(rect.height);
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