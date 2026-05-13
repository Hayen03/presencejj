use std::{fmt::Debug, sync::{Arc, RwLock}};

use lazy_static::lazy_static;
use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::{Color, Style, Stylize}, symbols::border, text::{Line, Span, Text}, widgets::{Block, Clear, Paragraph, Widget, WidgetRef, Wrap}};
use ratatui_textarea::{CursorMove, Input, Key, TextArea};

use crate::{data::{BoolJustifie, adresse::{Adresse, CodePostal, Pays, Province, Ville}}, ui::{AppState, Screen, ScreenSize, UIError, UpdateAction, actions::UpdateActions, line_width, screens::ENTER_ESC_INSTRUCTIONS, str_width}};

pub type TextInputValidation = Arc<dyn (Fn(&str) -> bool) + Send + Sync>;
pub type InputAfterResult = Result<UpdateActions, UIError>;
pub type TextInputAfter = Box<dyn Fn(Option<&str>, Arc<AppState>) -> InputAfterResult + Send + Sync>;

lazy_static!{
	pub static ref INPUT_BLOCK_INSTRUCTION_WIDTH: u16 = (line_width(&ENTER_ESC_INSTRUCTIONS) as u16).saturating_add(2);
	pub static ref INPUT_SCREEN_BLOCK: Block<'static> = Block::bordered()
		.border_set(border::THICK)
		.border_style(Style::new().white())
		.title_bottom(ENTER_ESC_INSTRUCTIONS.clone())
		.bg(Color::Black);
	pub static ref INPUT_SCREEN_SUB_BLOCK: Block<'static> = Block::bordered()
		.border_set(border::ONE_EIGHTH_WIDE)
		.border_style(Style::new().gray());
	pub static ref INPUT_STYLE: Style = Style::new().white();
	pub static ref INPUT_PLACEHOLDER_STYLE: Style = Style::new().dark_gray().italic();
	pub static ref INPUT_CURSOR_STYLE: Style = Style::new().white().on_black().reversed().slow_blink();
	pub static ref INPUT_VALID_STYLE: Style = Style::new().green();
	pub static ref INPUT_INVALID_STYLE: Style = Style::new().red();

	pub static ref BOOL_JUSTIFIE_INPUT_BLOCK: Block<'static> = Block::bordered()
		.border_set(border::THICK)
		.border_style(Style::new().white())
		.title_bottom(ENTER_ESC_INSTRUCTIONS.clone())
		.bg(Color::Black);
	pub static ref ADDRESS_INPUT_BLOCK: Block<'static> = Block::bordered()
		.border_set(border::THICK)
		.border_style(Style::new().white())
		.title_bottom(ENTER_ESC_INSTRUCTIONS.clone())
		.bg(Color::Black);
}

pub struct LineInputScreen<'a> {
	input: RwLock<TextArea<'a>>,
	message: Text<'a>,
	title: Line<'a>,
	title_width: u16,
	prefered_width: u16,
	size: (ScreenSize, ScreenSize),
	placeholder_width: u16,
	validation: Option<TextInputValidation>,
	after: Option<TextInputAfter>,
	placeholder: Option<String>,
}
impl Default for LineInputScreen<'_> {
	fn default() -> Self {
		let mut text_area = TextArea::default();
		text_area.set_style(*INPUT_STYLE);
		text_area.set_cursor_style(*INPUT_CURSOR_STYLE);
		text_area.set_placeholder_style(*INPUT_PLACEHOLDER_STYLE);
		Self {
			input: RwLock::new(text_area),
			message: Text::default(),
			title: Line::default(),
			title_width: 0,
			prefered_width: 0,
			size: (ScreenSize::Fit { min: 12, max: u16::MAX }, ScreenSize::Fit { min: 3, max: u16::MAX }),
			placeholder_width: 0,
			validation: None,
			after: None,
			placeholder: None,
		}
	}
}
impl<'a> std::fmt::Debug for LineInputScreen<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("InputScreen")
			.field("message", &self.message)
			.field("title", &self.title)
			.field("title_width", &self.title_width)
			.field("prefered_width", &self.prefered_width)
			.field("size", &self.size)
			.field("placeholder_width", &self.placeholder_width)
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
		self.prefered_width = self.message.width() as u16;
		self
	}
	pub fn with_title(mut self, title: String) -> Self {
		let title = Line::from(format!(" {} ", title)).centered().white().bold();
		let title_width = (line_width(&title) as u16).saturating_add(2);
		self.title = title;
		self.title_width = title_width;
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
		self.placeholder_width = str_width(&placeholder) as u16;
		self.placeholder = Some(placeholder);
		self
	}
	pub fn with_size(mut self, width: ScreenSize, height: ScreenSize) -> Self {
		self.size = (width, height);
		self
	}
	pub fn with_value(self, value: String) -> Self {
		let mut lock = self.input.write().expect("Poisoned Lock");
		lock.clear();
		lock.insert_str(value);
		drop(lock);
		self
	}
	pub fn get_input(&self) -> String {
		self.input.read().expect("Poisoned Lock").lines()[0].clone()
	}
}
impl<'a> WidgetRef for LineInputScreen<'a> {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		let width = self.size.0.resolve(
			area.width,
			self.prefered_width
				.max(self.title_width)
				.max(self.placeholder_width)
				.max(*INPUT_BLOCK_INSTRUCTION_WIDTH)
				.saturating_add(2)
		);
		let tmp_area = area.centered(ratatui::layout::Constraint::Length(width), ratatui::layout::Constraint::Min(0));
		
		let main_block = INPUT_SCREEN_BLOCK.clone()
			.title_top(self.title.clone());
		let tmp_inner = main_block.inner(tmp_area);
		let block_height_diff = area.height - tmp_inner.height;

		let desc = Paragraph::new(self.message.clone())
			.wrap(Wrap { trim: false });
		let desc_height = desc.line_count(tmp_inner.width);

		let desired_height = 3 + desc_height as u16 + block_height_diff; // 3 for the input line and input border
		let height = self.size.1.resolve(area.height, desired_height);
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

		let input_block = INPUT_SCREEN_SUB_BLOCK.clone();
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
							self.input.write().expect("Poisoned Lock").set_style(*INPUT_VALID_STYLE);
						} else {
							self.input.write().expect("Poisoned Lock").set_style(*INPUT_INVALID_STYLE);
						}
					} else {
						self.input.write().expect("Poisoned Lock").set_style(*INPUT_STYLE);
					}
				}
				Ok(crate::ui::UpdateAction::Redraw.one())
			},
		}
	}
}

pub type BoolJustifieAfter = dyn Fn(Option<BoolJustifie>, Arc<AppState>) -> InputAfterResult + Send + Sync;
pub struct BoolJustifieInputScreen {
	reponse: Option<bool>,
	justification: TextArea<'static>,
	message: Text<'static>,
	title: Line<'static>,
	size: (ScreenSize, ScreenSize),
	title_width: u16,
	prefered_width: u16,
	placeholder_width: u16,
	focus_on_justification: bool,
	after: Option<Box<BoolJustifieAfter>>,
	input_lines: u16,
}
impl std::fmt::Debug for BoolJustifieInputScreen {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("BoolJustifieInputScreen")
			.field("reponse", &self.reponse)
			.field("message", &self.message)
			.field("title", &self.title)
			.field("size", &self.size)
			.field("title_width", &self.title_width)
			.field("prefered_width", &self.prefered_width)
			.field("placeholder_width", &self.placeholder_width)
			.field("focus_on_justification", &self.focus_on_justification)
			.field("after", &self.after.is_some())
			.finish()
	}
}
impl Default for BoolJustifieInputScreen {
	fn default() -> Self {
		let mut justification = TextArea::default();
		justification.set_cursor_style(Style::new().white().on_black().reversed().slow_blink());
		justification.set_wrap_mode(ratatui_textarea::WrapMode::Glyph);
		Self {
			reponse: None,
			justification,
			message: Text::default(),
			title: Line::default(),
			size: (ScreenSize::Fit { min: 20, max: u16::MAX }, ScreenSize::Fit { min: 6, max: u16::MAX }),
			title_width: 0,
			prefered_width: 0,
			placeholder_width: 0,
			focus_on_justification: false,
			after: None,
			input_lines: 2,
		}
	}
}
impl BoolJustifieInputScreen {
	pub fn with_message(mut self, message: String) -> Self {
		let width = str_width(&message);
		self.message = Text::from(message);
		self.prefered_width = width as u16;
		self
	}
	pub fn with_title(mut self, title: String) -> Self {
		let title = Line::from(format!(" {} ", title)).centered().white().bold();
		let title_width = (line_width(&title) as u16).saturating_add(2);
		self.title = title;
		self.title_width = title_width;
		self
	}
	pub fn with_reponse(mut self, reponse: Option<bool>) -> Self {
		self.reponse = reponse;
		self
	}
	pub fn with_justification(mut self, justification: Option<String>) -> Self {
		self.justification.clear();
		if let Some(justification) = justification {
			self.justification.insert_str(&justification);
		}
		self
	}
	pub fn with_size(mut self, width: ScreenSize, height: ScreenSize) -> Self {
		self.size = (width, height);
		self
	}
	pub fn with_after(mut self, after: Box<BoolJustifieAfter>) -> Self {
		self.after = Some(after);
		self
	}
	pub fn with_placeholder(mut self, placeholder: String) -> Self {
		self.justification.set_placeholder_text(placeholder.as_str());
		self.placeholder_width = str_width(&placeholder) as u16;
		self
	}
	pub fn with_input_lines(mut self, lines: u16) -> Self {
		self.input_lines = lines;
		self
	}

	pub fn get_value(&self) -> Option<BoolJustifie> {
		self.reponse.map(|r| {
			let justification = self.justification.lines().join("\n");
			let justification = String::from(justification.trim());
			let justification = if justification.is_empty() { None } else { Some(justification) };
			BoolJustifie { reponse: r, justification }
		})
	}

	fn stylize_reponse(&self, reponse: bool) -> Span<'static> {
		let s = if reponse { "Oui" } else { "Non" };
		let span = Span::from(s);
		let span = if !self.focus_on_justification && self.reponse == Some(reponse) {
			span.on_dark_gray().bold()
		} else {
			span
		};
		if self.reponse == Some(reponse) {
			if reponse {
				span.green()
			} else {
				span.red()
			}
		} else {
			span.gray()
		}
	}
}
impl WidgetRef for BoolJustifieInputScreen {
	fn render_ref(&self,area: Rect,buf: &mut Buffer) {
		let reponse_spans = vec![
			self.stylize_reponse(true),
			" | ".gray(),
			self.stylize_reponse(false),
		];
		let reponse_line = Line::from(reponse_spans);

		let width = self.size.0.resolve(area.width, [
			self.placeholder_width, 
			self.prefered_width, 
			self.title_width, 
			line_width(&reponse_line) as u16,
			*INPUT_BLOCK_INSTRUCTION_WIDTH,
		].iter().max().copied().unwrap_or(0));
		let tmp_area = area.centered(ratatui::layout::Constraint::Length(width), ratatui::layout::Constraint::Min(0));

		let main_block = BOOL_JUSTIFIE_INPUT_BLOCK.clone()
			.title_top(self.title.clone());
		let tmp_inner = main_block.inner(tmp_area);
		let block_height_diff = area.height - tmp_inner.height;

		let desc = Paragraph::new(self.message.clone())
			.wrap(Wrap { trim: false });
		let desc_height = desc.line_count(tmp_inner.width) as u16;

		let desired_height = desc_height + self.input_lines + 5; // 5 for main border, input border and reponse line
		let height = self.size.1.resolve(area.height, desired_height);
		let area = area.centered(ratatui::layout::Constraint::Length(width), ratatui::layout::Constraint::Length(height));
		let inner = main_block.inner(area);
		let desc_area = Rect {
			x: inner.x,
			y: inner.y,
			width: inner.width,
			height: desc_height,
		};
		let response_area = Rect {
			x: inner.x,
			y: inner.y + desc_height,
			width: inner.width,
			height: 1,
		};
		let input_area = Rect {
			x: inner.x,
			y: inner.y + desc_height + 1,
			width: inner.width,
			height: self.input_lines + 2, // 2 for response line and input border
		};
		let input_block = INPUT_SCREEN_SUB_BLOCK.clone();
		let input_inner = input_block.inner(input_area);

		Clear.render(area, buf);
		main_block.render(area, buf);
		desc.render(desc_area, buf);
		Paragraph::new(reponse_line).render(response_area, buf);
		input_block.render(input_area, buf);
		self.justification.render(input_inner, buf);
	}
}
impl Screen for BoolJustifieInputScreen {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<AppState>) -> Result<UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Enter => { // valider la reponse
						if let Some(after) = self.after.as_deref() {
							after(self.get_value(), state.clone())
						} else {
							Ok(UpdateAction::Pop.one())
						}
					},
					cte::KeyCode::Esc => { // just leave the input screen
						Ok(UpdateAction::Pop.one())
					},
					cte::KeyCode::Tab => { // switch focus between reponse and justification
						self.focus_on_justification = !self.focus_on_justification;
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Left | cte::KeyCode::Right if !self.focus_on_justification => {
						self.reponse = Some(self.reponse.unwrap_or(false) ^ true);
						Ok(UpdateAction::Redraw.one())
					},
					input if self.focus_on_justification => { // pass the input to the justification text area
						self.justification.input(Input::from(key));
						Ok(UpdateAction::Redraw.one())
					},
					_ => {
						Ok(UpdateAction::Continue.one())
					},
				}
				
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}

pub type AdresseInputAfter = dyn Fn(Option<Adresse>, Arc<AppState>) -> InputAfterResult + Send + Sync;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdresseField {
	Numero,
	Appartement,
	Rue,
	CodePostal,
	Ville,
	Province,
	Pays,
}
impl AdresseField {
	const VALUES: [AdresseField; 7] = [
		AdresseField::Numero,
		AdresseField::Appartement,
		AdresseField::Rue,
		AdresseField::CodePostal,
		AdresseField::Ville,
		AdresseField::Province,
		AdresseField::Pays,
	];
	fn label(&self) -> &'static str {
		match self {
			Self::Numero => "Numéro",
			Self::Appartement => "Appartement",
			Self::Rue => "Rue",
			Self::CodePostal => "Code Postal",
			Self::Ville => "Ville",
			Self::Province => "Province",
			Self::Pays => "Pays",
		}
	}
	fn next(&self) -> Self {
		match self {
			Self::Numero => Self::Appartement,
			Self::Appartement => Self::Rue,
			Self::Rue => Self::CodePostal,
			Self::CodePostal => Self::Ville,
			Self::Ville => Self::Province,
			Self::Province => Self::Pays,
			Self::Pays => Self::Numero,
		}
	}
	fn prev(&self) -> Self {
		match self {
			Self::Numero => Self::Pays,
			Self::Appartement => Self::Numero,
			Self::Rue => Self::Appartement,
			Self::CodePostal => Self::Rue,
			Self::Ville => Self::CodePostal,
			Self::Province => Self::Ville,
			Self::Pays => Self::Province,
		}
	}
}
pub struct AdresseInputScreen {
	numero: String,
	rue: String,
	app: String,
	code_postal: String,
	ville: String,
	pays: String,
	province: String,
	textarea: RwLock<TextArea<'static>>,
	sel: AdresseField,
	after: Option<Box<AdresseInputAfter>>,
	size: (ScreenSize, ScreenSize),
	title: Line<'static>,
	title_width: u16,
}
impl Debug for AdresseInputScreen {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("AdresseInputScreen")
			.field("numero", &self.numero)
			.field("rue", &self.rue)
			.field("appartement", &self.app)
			.field("code_postal", &self.code_postal)
			.field("ville", &self.ville)
			.field("pays", &self.pays)
			.field("province", &self.province)
			.field("sel", &self.sel)
			.field("after", &self.after.is_some())
			.field("size", &self.size)
			.field("title", &self.title)
			.field("title_width", &self.title_width)
			.finish()
	}
}
impl Default for AdresseInputScreen {
	fn default() -> Self {
		let mut textarea = TextArea::default();
		textarea.set_cursor_style(Style::new().white().on_black().reversed().slow_blink());
		Self {
			numero: String::new(),
			rue: String::new(),
			app: String::new(),
			code_postal: String::new(),
			ville: String::new(),
			pays: String::new(),
			province: String::new(),
			textarea: RwLock::new(textarea),
			sel: AdresseField::Numero,
			after: None,
			size: (ScreenSize::Fit { min: 24, max: u16::MAX }, ScreenSize::Length(2 + AdresseField::VALUES.len() as u16)),
			title: Line::default(),
			title_width: 0,
		}
	}
}
impl WidgetRef for AdresseInputScreen {
	fn render_ref(&self,area: Rect,buf: &mut Buffer) {
		let label_par = Paragraph::new(Text::from(AdresseField::VALUES.iter().map(|f| {
			Line::from(format!("{}: ", f.label())).white().bold()
		}).collect::<Vec<_>>()));
		let label_width = label_par.line_width() as u16;

		let prefered_width = (Self::PREFERED_FIELD_WIDTH + label_width + 3).max(self.title_width).max(*INPUT_BLOCK_INSTRUCTION_WIDTH); // 3 for border and gutter
		let width = self.size.0.resolve(area.width, prefered_width); // 2 for border and 1 for gutter
		let height = self.size.1.resolve(area.height, 2 + AdresseField::VALUES.len() as u16); // 2 for border
		let area = area.centered(Constraint::Length(width), Constraint::Length(height));
		

		let block = ADDRESS_INPUT_BLOCK.clone().title_top(self.title.clone());
		let inner = block.inner(area);
		let label_area = Rect {
			x: inner.x,
			y: inner.y,
			width: label_width,
			height: inner.height,
		};
		let field_width = area.width.saturating_sub(label_width).saturating_sub(1);

		Clear.render(area, buf);
		block.render(area, buf);
		label_par.render(label_area, buf);

		// render the fields
		for (i, field) in AdresseField::VALUES.iter().enumerate() {
			let field_area = Rect {
				x: inner.x + label_width + 1,
				y: inner.y + i as u16,
				width: field_width,
				height: 1,
			};
			if *field == self.sel { // render the text area for the selected field
				let mut lock = self.textarea.write().expect("Poisoned Lock");
				let val = lock.lines().join("\n");
				let style = style_field(*field, val.trim());
				lock.set_style(style);
				lock.render(field_area, buf);
			} else {
				let val = match field {
					AdresseField::Numero => &self.numero,
					AdresseField::Appartement => &self.app,
					AdresseField::Rue => &self.rue,
					AdresseField::CodePostal => &self.code_postal,
					AdresseField::Ville => &self.ville,
					AdresseField::Pays => &self.pays,
					AdresseField::Province => &self.province,
				};
				let style = style_field(*field, val.trim());
				let par = Paragraph::new(Text::from(val.as_str()).style(style));
				par.render(field_area, buf);
			}
		}

	}
}

impl AdresseInputScreen {
	const PREFERED_FIELD_WIDTH: u16 = 12;
	pub fn with_title(mut self, title: String) -> Self {
		let title = Line::from(format!(" {} ", title)).centered().white().bold();
		let title_width = (line_width(&title) as u16).saturating_add(2);
		self.title = title;
		self.title_width = title_width;
		self
	}
	pub fn with_after(mut self, after: Box<AdresseInputAfter>) -> Self {
		self.after = Some(after);
		self
	}
	pub fn with_value(mut self, adresse: Adresse) -> Self {
		self.numero = adresse.numero.map(|n| n.to_string()).unwrap_or_default();
		self.rue = adresse.rue.unwrap_or_default();
		self.app = adresse.appartement.map(|a| a.to_string()).unwrap_or_default();
		self.code_postal = adresse.code_postal.map(|c| c.to_string()).unwrap_or_default();
		self.ville = adresse.ville.map(|v| v.to_string()).unwrap_or_default();
		self.pays = adresse.pays.map(|p| p.to_string()).unwrap_or_default();
		self.province = adresse.province.map(|p| p.to_string()).unwrap_or_default();
		self.update_textarea();
		self
	}
	pub fn with_size(mut self, width: ScreenSize, height: ScreenSize) -> Self {
		self.size = (width, height);
		self
	}

	pub fn get_value(&self) -> Option<Adresse> {
		let numero = if self.sel == AdresseField::Numero {
			self.textarea.read().expect("Poisoned Lock").lines().join("\n")
		} else {
			self.numero.clone()
		};
		let app = if self.sel == AdresseField::Appartement {
			self.textarea.read().expect("Poisoned Lock").lines().join("\n")
		} else {
			self.app.clone()
		};
		let rue = if self.sel == AdresseField::Rue {
			self.textarea.read().expect("Poisoned Lock").lines().join("\n")
		} else {
			self.rue.clone()
		};
		let code_postal = if self.sel == AdresseField::CodePostal {
			self.textarea.read().expect("Poisoned Lock").lines().join("\n")
		} else {
			self.code_postal.clone()
		};
		let ville = if self.sel == AdresseField::Ville {
			self.textarea.read().expect("Poisoned Lock").lines().join("\n")
		} else {
			self.ville.clone()
		};
		let pays = if self.sel == AdresseField::Pays {
			self.textarea.read().expect("Poisoned Lock").lines().join("\n")
		} else {
			self.pays.clone()
		};
		let province = if self.sel == AdresseField::Province {
			self.textarea.read().expect("Poisoned Lock").lines().join("\n")
		} else {
			self.province.clone()
		};
		let numero = if numero.trim().is_empty() { None } else {
			match numero.trim().parse::<i32>() {
				Ok(n) => Some(n),
				Err(_) => return None, // invalid input
			}
		};
		let app = if app.trim().is_empty() { None } else {
			match app.trim().parse::<i32>() {
				Ok(n) => Some(n),
				Err(_) => return None, // invalid input
			}
		};
		let code_postal = if code_postal.trim().is_empty() { None } else {
			match code_postal.trim().parse::<CodePostal>() {
				Ok(c) => Some(c),
				Err(_) => return None, // invalid input
			}
		};
		let rue = if rue.trim().is_empty() { None } else { Some(rue.trim().to_string()) };
		let ville = if ville.trim().is_empty() { None } else { Some(Ville::from(ville.trim())) };
		let pays = if pays.trim().is_empty() { None } else { Some(Pays::from(pays.trim())) };
		let province = if province.trim().is_empty() { None } else { Some(Province::from(province.trim())) };
		Some(Adresse {
			numero,
			rue,
			appartement: app,
			code_postal,
			ville,
			pays,
			province,
		})
	}

	fn update_textarea(&self) {
		let val = match self.sel {
			AdresseField::Numero => &self.numero,
			AdresseField::Appartement => &self.app,
			AdresseField::Rue => &self.rue,
			AdresseField::CodePostal => &self.code_postal,
			AdresseField::Ville => &self.ville,
			AdresseField::Pays => &self.pays,
			AdresseField::Province => &self.province,
		};
		let mut lock = self.textarea.write().expect("Poisoned Lock");
		lock.clear();
		lock.insert_str(val);
		lock.move_cursor(CursorMove::End);
	}
	fn record_textarea(&mut self) {
		let val = self.textarea.read().expect("Poisoned Lock").lines().join("\n");
		match self.sel {
			AdresseField::Numero => self.numero = val,
			AdresseField::Appartement => self.app = val,
			AdresseField::Rue => self.rue = val,
			AdresseField::CodePostal => self.code_postal = val,
			AdresseField::Ville => self.ville = val,
			AdresseField::Pays => self.pays = val,
			AdresseField::Province => self.province = val,
		}
	}
	fn next_field(&mut self) {
		self.record_textarea();
		self.sel = self.sel.next();
		self.update_textarea();
	}
	fn prev_field(&mut self) {
		self.record_textarea();
		self.sel = self.sel.prev();
		self.update_textarea();
	}
}
impl Screen for AdresseInputScreen {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<AppState>) -> Result<UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Enter => {
						self.record_textarea();
						if let Some(after) = self.after.as_deref() {
							after(self.get_value(), state.clone())
						} else {
							Ok(UpdateAction::Pop.one())
						}
					},
					cte::KeyCode::Esc => {
						Ok(UpdateAction::Pop.one())
					},
					cte::KeyCode::Tab => {
						self.next_field();
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Up => {
						self.prev_field();
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Down => {
						self.next_field();
						Ok(UpdateAction::Redraw.one())
					},
					input => {
						self.textarea.write().expect("Poisoned Lock").input(Input::from(key));
						Ok(UpdateAction::Redraw.one())
					},
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}

fn style_field(field: AdresseField, value: &str) -> Style {
	match field {
		AdresseField::Numero => {
			if value.parse::<i32>().is_ok() {
				Style::new().green()
			} else {
				Style::new().red()
			}
		},
		AdresseField::Appartement => {
			if value.parse::<i32>().is_ok() {
				Style::new().green()
			} else {
				Style::new().red()
			}
		},
		AdresseField::CodePostal => {
			if value.parse::<CodePostal>().is_ok() {
				Style::new().green()
			} else {
				Style::new().red()
			}
		},
		_ => Style::new().gray(),
	}
}