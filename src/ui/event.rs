use crossterm::event::{KeyEvent, MouseEvent};
use std::{sync::mpsc, thread, time::{Duration, Instant}};

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
	handler: thread::JoinHandle<()>,
}
impl EventHandler {
	pub fn new(tick_rate: u64) -> Self {
		let tick_rate = Duration::from_millis(tick_rate);
		let (sender, receiver) = mpsc::channel();
		let handler = {
			let sender = sender.clone();
			thread::spawn(move || {
				let mut last_tick = Instant::now();
				loop {
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
							_ => unimplemented!(),
						}.expect("Failed to send terminal event");
					}
					if last_tick.elapsed() >= tick_rate {
						sender.send(Event::Tick).expect("Failed to send tick event");
						last_tick = Instant::now();
					}
				}
			})
		};
		Self { sender, receiver, handler }
	}
	pub fn next(&self) -> Result<Event, EventError> {
		Ok(self.receiver.recv()?)
	}
}