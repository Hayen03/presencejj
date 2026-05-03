use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style, Stylize}, symbols::border, text::{Line, Text}, widgets::{Block, Clear, Paragraph, Widget, WidgetRef, Wrap}};

use crate::ui::Screen;


#[derive(Debug)]
pub struct InfoScreen<'a> {
	title: String,
	text: Text<'a>,
	scroll: u16,
	max_size: (u16, u16),
}
impl<'a> InfoScreen<'a> {
	pub fn new(title: String, text: Text<'a>, max_size: (u16, u16)) -> Self {
		InfoScreen { title, text, scroll: 0, max_size }
	}
}
impl<'a> WidgetRef for InfoScreen<'a> {
	fn render_ref(&self, rect: Rect, buf: &mut Buffer) {
		let area = rect.centered(ratatui::layout::Constraint::Max(self.max_size.0), ratatui::layout::Constraint::Max(self.max_size.1));

		Clear.render(area, buf);

		let block = Block::bordered()
			.title_top(Line::from(format!(" {} ", self.title)).centered().yellow())
			.title_bottom(Line::from(" Appuyez sur Entrée pour fermer ").centered().gray())
			.border_set(border::THICK)
			.border_style(Style::new().yellow())
			.bg(Color::Black);
		let inner = block.inner(area);
		block.render(area, buf);

		let paragraph = Paragraph::new(self.text.clone())
			.wrap(Wrap { trim: false });
		let h = paragraph.line_count(inner.width);
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