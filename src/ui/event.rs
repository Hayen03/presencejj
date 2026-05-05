use crossterm::event::{KeyEvent, MouseEvent};
use std::{sync::mpsc::{self, SendError}, thread, time::{Duration, Instant}};

use ratatui::crossterm::event as cte;

#[derive(Debug)]
pub enum EventError {
	RecvError { src: mpsc::RecvError },
}
impl std::fmt::Display for EventError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			EventError::RecvError { src } => write!(f, "Receive error: {}", src),
		}
	}
}
impl std::error::Error for EventError {}
impl From<mpsc::RecvError> for EventError {
	fn from(src: mpsc::RecvError) -> Self {
		EventError::RecvError { src }
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
	/// event sender channel
	sender: mpsc::Sender<Event>,
	/// Receiver Channel
	receiver: mpsc::Receiver<Event>,
	/// Event handler thread
	handler: Option<thread::JoinHandle<()>>,
	// hook for cancellation of the event handler thread
	cancel_hook: std::sync::Arc<std::sync::Mutex<bool>>,
}
impl EventHandler {
	pub fn new(tick_rate: u64) -> Self {
		let tick_rate = Duration::from_millis(tick_rate);
		let (sender, receiver) = mpsc::channel();
		let cancel_hook = std::sync::Arc::new(std::sync::Mutex::new(false));
		let handler = {
			let sender = sender.clone();
			let cancel = cancel_hook.clone();
			thread::spawn(move || {
				let mut last_tick = Instant::now();
				loop {
					// check for cancellation
					if *cancel.lock().expect("Poisoned Lock") {
						break;
					}
					let timeout = tick_rate
						.checked_sub(last_tick.elapsed())
						.unwrap_or(tick_rate);
					if cte::poll(timeout).expect("unable to poll for event") {
						match cte::read().expect("unable to read event") {
							cte::Event::Key(key) => {
								if key.kind == cte::KeyEventKind::Press {
									sender.send(Event::Key(key))
								} else {
									Ok(())
								}
							},
							cte::Event::Mouse(mouse) => { sender.send(Event::Mouse(mouse)) },
							cte::Event::Resize(w, h) => { sender.send(Event::Resize(w, h)) },
							_ => continue,
						}.expect("Failed to send terminal event");
					}
					if last_tick.elapsed() >= tick_rate {
						sender.send(Event::Tick).expect("Failed to send tick event");
						last_tick = Instant::now();
					}
				}
			})
		};
		Self { sender, receiver, handler: Some(handler), cancel_hook }
	}
	pub fn next(&self) -> Result<Event, EventError> {
		Ok(self.receiver.recv()?)
	}
	pub fn send(&self, event: Event) -> Result<(), SendError<Event>> {
		self.sender.send(event)
	}
	pub fn end(&mut self) {
		*self.cancel_hook.lock().expect("Poisoned Lock") = true;
		if let Some(handle) = self.handler.take() {
			let _ = handle.join();
		}
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