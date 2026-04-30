mod progress;
mod error_screen;

pub use progress::*;
pub use error_screen::*;

#[derive(Debug, Clone, Default)]
pub enum Desc {
	#[default]
	None,
	Info(String),
	Warning(String),
	Error(String),
}