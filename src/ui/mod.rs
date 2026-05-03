use std::{any::Any, collections::VecDeque, fmt::Debug, path::{Path, PathBuf}, sync::{Arc, RwLock, mpsc::RecvError}};

use ratatui::{Frame, style::Color, text::Text};

use crate::{extract::ExtractError, groupes::{RegError, comptes::{CompteErr, CompteID, CompteReg, NULL_COMPTE}, groupes::{GroupeID, GroupeReg, NULL_GROUPE}, membres::{MembreID, MembreReg, NULL_MEMBRE}}, ui::{actions::UpdateActions, event::Event}};

pub mod app;
pub mod tui;
pub mod event;
pub mod actions;
pub mod screens;

#[derive(Debug)]
pub enum TextInputError {
	InvalidInput(String),
	NoInput,
}
impl std::fmt::Display for TextInputError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TextInputError::InvalidInput(desc) => write!(f, "Invalid input: {}", desc),
			TextInputError::NoInput => write!(f, "No input provided"),
		}
	}
}
impl std::error::Error for TextInputError {}

#[derive(Debug)]
pub enum UIError {
	IO{src: std::io::Error},
	Event {src: event::EventError },
	Runtime { src: Box<dyn Any + Send> },
	Extract { src: ExtractError },
	GroupeRegistry { src: RegError<GroupeID> },
	CompteRegistry { src: RegError<CompteID> },
	MembreRegistry { src: RegError<MembreID> },
	CancelAction{ desc: String },
	Compte { src: CompteErr },
	Input { src: TextInputError },
}
impl From<std::io::Error> for UIError {
	fn from(src: std::io::Error) -> Self {
		UIError::IO { src }
	}
}
impl From<tui::TuiError> for UIError {
	fn from(src: tui::TuiError) -> Self {
		match src {
			tui::TuiError::IOError { src } => UIError::IO { src },
		}
	}
}
impl From<event::EventError> for UIError {
	fn from(value: event::EventError) -> Self {
		UIError::Event { src: value }
	}
}
impl From<ExtractError> for UIError {
	fn from(value: ExtractError) -> Self {
		UIError::Extract { src: value }
	}
}
impl From<office::Error> for UIError {
	fn from(value: office::Error) -> Self {
		UIError::Extract { src: ExtractError::OfficeError { src: value } }
	}
}
impl From<RegError<GroupeID>> for UIError {
	fn from(value: RegError<GroupeID>) -> Self {
		UIError::GroupeRegistry { src: value }
	}
}
impl From<RegError<CompteID>> for UIError {
	fn from(value: RegError<CompteID>) -> Self {
		UIError::CompteRegistry { src: value }
	}
}
impl From<RegError<MembreID>> for UIError {
	fn from(value: RegError<MembreID>) -> Self {
		UIError::MembreRegistry { src: value }
	}
}
impl From<CompteErr> for UIError {
	fn from(value: CompteErr) -> Self {
		UIError::Compte { src: value }
	}
}
impl From<TextInputError> for UIError {
	fn from(value: TextInputError) -> Self {
		UIError::Input { src: value }
	}
}
impl std::fmt::Display for UIError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			UIError::IO { src } => write!(f, "IO error: {}", src),
			UIError::Event { src } => write!(f, "Event error: {}", src),
			UIError::Runtime { src } => {
				// try to downcast the src to a string, if it is a string, print it, otherwise print the debug representation of the src
				if let Some(src) = src.downcast_ref::<String>() {
					write!(f, "Runtime error: {}", src)
				} else if let Some(src) = src.downcast_ref::<&str>() {
					write!(f, "Runtime error: {}", src)

				} else {
					write!(f, "Runtime error: {:?}", src)
				}
			},
			UIError::Extract { src } => write!(f, "Extract error: {}", src),
			UIError::GroupeRegistry { src } => write!(f, "Groupe registry error: {}", src),
			UIError::CompteRegistry { src } => write!(f, "Compte registry error: {}", src),
			UIError::MembreRegistry { src } => write!(f, "Membre registry error: {}", src),
			UIError::CancelAction { desc } => write!(f, "Action cancelled: {}", desc),
			UIError::Compte { src } => write!(f, "Compte error: {}", src),
			UIError::Input { src } => write!(f, "Input error: {}", src),
		}
	}
}
impl std::error::Error for UIError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			UIError::IO { src } => Some(src),
			UIError::Event { src } => Some(src),
			UIError::Runtime { .. } => None,
			UIError::Extract { src } => Some(src),
			UIError::GroupeRegistry { src } => Some(src),
			UIError::CompteRegistry { src } => Some(src),
			UIError::MembreRegistry { src } => Some(src),
			UIError::CancelAction { .. } => None,
			UIError::Compte { src } => Some(src),
			UIError::Input { src } => Some(src),
		}
	}
}
impl UIError {
	pub fn msg(msg: &str) -> Self {
		UIError::Runtime { src: Box::new(msg.to_string()) }
	}
}

#[derive(Debug, Copy, Clone)]
pub enum ScreenSize {
	Fill,
	Length(u16),
	Fit{ min: u16, max: u16 },
	Ratio(f32),
}

#[derive(Debug, Copy, Clone)]
pub struct Theme {
	menu_item_base_color: Color,
	menu_item_selected_color: Color,
	menu_item_selected_bg_color: Color,
	background_color: Color,
	main_menu_width: u16,
	app_min_width: u16,
	progress_bar_height: ScreenSize,
	progress_bar_color: Color,
	progress_bar_max_width: ScreenSize,
	max_error_box_width: ScreenSize,
	max_error_box_height: ScreenSize,
	info_box_max_width: ScreenSize,
	info_box_max_height: ScreenSize,
	popup_menu_width: ScreenSize,
	popup_menu_height: ScreenSize,
}
impl Theme {
	const DARK: Self = Self {
		menu_item_base_color: Color::White,
		menu_item_selected_color: Color::Yellow,
		menu_item_selected_bg_color: Color::DarkGray,
		background_color: Color::Black,
		main_menu_width: 30,
		app_min_width: 80, // if the terminal is smaller than this, it will only render one screen at a time instead of seeing the screen and the menu at the same time with pop-ups on top.
		progress_bar_height: ScreenSize::Length(6),
		progress_bar_color: Color::White,
		progress_bar_max_width: ScreenSize::Length(120),
		max_error_box_width: ScreenSize::Fit { min: 20, max: 60 },
		max_error_box_height: ScreenSize::Fit {min: 6, max: 20 },
		info_box_max_width: ScreenSize::Fit { min: 20, max: 60 },
		info_box_max_height: ScreenSize::Fit { min: 8, max: 20 },
		popup_menu_width: ScreenSize::Fit { min: 20, max: 60 },
		popup_menu_height: ScreenSize::Fit { min: 0, max: 20 },
	};
}

#[derive(Debug)]
pub enum UpdateAction {
	Continue,
	Quit,
	Pop,
	Push(Box<dyn Screen>),
	PushSub(Box<dyn Screen>),
	Replace(Box<dyn Screen>),
	ReplaceSub(Box<dyn Screen>),
	ErrorPopUp(Box<dyn std::error::Error>),
	ErrorReplace(Box<dyn std::error::Error>),
}
impl UpdateAction {
	pub fn one(self) -> Vec<Self> {
		vec![self]
	}
	pub fn empty() -> Vec<Self> {
		vec![]
	}
}

pub trait Screen where Self: ratatui::widgets::WidgetRef + std::fmt::Debug {
	fn handle_event(&mut self, event: event::Event, state: Arc<AppState>) -> Result<UpdateActions, UIError>;
	fn render_focus(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, focus: bool) {
		self.render_ref(area, buf);
	}
	fn background_update(&mut self, event: Event) -> Result<UpdateActions, UIError> {
		Ok(UpdateAction::Continue.one())
	}
}

#[derive(Debug)]
pub struct AppState {
	pub old_out_dir: RwLock<PathBuf>,
	pub old_in_dir: RwLock<PathBuf>,
	pub config: RwLock<crate::config::Config>,
	pub groupes: RwLock<GroupeReg>,
	pub comptes: RwLock<CompteReg>,
	pub membres: RwLock<MembreReg>,
	pub theme: RwLock<Theme>,
	pub polls: RwLock<VecDeque<PollRequest<'static>>>,
}
impl Default for AppState {
	fn default() -> Self {
		let mut groupes = GroupeReg::default();
		let mut comptes = CompteReg::default();
		let mut membres = MembreReg::default();

		let _ = groupes.add(NULL_GROUPE.clone());
		let _ = comptes.add(NULL_COMPTE.clone());
		let _ = membres.add(NULL_MEMBRE.clone());

		Self {
			old_out_dir: RwLock::new(PathBuf::from("/")),
			old_in_dir: RwLock::new(PathBuf::from("/")),
			config: RwLock::new(crate::config::Config::default()),
			groupes: RwLock::new(groupes),
			comptes: RwLock::new(comptes),
			membres: RwLock::new(membres),
			theme: RwLock::new(Theme::DARK),
			polls: RwLock::new(VecDeque::new()),
		}
	}
}

#[derive(Clone, Default)]
pub struct Poll<'a> {
	pub title: String,
	pub prompt: Text<'a>,
	pub validation: Option<crate::ui::screens::TextInputValidation>,
	pub show_error: bool,
}
impl Debug for Poll<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Poll")
			.field("title", &self.title)
			.field("prompt", &self.prompt)
			.field("validation", &self.validation.is_some())
			.field("show_error", &self.show_error)
			.finish()
	}
}
impl<'a> Poll<'a> {
	pub fn with_title(mut self, title: String) -> Self {
		self.title = title;
		self
	}
	pub fn with_prompt(mut self, prompt: Text<'a>) -> Self {
		self.prompt = prompt;
		self
	}
	pub fn with_validation(mut self, validation: crate::ui::screens::TextInputValidation) -> Self {
		self.validation = Some(validation);
		self
	}
	pub fn with_show_error(mut self, show_error: bool) -> Self {
		self.show_error = show_error;
		self
	}
	pub fn no_validation(mut self) -> Self {
		self.validation = None;
		self
	}
}
impl Poll<'static> {
	pub fn poll(self, state: Arc<AppState>) -> Result<Option<String>, RecvError> {
		let (send, recv) = std::sync::mpsc::channel::<Option<String>>();
		let poll_request = PollRequest {
			data: self,
			answer_to: send,
		};
		state.polls.write().expect("Poisoned Lock").push_back(poll_request);
		recv.recv()
	}
}


pub struct PollRequest<'a> {
	pub data: Poll<'a>,
	pub answer_to: std::sync::mpsc::Sender<Option<String>>,
}
impl<'a> PollRequest<'a> {
	pub fn new(answer_to: std::sync::mpsc::Sender<Option<String>>) -> Self {
		Self {
			data: Poll::default(),
			answer_to,
		}
	}

	pub fn to_line_input_screen(self) -> screens::LineInputScreen<'a> {
		let val = self.data.validation.clone();
		let screen = screens::LineInputScreen::default()
			.with_title(self.data.title)
			.with_message(self.data.prompt)
			.with_after(Box::new(move |result, _state| {
				if let Some(result) = result {
					if let Some(val) = val.as_ref() {
						if val(result) {
							match self.answer_to.send(Some(result.to_string())) {
								Ok(_) => { Ok(UpdateAction::Pop.one()) },
								Err(e) => { Err(UIError::Runtime { src: Box::new(format!("Failed to send poll answer: {}", e)) }) },
							}
						} else {
							// invalid input
							if self.data.show_error {
								Ok(UpdateAction::ErrorPopUp(Box::new(UIError::msg("Input invalide"))).one())
							} else {
								Ok(UpdateAction::Continue.one())
							}
						}
					} else { // no validation required, send the result to answer_to
						match self.answer_to.send(Some(result.to_string())) {
							Ok(_) => { Ok(UpdateAction::Pop.one()) },
							Err(e) => { Err(UIError::Runtime { src: Box::new(format!("Failed to send poll answer: {}", e)) }) },
						}
					}
				} else { // cancel the prompt, send None to answer_to
					match self.answer_to.send(None) {
						Ok(_) => { Ok(UpdateAction::Pop.one()) },
						Err(e) => { Err(UIError::Runtime { src: Box::new(format!("Failed to send poll answer: {}", e)) }) },
					}
				}
			}));
		if let Some(validation) = self.data.validation {
			screen.with_validation(validation)
		} else {
			screen
		}
	}
}
impl Debug for PollRequest<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Poll")
			.field("title", &self.data.title)
			.field("prompt", &self.data.prompt)
			.field("validation", &self.data.validation.is_some())
			.field("show_error", &self.data.show_error)
			.finish()
	}
}
