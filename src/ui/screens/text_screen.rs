use std::{cell::Cell, sync::{Arc, Mutex}};

use lazy_static::lazy_static;
use ratatui::{style::Stylize, text::{Line, Text}, widgets::{Block, Clear, Paragraph, Widget, WidgetRef}};

use crate::ui::{Screen, UpdateAction, screens::ENTER_ESC_INSTRUCTIONS};

lazy_static!{
	pub static ref TEXT_SCREEN_BLOCK: Block<'static> = Block::bordered()
		.border_set(ratatui::symbols::border::THICK)
		.border_style(ratatui::style::Style::new().white())
		.bg(ratatui::style::Color::Black);
}

#[derive(Debug)]
pub struct TextScreen<'a> {
	title: Line<'a>,
	text: Text<'a>,
	scroll: Cell<u16>,
	cancel_hook: Arc<Mutex<bool>>,
}
impl<'a> TextScreen<'a> {
	pub fn new(title: Line<'a>, text: Text<'a>) -> Self {
		Self { title, text, scroll: Cell::new(0), cancel_hook: Arc::new(Mutex::new(false)) }
	}
	pub fn get_cancel_hook(&self) -> Arc<Mutex<bool>> {
		self.cancel_hook.clone()
	}
}
impl<'a> WidgetRef for TextScreen<'a> {
	fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
		Clear.render(area, buf);
		let block = TEXT_SCREEN_BLOCK.clone()
			.title_top(self.title.clone())
			.title_bottom(ENTER_ESC_INSTRUCTIONS.clone());
		let inner = block.inner(area);
		block.render(area, buf);

		let par = Paragraph::new(self.text.clone());
		let h = par.line_count(inner.width);
		let max_scroll = h.saturating_sub(inner.height as usize) as u16;
		let scroll = self.scroll.get().min(max_scroll);
		let par = par.scroll((scroll, 0));
		par.render(inner, buf);
	}
}
impl<'a> Screen for TextScreen<'a> {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: std::sync::Arc<crate::ui::AppState>) -> Result<crate::ui::actions::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Up => {
						let new_scroll = self.scroll.get().saturating_sub(1);
						self.scroll.set(new_scroll);
						Ok(UpdateAction::Redraw.one())
					}
					cte::KeyCode::Down => {
						let new_scroll = self.scroll.get().saturating_add(1);
						self.scroll.set(new_scroll);
						Ok(UpdateAction::Redraw.one())
					}
					cte::KeyCode::Enter => {
						Ok(UpdateAction::Pop.one())
					}
					cte::KeyCode::Esc => {
						*self.cancel_hook.lock().expect("Poisoned Lock") = true; // cancel the action, but we will not wait for any thread using this hook to finish
						Ok(UpdateAction::Pop.one())
					},
					_ => Ok(UpdateAction::Continue.one()),
				}
			}
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}
