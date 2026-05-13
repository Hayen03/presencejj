use crossterm::event::{KeyEvent, MouseEvent};
use std::time::{Duration, Instant};

use ratatui::crossterm::event as cte;

#[derive(Debug)]
pub enum EventError {
	IO { src: std::io::Error },
}
impl std::fmt::Display for EventError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			EventError::IO { src } => write!(f, "IO error: {}", src),
		}
	}
}
impl std::error::Error for EventError {}
impl From<std::io::Error> for EventError {
	fn from(src: std::io::Error) -> Self {
		EventError::IO { src }
	}
}


#[derive(Debug, Clone, Copy)]
pub enum Event {
	Tick,
	Key(KeyEvent),
	Mouse(MouseEvent),
	Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
	tick_rate: Duration,
	last_tick: Instant,
	focused: bool,
}
impl EventHandler {
	pub fn new(tick_rate: u64) -> Self {
		let tick_rate = Duration::from_millis(tick_rate);
		Self { tick_rate, last_tick: Instant::now(), focused: true }
	}
	pub fn next(&mut self) -> Result<Event, EventError> {
		loop {
			let timeout = self.tick_rate
				.checked_sub(self.last_tick.elapsed())
				.unwrap_or(Duration::ZERO);
			if cte::poll(timeout)? {
				match cte::read()? {
					cte::Event::Key(key) if key.kind == cte::KeyEventKind::Press && self.focused => return Ok(Event::Key(key)),
					cte::Event::Mouse(mouse) if self.focused => return Ok(Event::Mouse(mouse)),
					cte::Event::Resize(w, h) => return Ok(Event::Resize(w, h)),
					cte::Event::FocusGained => self.focused = true,
					cte::Event::FocusLost => self.focused = false,
					_ => {},
				}
			}
			if self.last_tick.elapsed() >= self.tick_rate {
				self.last_tick = Instant::now();
				return Ok(Event::Tick);
			}
		}
	}
	pub fn end(&mut self) {
	}
}
impl From<Event> for ratatui_textarea::Input {
	fn from(value: Event) -> Self {
		if let Event::Key(key) = value {
			let shift = key.modifiers.contains(cte::KeyModifiers::SHIFT);
			let ctrl = key.modifiers.contains(cte::KeyModifiers::CONTROL);
			let alt = key.modifiers.contains(cte::KeyModifiers::ALT);
			ratatui_textarea::Input { key: key.code.into(), ctrl, alt, shift }
		} else {
			ratatui_textarea::Input { key: ratatui_textarea::Key::Null, ctrl: false, alt: false, shift: false }
		}
	}
}
