use std::{any::Any, path::{Path, PathBuf}, sync::{Arc, RwLock}};

use ratatui::{Frame, style::Color};

use crate::{extract::ExtractError, groupes::{RegError, comptes::{CompteErr, CompteID, CompteReg, NULL_COMPTE}, groupes::{GroupeID, GroupeReg, NULL_GROUPE}, membres::{MembreID, MembreReg, NULL_MEMBRE}}};

pub mod app;
pub mod tui;
pub mod event;
pub mod actions;
pub mod screens;

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
		}
	}
}
impl std::error::Error for UIError {}

#[derive(Debug, Copy, Clone)]
pub struct Theme {
	menu_item_base_color: Color,
	menu_item_selected_color: Color,
	menu_item_selected_bg_color: Color,
	background_color: Color,
	main_menu_width: u16,
	app_min_width: u16,
	progress_bar_height: u16,
	progress_bar_color: Color,
	progress_bar_max_width: u16,
	max_error_box_width: u16,
	max_error_box_height: u16,
	info_box_max_width: u16,
	info_box_max_height: u16,
}
impl Theme {
	const DARK: Self = Self {
		menu_item_base_color: Color::White,
		menu_item_selected_color: Color::Yellow,
		menu_item_selected_bg_color: Color::DarkGray,
		background_color: Color::Black,
		main_menu_width: 30,
		app_min_width: 80, // if the terminal is smaller than this, it will only render one screen at a time instead of seeing the screen and the menu at the same time with pop-ups on top.
		progress_bar_height: 6,
		progress_bar_color: Color::White,
		progress_bar_max_width: 120,
		max_error_box_width: 160,
		max_error_box_height: 40,
		info_box_max_width: 80,
		info_box_max_height: 60,
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

pub trait Screen where Self: ratatui::widgets::WidgetRef + std::fmt::Debug {
	fn handle_event(&mut self, event: event::Event, state: Arc<AppState>) -> Result<UpdateAction, UIError>;
	fn render_focus(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, focus: bool) {
		self.render_ref(area, buf);
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
		}
	}
}