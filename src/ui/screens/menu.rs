use std::{fmt::Debug, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::Rect, style::Style, symbols::border, text::Line, widgets::{Block, WidgetRef}};

use crate::ui::{AppState, Screen, ScreenSize, UIError, UpdateAction, actions::UpdateActions, event::{self}};
use crate::ui::actions;
use unicode_segmentation::UnicodeSegmentation;

pub struct MenuItem<Ids> where Ids: ToString + Debug {
	pub id: Ids,
	pub action: Box<actions::Action>,
}
impl<Ids> Debug for MenuItem<Ids> where Ids: ToString + Debug {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MenuItem")
			.field("id", &self.id.to_string())
			.finish()
	}
}

#[derive(Debug)]
pub struct Menu<'a, Ids> where Ids: ToString + Debug {
	items: Box<[MenuItem<Ids>]>,
	selected: usize,
	title: String,
	size: (ScreenSize, ScreenSize),
	widget: ratatui::widgets::List<'a>,
	_block: Block<'a>,
	fit_width: u16,
}
impl<'a, Ids> Menu<'a, Ids> where Ids: ToString + Debug {
	pub fn new(items: Box<[MenuItem<Ids>]>) -> Self {
		let fit_width = items.iter().map(|item| item.id.to_string().graphemes(true).count() as u16).max().unwrap_or(0);
		let block = Block::bordered()
			.title(" Menu ")
			.border_set(border::THICK);
		let widget = ratatui::widgets::List::new(items.iter().map(|item| Line::from(item.id.to_string())))
			.style(Style::new().white())
			.highlight_style(Style::new().yellow().on_dark_gray())
			.block(block.clone());
		Menu { items, selected: 0, title: String::new(), widget, _block: block, size: (ScreenSize::Fill, ScreenSize::Fill), fit_width }
	}
	pub fn with_title(mut self, title: String) -> Self {
		self.title = title;
		self
	}
	pub fn with_size(mut self, width: ScreenSize, height: ScreenSize) -> Self {
		self.size = (width, height);
		self
	}
	pub fn get_selected(&self) -> Option<&MenuItem<Ids>> {
		if self.items.is_empty() {
			None
		} else {
			Some(&self.items[self.selected])
		}
	}
	pub fn select(&mut self, index: usize) -> bool {
		if index < self.items.len() {
			self.selected = index;
			true
		} else {
			false
		}
	}
	pub fn next(&mut self) {
		if !self.items.is_empty() {
			self.selected = (self.selected + 1) % self.items.len();
		}
	}
	pub fn previous(&mut self) {
		if !self.items.is_empty() {
			self.selected = (self.selected + self.items.len() - 1) % self.items.len();
		}
	}
	pub fn block(mut self, block: Block<'a>) -> Self {
		self._block = block;
		self.widget = self.widget.block(self._block.clone());
		self
	}

	fn handle_key(&mut self, event: KeyEvent, state: Arc<AppState>) -> Result<UpdateActions, UIError> {
		match event.code {
			KeyCode::Up => {
				self.previous();
				Ok(UpdateAction::Continue.one())
			},
			KeyCode::Down => {
				self.next();
				Ok(UpdateAction::Continue.one())
			},
			KeyCode::Esc => Ok(UpdateAction::Quit.one()),
			KeyCode::Enter => {
				if let Some(item) = self.get_selected() {
					(item.action)(state)
				} else {
					Ok(UpdateAction::Continue.one())
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}
impl<'a, Ids> WidgetRef for Menu<'a, Ids> where Ids: ToString + Debug {
	fn render_ref(&self, area: Rect, buf: &mut Buffer)
		where
			Self: Sized {
		self.render_focus(area, buf, true);
	}
}
impl<'a, Ids> Screen for Menu<'a, Ids> where Ids: ToString + Debug {
	fn handle_event(&mut self, event: event::Event, state: Arc<AppState>) -> Result<UpdateActions, UIError> {
		match event  {
			event::Event::Key(ke) => self.handle_key(ke, state),
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
	fn render_focus(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, focus: bool) {
		let area = {
			let width = match self.size.0 {
				ScreenSize::Fill => area.width,
				ScreenSize::Length(l) => l,
				ScreenSize::Ratio(r) => (area.width as f32 * r).floor() as u16,
				ScreenSize::Fit {min, max} => {
					(self.fit_width + 4).clamp(min, max)
				},
			};
			let height = match self.size.1 {
				ScreenSize::Fill => area.height,
				ScreenSize::Length(l) => l,
				ScreenSize::Ratio(r) => (area.height as f32 * r).floor() as u16,
				ScreenSize::Fit {min, max} => {
					(self.widget.len() as u16 + 2).clamp(min, max)
				},
			};
			area.centered(ratatui::layout::Constraint::Max(width), ratatui::layout::Constraint::Max(height))
		};
		let mut state = ratatui::widgets::ListState::default().with_selected(Some(self.selected));
		let widget = if focus {
			self.widget.clone()
		} else {
			self.widget.clone()
				.style(Style::new().gray())
				.highlight_style(Style::new().gray().on_dark_gray())
				.block(self._block.clone()
					.border_style(Style::new().gray()))
		};
		ratatui::widgets::StatefulWidget::render(&widget, area, buf, &mut state);
	}
}