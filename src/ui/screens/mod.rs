mod progress;
mod error_screen;
mod progress_log_screen;
mod info_screen;
mod input_screen;
mod menu;
mod task_screen;
mod text_screen;
mod groupe_table;
mod membre_table;
mod compte_table;
mod view_table;
mod page_membre;

pub use progress::ProgressBar;
pub use error_screen::*;
pub use progress_log_screen::*;
pub use info_screen::*;
pub use input_screen::*;
pub use menu::*;
pub use task_screen::*;
pub use text_screen::*;
pub use groupe_table::*;
pub use membre_table::*;
pub use compte_table::*;
pub use view_table::*;
pub use page_membre::*;
use ratatui::{style::Stylize, text::{Line, Text}, widgets::{Paragraph, Wrap}};
use lazy_static::lazy_static;

lazy_static!{
	pub static ref ENTER_INSTRUCTIONS: Line<'static> = Line::from(vec![
		" Appuyez sur ".gray(),
		"Entrée".light_blue().bold(),
		" pour continuer ".gray(),
	]).centered();
	pub static ref ESC_INSTRUCTIONS: Line<'static> = Line::from(vec![
		" Appuyez sur ".gray(),
		"ESC".light_blue().bold(),
		" pour annuler ".gray(),
	]).centered();
	pub static ref ENTER_ESC_INSTRUCTIONS: Line<'static> = Line::from(vec![
		" Appuyez sur ".gray(),
		"Entrée".light_blue().bold(),
		" pour valider, ".gray(),
		"Esc".light_blue().bold(),
		" pour annuler ".gray(),
	]).centered();
}


#[derive(Debug, Clone, Default)]
pub enum Desc {
	#[default]
	None,
	Info(String),
	Warning(String),
	Error(String),
}
impl Desc {
	pub fn as_str(&self) -> &str {
		match self {
			Desc::None => "",
			Desc::Info(s) | Desc::Warning(s) | Desc::Error(s) => s.as_str(),
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct Logger<'a> {
	text: Text<'a>,
}
impl Logger<'_> {
	pub fn log(&mut self, desc: Desc) {
		let line = match desc {
			Desc::None => Line::default(),
			Desc::Info(s) => Line::from(s).green(),
			Desc::Warning(s) => Line::from(s).yellow(),
			Desc::Error(s) => Line::from(s).red(),
		};
		self.text.push_line(line);
	}
	pub fn widget(&'_ self) -> Paragraph<'_> {
		Paragraph::new(self.text.clone()).wrap(Wrap { trim: false })
	}
	/*
	pub fn height(&self, width: u16) -> u16 {
		self.lns.iter().filter_map(|desc| {
			match desc {
				Desc::None => None,
				Desc::Info(s) => Some(textwrap::wrap(s, width as usize).len() as u16),
				Desc::Warning(s) => Some(textwrap::wrap(s, width as usize).len() as u16),
				Desc::Error(s) => Some(textwrap::wrap(s, width as usize).len() as u16),
			}
		}).sum()
	}
	*/
}

#[derive(Debug, Clone, Copy, Default)]
enum ScrollMode {
	#[default]
	Auto,
	Manual(usize),
}

#[derive(Debug)]
pub enum PageError {
	NonmatchingIDs{msg: String},
	MissingData{msg: String},
}
impl std::fmt::Display for PageError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PageError::NonmatchingIDs { msg } => write!(f, "Non matching IDs:{}", msg),
			PageError::MissingData { msg } => write!(f, "Missing data: {}", msg),
		}
	}
}
impl std::error::Error for PageError {}