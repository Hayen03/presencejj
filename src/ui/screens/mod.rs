mod progress;
mod error_screen;
mod progress_log_screen;
mod info_screen;
mod input_screen;

pub use progress::ProgressBar;
pub use error_screen::*;
pub use progress_log_screen::*;
pub use info_screen::*;
pub use input_screen::*;
use ratatui::{style::Stylize, text::Line, widgets::{Paragraph, Wrap}};

#[derive(Debug, Clone, Default)]
pub enum Desc {
	#[default]
	None,
	Info(String),
	Warning(String),
	Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct Logger {
	lns: Vec<Desc>,
}
impl Logger {
	pub fn log(&mut self, desc: Desc) {
		self.lns.push(desc);
	}
	pub fn widget(&'_ self) -> Paragraph<'_> {
		let text = self.lns.iter().filter_map(|desc| {
			match desc {
				Desc::None => None,
				Desc::Info(s) => Some(Line::from(s.as_str()).green()),
				Desc::Warning(s) => Some(Line::from(s.as_str()).yellow()),
				Desc::Error(s) => Some(Line::from(s.as_str()).red()),
			}
		}).collect::<Vec<Line>>();
		Paragraph::new(text).wrap(Wrap { trim: false })
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