
use crossterm::{event::DisableMouseCapture, execute};
use ratatui::{Terminal, prelude::CrosstermBackend};

pub type CrosstermTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

use crate::ui::{app::App, event::EventHandler};

#[derive(Debug)]
pub enum TuiError {
	IOError { src: std::io::Error },
}
impl From<std::io::Error> for TuiError {
	fn from(src: std::io::Error) -> Self {
		TuiError::IOError { src }
	}
}
impl std::fmt::Display for TuiError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TuiError::IOError { src } => write!(f, "IO error: {}", src),
		}
	}
}
impl std::error::Error for TuiError {}

pub struct Tui {
	terminal: CrosstermTerminal,
	pub events: EventHandler,
}

impl Tui {
	pub fn enter() -> Result<Self, TuiError> {
		/*
		terminal::enable_raw_mode()?;
		execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

		let panic_hook = panic::take_hook();
		panic::set_hook(Box::new(move |panic_info| {
			Self::reset().expect("Failed to reset terminal on panic");
			panic_hook(panic_info);
		}));
		*/

		//self.terminal.hide_cursor()?;
		//self.terminal.clear()?;
		execute!(std::io::stdout(), DisableMouseCapture)?;
		let terminal = ratatui::init();
		let events = EventHandler::new(250);
		Ok(Self { terminal, events })
	}
	/*
	pub fn reset() -> Result<(), TuiError> {
		terminal::disable_raw_mode()?;
		execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
		Ok(())
	}
	*/
	pub fn exit(&mut self) -> Result<(), TuiError> {
		// end the event loop
		self.events.end();
		execute!(std::io::stdout(), DisableMouseCapture)?;
		ratatui::restore();
		//Self::reset()?;
		//self.terminal.show_cursor()?;
		Ok(())
	}
	pub fn draw(&mut self, app: &App) -> Result<(), TuiError> {
		self.terminal.draw(|frame| frame.render_widget(app, frame.area()))?;
		Ok(())
	}
}
