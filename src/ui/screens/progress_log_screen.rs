use std::{cell::Cell, sync::{Arc, Mutex}};

use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Stylize}, symbols::border, text::Line, widgets::{Block, Borders, Clear, Gauge, Widget, WidgetRef}};

use crate::ui::{Screen, UIError, screens::{Logger, ScrollMode}};

#[derive(Debug)]
pub struct ProgressLogScreen<'a> {
	scroll_mode: Cell<ScrollMode>,
	logger: Arc<Mutex<Logger<'a>>>,
	target: u32,
	progress: Arc<Mutex<u32>>,
	title: String,
	thread_handle: Option<std::thread::JoinHandle<Result<(), UIError>>>,
	cancel_hook: Arc<Mutex<bool>>, // to cancel the thread early
	max_scroll: Cell<usize>,
}
impl<'a> ProgressLogScreen<'a> {
	pub fn new(title: String, target: u32) -> Self {
		Self {
			scroll_mode: Cell::new(ScrollMode::Auto),
			logger: Arc::new(Mutex::new(Logger::default())),
			target,
			progress: Arc::new(Mutex::new(0)),
			title,
			thread_handle: None,
			cancel_hook: Arc::new(Mutex::new(false)),
			max_scroll: Cell::new(0),
		}
	}
	pub fn get_logger(&self) -> Arc<Mutex<Logger<'a>>> {
		self.logger.clone()
	}
	pub fn ratio(&self) -> f64 {
		if self.target == 0 {
			1.0
		} else {
			let progress = *self.progress.lock().expect("Poisoned Lock");
			(progress as f64 / self.target as f64).clamp(0.0, 1.0)
		}
	}
	pub fn is_done(&self) -> bool {
		*self.progress.lock().expect("Poisoned Lock") >= self.target
	}
	pub fn get_progress_hook(&self) -> Arc<Mutex<u32>> {
		self.progress.clone()
	}
	pub fn get_cancel_hook(&self) -> Arc<Mutex<bool>> {
		self.cancel_hook.clone()
	}
	pub fn with_thread(mut self, thread_handle: std::thread::JoinHandle<Result<(), UIError>>) -> Self {
		self.thread_handle = Some(thread_handle);
		self
	}
}

impl WidgetRef for ProgressLogScreen<'_> {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		Clear.render(area, buf);

		let instructions = if self.is_done() {
			Line::from(" Terminé ! Appuyez sur Entrée pour continuer ").green()
		} else {
			Line::from(format!(" Chargement en cours... {:.2}% Appuyez sur Esc pour Annuler ", self.ratio() * 100.0)).gray()
		};
		let block = Block::bordered()
			.title_top(Line::from(format!(" {} ", self.title)).white().centered())
			.title_bottom(instructions.centered())
			.border_set(border::THICK)
			.bg(Color::Black);
		let inner = block.inner(area);
		block.render(area, buf);

		// progress bar (first line)
		let progress_area = Rect {
			x: inner.x,
			y: inner.y,
			width: inner.width,
			height: 1,
		};
		let progress_ratio = self.ratio();
		let progress_block = Block::bordered()
			.border_set(border::PLAIN)
			.borders(Borders::LEFT | Borders::RIGHT);
		Gauge::default()
		.ratio(progress_ratio)
		.label(format!("{}/{}", *self.progress.lock().expect("Poisoned Lock"), self.target))
		.use_unicode(true)
		.block(progress_block)
		.render(progress_area, buf);

		// logging
		let log_area = Rect {
			x: inner.x,
			y: inner.y + 1,
			width: inner.width,
			height: inner.height - 1,
		};
		let logger = self.logger.lock().expect("Poisoned Lock");
		let logs = logger.widget();
		let logh = logs.line_count(log_area.width);
		// determine the scrolling
		let max_scroll = logh.saturating_sub(log_area.height as usize);
		let scroll = match self.scroll_mode.get() {
			ScrollMode::Auto => max_scroll,
			ScrollMode::Manual(s) => s.min(max_scroll),
		};
		// update back to auto scroll if we're at the bottom and new logs are added
		if scroll == max_scroll {
			self.scroll_mode.set(ScrollMode::Auto);
		}
		logs.scroll((scroll.try_into().unwrap_or(u16::MAX), 0)).render(log_area, buf);
		self.max_scroll.set(max_scroll);
	}
}
impl Screen for ProgressLogScreen<'_> {
	fn handle_event(&mut self, event: crate::ui::event::Event, _state: Arc<crate::ui::AppState>) -> Result<crate::ui::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event::KeyCode;
				match key.code {
					KeyCode::Esc => {
						*self.cancel_hook.lock().expect("Poisoned Lock") = true;
						let thread_result = self.thread_handle.take().map(|th| th.join());
						match thread_result {
							None => Ok(crate::ui::UpdateAction::Pop.one()), // no thread
							Some(Ok(Ok(()))) => Ok(crate::ui::UpdateAction::Pop.one()), // thread completed successfully
							Some(Err(err)) => Ok(crate::ui::UpdateAction::ErrorReplace(Box::new(UIError::Runtime { src: err })).one()), // thread panicked
							Some(Ok(Err(err))) => Ok(crate::ui::UpdateAction::ErrorReplace(Box::new(err)).one()), // thread completed with error
						}
					},
					KeyCode::Enter if self.is_done() => {
						// make sure the thread is completed
						let thread_result = self.thread_handle.take().map(|th| th.join());
						match thread_result {
							None => Ok(crate::ui::UpdateAction::Pop.one()), // no thread
							Some(Ok(Ok(()))) => Ok(crate::ui::UpdateAction::Pop.one()), // thread completed successfully
							Some(Err(err)) => Ok(crate::ui::UpdateAction::ErrorReplace(Box::new(UIError::Runtime { src: err })).one()), // thread panicked
							Some(Ok(Err(err))) => Ok(crate::ui::UpdateAction::ErrorReplace(Box::new(err)).one()), // thread completed with error
						}
					},
					KeyCode::Up => {
						// switch to manual scroll and decrease scroll
						let current_scroll = match self.scroll_mode.get() {
							ScrollMode::Auto => self.max_scroll.get(),
							ScrollMode::Manual(s) => s,
						};
						self.scroll_mode.set(ScrollMode::Manual(current_scroll.saturating_sub(1)));
						Ok(crate::ui::UpdateAction::Continue.one())
					},
					KeyCode::Down => {
						// switch to manual scroll and decrease scroll
						let current_scroll = match self.scroll_mode.get() {
							ScrollMode::Auto => self.max_scroll.get(),
							ScrollMode::Manual(s) => s,
						};
						self.scroll_mode.set(ScrollMode::Manual(current_scroll.saturating_add(1)));
						Ok(crate::ui::UpdateAction::Continue.one())
					},
					_ => Ok(crate::ui::UpdateAction::Continue.one()),
				}
			},
			crate::ui::event::Event::Tick => {
				// check if thread is completed
				if self.thread_handle.as_ref().map(|th| th.is_finished()).unwrap_or(false) {
					let thread_result = self.thread_handle.take().expect("Thread handle should be Some since it's finished").join();
					match thread_result {
						Ok(Ok(())) => { // completed successfully
							// set the progress bar to full to mark as completed
							*self.progress.lock().expect("Poisoned Lock") = self.target;
							Ok(crate::ui::UpdateAction::Continue.one())
						},
						Ok(Err(err)) => { // completed with error
							Ok(crate::ui::UpdateAction::ErrorReplace(Box::new(err)).one())
						},
						Err(err) => { // thread panicked
							Ok(crate::ui::UpdateAction::ErrorReplace(Box::new(UIError::Runtime { src: err })).one())
						},
					}
				} else {
					Ok(crate::ui::UpdateAction::Continue.one())
				}
			},
			crate::ui::event::Event::Mouse(mouse) => {
				use crossterm::event::MouseEventKind;
				match mouse.kind {
					MouseEventKind::ScrollUp => {
						// switch to manual scroll and decrease scroll
						let current_scroll = match self.scroll_mode.get() {
							ScrollMode::Auto => self.max_scroll.get(),
							ScrollMode::Manual(s) => s,
						};
						self.scroll_mode.set(ScrollMode::Manual(current_scroll.saturating_sub(1)));
						Ok(crate::ui::UpdateAction::Continue.one())
					},
					MouseEventKind::ScrollDown => {
						// switch to manual scroll and decrease scroll
						let current_scroll = match self.scroll_mode.get() {
							ScrollMode::Auto => self.max_scroll.get(),
							ScrollMode::Manual(s) => s,
						};
						self.scroll_mode.set(ScrollMode::Manual(current_scroll.saturating_add(1)));
						Ok(crate::ui::UpdateAction::Continue.one())
					},
					_ => Ok(crate::ui::UpdateAction::Continue.one()),
				}
			},
			_ => {
				Ok(crate::ui::UpdateAction::Continue.one())
			},
		}
	}
}