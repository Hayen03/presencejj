
use std::{any::Any, panic::UnwindSafe};

use crossterm::{event::DisableMouseCapture, execute};
use ratatui::{Terminal, buffer::Buffer, layout::Rect, prelude::CrosstermBackend, style::Color};

pub type CrosstermTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

use crate::ui::{app::App, event::EventHandler};

#[derive(Debug)]
pub enum TuiError {
	IOError { src: std::io::Error },
	Panic { info: Box<dyn Any + Send> },
}
impl From<std::io::Error> for TuiError {
	fn from(src: std::io::Error) -> Self {
		TuiError::IOError { src }
	}
}
impl From<Box<dyn Any + Send>> for TuiError {
	fn from(value: Box<dyn Any + Send>) -> Self {
		TuiError::Panic { info: value }
	}
}
impl std::fmt::Display for TuiError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TuiError::IOError { src } => write!(f, "IO error: {}", src),
			TuiError::Panic { info } => {
				if let Some(s) = info.downcast_ref::<&str>() {
					write!(f, "Panic: {}", s)
				} else if let Some(s) = info.downcast_ref::<String>() {
					write!(f, "Panic: {}", s)
				} else {
					write!(f, "Panic with non-string payload")
				}
			},
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
		self.terminal.draw(|frame| {
			let area = frame.area();
			frame.render_widget(app, area);
			normalize_default_colors(frame.buffer_mut(), area);
		})?;
		Ok(())
	}

	pub fn suspend_raw_mode<T>(&mut self, f: impl FnOnce() -> T + UnwindSafe) -> Result<T, TuiError> {
		crossterm::terminal::disable_raw_mode()?;
		execute!(std::io::stdout(), DisableMouseCapture)?;
		let res = std::panic::catch_unwind(f);
		execute!(std::io::stdout(), DisableMouseCapture)?;
		crossterm::terminal::enable_raw_mode()?;
		res.map_err(TuiError::from)
	}
}

fn normalize_default_colors(buf: &mut Buffer, area: Rect) {
	for y in area.y..area.y.saturating_add(area.height) {
		for x in area.x..area.x.saturating_add(area.width) {
			if let Some(cell) = buf.cell_mut((x, y)) {
				if cell.fg == Color::Reset {
					cell.fg = Color::Gray;
				}
				if cell.bg == Color::Reset {
					cell.bg = Color::Black;
				}
			}
		}
	}
}
