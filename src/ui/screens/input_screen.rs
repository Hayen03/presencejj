use std::sync::{Arc, RwLock};

use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style, Stylize}, symbols::border, text::{Line, Text}, widgets::{Block, Clear, Paragraph, Widget, WidgetRef, Wrap}};
use ratatui_textarea::{Input, Key, TextArea};

use crate::ui::{AppState, Screen, UIError, actions::UpdateActions};

static MAX_WIDTH: u16 = 80;

pub type TextInputValidation = Box<dyn Fn(&str) -> bool>;
pub type TextInputAfterResult = Result<UpdateActions, UIError>;
pub type TextInputAfter = Box<dyn Fn(Option<&str>, Arc<AppState>) -> TextInputAfterResult>;
#[derive(Default)]
pub struct LineInputScreen<'a> {
	input: RwLock<TextArea<'a>>,
	message: Text<'a>,
	title: String,
	validation: Option<TextInputValidation>,
	after: Option<TextInputAfter>,
	placeholder: Option<String>,
}
impl<'a> std::fmt::Debug for LineInputScreen<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("InputScreen")
			.field("message", &self.message)
			.field("title", &self.title)
			.field("validation", &self.validation.is_some())
			.field("after", &self.after.is_some())
			.field("placeholder", &self.placeholder)
			.field("input", &self.input)
			.finish()
	}
}
impl<'a> LineInputScreen<'a> {
	pub fn with_message(mut self, message: Text<'a>) -> Self {
		self.message = message;
		self
	}
	pub fn with_title(mut self, title: String) -> Self {
		self.title = title;
		self
	}
	pub fn with_validation(mut self, validation: TextInputValidation) -> Self {
		self.validation = Some(validation);
		self
	}
	pub fn with_after(mut self, after: TextInputAfter) -> Self {
		self.after = Some(after);
		self
	}
	pub fn with_placeholder(mut self, placeholder: String) -> Self {
		self.input.write().expect("Poisoned Lock").set_placeholder_text(placeholder.as_str());
		self.placeholder = Some(placeholder);
		self
	}
	pub fn get_input(&self) -> String {
		self.input.read().expect("Poisoned Lock").lines()[0].clone()
	}
}
impl<'a> WidgetRef for LineInputScreen<'a> {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		let width = area.width.min(MAX_WIDTH);
		
		let main_block = Block::bordered()
			.border_set(border::THICK)
			.border_style(Style::new().white())
			.title_top(Line::from(format!(" {} ", self.title)).centered().white())
			.bg(Color::Black);
		let tmp_inner = main_block.inner(area);
		let block_height_diff = area.height - tmp_inner.height;

		let desc = Paragraph::new(self.message.clone())
			.wrap(Wrap { trim: false });
		let desc_height = desc.line_count(tmp_inner.width);

		let desired_height = 3 + desc_height as u16 + block_height_diff; // 3 for the input line and input border
		let height = desired_height.min(area.height);
		let area = area.centered(ratatui::layout::Constraint::Length(width), ratatui::layout::Constraint::Length(height));
		let inner = main_block.inner(area);
		let desc_height = height.saturating_sub(3 + block_height_diff);
		let desc_area = Rect {
			x: inner.x,
			y: inner.y,
			width: inner.width,
			height: desc_height,
		};
		let input_area = Rect {
			x: inner.x,
			y: inner.y + desc_height,
			width: inner.width,
			height: 3,
		};

		Clear.render(area, buf);
		main_block.render(area, buf);
		desc.render(desc_area, buf);

		let input_block = Block::bordered()
			.border_set(border::ONE_EIGHTH_WIDE)
			.border_style(Style::new().gray());
		let input_inner = input_block.inner(input_area);
		input_block.render(input_area, buf);
		self.input.read().expect("Poisoned Lock").render(input_inner, buf);
	}
}
impl<'a> Screen for LineInputScreen<'a> {
	fn handle_event(&mut self, event: crate::ui::event::Event, _state: std::sync::Arc<crate::ui::AppState>) -> Result<crate::ui::UpdateActions, UIError> {

		match event.into() {
			Input { key: Key::Esc, .. } => {
				// run the after function
				if let Some(after) = self.after.as_deref() {
					let res = after(None, _state.clone());
					if let Err(err) = res {
						Ok(crate::ui::UpdateAction::ErrorReplace(Box::new(err)).one())
					} else {
						Ok(crate::ui::UpdateAction::Pop.one())
					}
				} else {
					Ok(crate::ui::UpdateAction::Pop.one())
				}
			},
			Input { key: Key::Enter, ..} => {
				// run the after function
				if let Some(after) = self.after.as_deref() {
					let res = after(Some(self.get_input().as_str()), _state.clone());
					res
				} else {
					Ok(crate::ui::UpdateAction::Pop.one())
				}
			},
			input => {
				if self.input.write().expect("Poisoned Lock").input(input) {
					// if there is a validation validate and change the color of the input text
					if let Some(validation) = self.validation.as_deref() {
						if validation(self.get_input().as_str()) {
							self.input.write().expect("Poisoned Lock").set_style(Style::new().green());
						} else {
							self.input.write().expect("Poisoned Lock").set_style(Style::new().red());
						}
					} else {
						self.input.write().expect("Poisoned Lock").set_style(Style::new().white());
					}
				}
				Ok(crate::ui::UpdateAction::Continue.one())
			},
		}
	}
}
