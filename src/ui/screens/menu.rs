use std::{fmt::Debug, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style, Stylize}, symbols::border, text::Line, widgets::{Block, Clear, Widget, WidgetRef}};

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

// TODO refactor this to be prettier

pub struct Menu<'a, Ids> where Ids: ToString + Debug {
	items: Box<[MenuItem<Ids>]>,
	selected: usize,
	title: Option<String>,
	size: (ScreenSize, ScreenSize),
	widget: ratatui::widgets::List<'a>,
	_block: Block<'a>,
	fit_width: u16,
	title_width: u16,
	cancel_action: Option<Box<actions::Action>>,
}
impl<Ids> Debug for Menu<'_, Ids> where Ids: ToString + Debug {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Menu")
			.field("items", &self.items)
			.field("selected", &self.selected)
			.field("title", &self.title)
			.field("size", &self.size)
			.field("cancel_action", &self.cancel_action.is_some())
			.finish()
	}
}
impl<'a, Ids> Menu<'a, Ids> where Ids: ToString + Debug {
	pub fn new(items: Box<[MenuItem<Ids>]>) -> Self {
		let fit_width = items.iter().map(|item| item.id.to_string().graphemes(true).count() as u16).max().unwrap_or(0);
		let block = Block::bordered()
			.title_top(" Menu ")
			.border_set(border::THICK)
			.bg(Color::Black);
		let widget = ratatui::widgets::List::new(items.iter().map(|item| Line::from(item.id.to_string())))
			.style(Style::new().white())
			.highlight_style(Style::new().yellow().on_dark_gray())
			.block(block.clone());
		Menu { items, selected: 0, title: None, widget, _block: block, size: (ScreenSize::Fill, ScreenSize::Fill), fit_width, title_width: 0, cancel_action: None }
	}
	pub fn with_title(mut self, title: String) -> Self {
		let ln = Line::from(format!(" {title} ")).centered();
		self.title_width = ln.spans.iter().map(|s| s.width()).sum::<usize>() as u16;
		// remove previous title if exists
		self._block = self._block.title_top("").title_top(ln);
		self.title = Some(title);
		self.widget = self.widget.block(self._block.clone());
		self
	}
	pub fn with_size(mut self, width: ScreenSize, height: ScreenSize) -> Self {
		self.size = (width, height);
		self
	}
	pub fn with_cancel_action(mut self, action: Box<actions::Action>) -> Self {
		self.cancel_action = Some(action);
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
			KeyCode::Esc => {
				if let Some(action) = &self.cancel_action {
					action(state)
				} else {
					Ok(UpdateAction::Pop.one())
				}
			},
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
			let width = self.size.0.resolve(area.width, self.fit_width.max(self.title_width).saturating_add(4));
			let height = self.size.1.resolve(area.height, (self.widget.len() as u16).saturating_add(2));
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
		Clear.render(area, buf);
		ratatui::widgets::StatefulWidget::render(&widget, area, buf, &mut state);
	}
}