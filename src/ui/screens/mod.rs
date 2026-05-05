mod progress;
mod error_screen;
mod progress_log_screen;
mod info_screen;
mod input_screen;
mod menu;

pub use progress::ProgressBar;
pub use error_screen::*;
pub use progress_log_screen::*;
pub use info_screen::*;
pub use input_screen::*;
pub use menu::*;
use ratatui::{style::Stylize, text::{Line, Text}, widgets::{Paragraph, Wrap}};

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