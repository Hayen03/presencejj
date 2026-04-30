use std::{sync::{Arc, Mutex}, thread::JoinHandle};

use ratatui::{layout::Rect, style::{Color, Style, Stylize}, symbols::border, text::Line, widgets::{Block, Gauge, Paragraph, Widget, WidgetRef}};

use crate::ui::{AppState, Theme, UIError, screens::Desc};



#[derive(Debug)]
pub struct ProgressBar {
	progress: Arc<Mutex<u32>>,
	_cancel: Arc<Mutex<bool>>,
	text: Arc<Mutex<Desc>>,
	target: u32,
	thread_handle: Option<std::thread::JoinHandle<Result<(), UIError>>>,
	title: String,
}
impl ProgressBar {
	pub fn new(title: String, target: u32) -> Self {
		Self { progress: Arc::new(Mutex::new(0)), _cancel: Arc::new(Mutex::new(false)), text: Arc::new(Mutex::new(Desc::None)), target, thread_handle: None, title }
	}
	pub fn get_progress(&self) -> u32 {
		*self.progress.lock().unwrap()
	}
	pub fn set_progress(&self, progress: u32) {
		*self.progress.lock().unwrap() = progress;
	}
	pub fn get_target(&self) -> u32 {
		self.target
	}
	pub fn is_done(&self) -> bool {
		self.get_progress() >= self.target
	}
	pub fn cancel(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		*self._cancel.lock().unwrap() = true;
		if let Some(handle) = self.thread_handle.take() {
			match handle.join() {
				Ok(Err(e)) => Err(Box::new(e)),
				Ok(Ok(())) => Ok(()),
				Err(e) => Err(Box::new(UIError::Runtime { src: e })),
			}
		} else {
			Ok(())
		}
	}
	pub fn get_cancel_reference(&self) -> Arc<Mutex<bool>> {
		self._cancel.clone()
	}
	pub fn get_progress_reference(&self) -> Arc<Mutex<u32>> {
		self.progress.clone()
	}
	pub fn get_text_reference(&self) -> Arc<Mutex<Desc>> {
		self.text.clone()
	}
	pub fn with_thread(mut self, thread_handle: std::thread::JoinHandle<Result<(), UIError>>) -> Self {
		self.thread_handle = Some(thread_handle);
		self
	}
}
impl WidgetRef for ProgressBar {
	fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
		let progress = self.get_progress();
		let target = self.get_target();
		let ratio = if target == 0 { 1.0 } else { progress as f64 / target as f64 };
		// find the actual area where we want to render the progress bar, with some padding
		// render as progress bar
		let bar_height = Theme::DARK.progress_bar_height.min(area.height);
		let bar_width = Theme::DARK.progress_bar_max_width.min(area.width - 4);
		let bar_rect = area.centered(
			ratatui::layout::Constraint::Length(bar_width), 
			ratatui::layout::Constraint::Length(bar_height),
		);
		ratatui::widgets::Clear.render(bar_rect, buf); // clear the area before rendering the progress bar

		let title = Line::from(self.title.as_str()).white().bold();
		let desc = match self.text.lock().expect("Poisoned lock").clone() {
			Desc::None => Line::from(""),
			Desc::Info(s) => Line::from(s).green(),
			Desc::Warning(s) => Line::from(s).yellow(),
			Desc::Error(s) => Line::from(s).red(),
		}.left_aligned();

		if bar_rect.width < 30 {
			let instruction = if self.is_done() {
			Line::from(" Appuyez sur Entrée pour continuer. ")
			} else {
				Line::from(" Appuyez sur Esc pour annuler. ")
			};
			let bar_block = Block::bordered()
				.title(title.centered())
				.title_bottom(instruction.centered().gray())
				.border_set(border::THICK);
			// render as text
			let text = Line::from(format!("{:03.2} %", ratio * 100.0)).centered().white();
			Paragraph::new(vec![desc, text])
				.centered()
				.block(bar_block)
				.bg(Color::Black)
				.render(bar_rect, buf);
			
		} else {
			let instruction = if self.is_done() {
			Line::from(" Chargement terminé! Appuyez sur Entrée pour continuer. ")
			} else {
				Line::from(format!(" Chargement en cours... {:03.2}% Appuyez sur Esc pour annuler. ", ratio * 100.0))
			};
			let bar_block = Block::bordered()
				.title(title.centered())
				.title_bottom(instruction.centered().gray())
				.border_set(border::THICK)
				.bg(Color::Black);
			let inner = bar_block.inner(bar_rect);
			let bar_area = Rect { // offset the inner area by one to give space to the desc label
				x: inner.x,
				y: inner.y + 1,
				width: inner.width,
				height: inner.height - 1,
			};
			let desc_area = Rect {
				x: inner.x,
				y: inner.y,
				width: inner.width,
				height: 1,
			};
			bar_block.render(bar_rect, buf);
			Gauge::default()
				//.block(bar_block)
				.gauge_style(Style::new().white().on_black().italic())
				.use_unicode(true)
				.ratio(ratio)
				.render(bar_area, buf);
			Paragraph::new(vec![desc])
				.left_aligned()
				.render(desc_area, buf);
		}
	}
}
impl crate::ui::Screen for ProgressBar {
	fn handle_event(&mut self, event: crate::ui::event::Event, _state: Arc<AppState>) -> Result<crate::ui::UpdateAction, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				match key.code {
					crossterm::event::KeyCode::Esc => {
						let thread_result = self.cancel();
						if let Err(err) = thread_result {
							return Ok(crate::ui::UpdateAction::ErrorReplace(err));
						}
						Ok(crate::ui::UpdateAction::Pop)
					},
					crossterm::event::KeyCode::Enter if self.is_done() => Ok(crate::ui::UpdateAction::Pop),
					_ => Ok(crate::ui::UpdateAction::Continue),
				}
			},
			crate::ui::event::Event::Tick => {
				// poll for error
				if let Some(thread_completed) = self.thread_handle.as_ref().map(|handle| handle.is_finished()) {
					if thread_completed {
						let thread_result = self.thread_handle.take().unwrap().join();
						if let Err(err) = thread_result {
							Ok(crate::ui::UpdateAction::ErrorReplace(Box::new(UIError::Runtime { src: err })))
						} else {
							self.set_progress(self.get_target());
							Ok(crate::ui::UpdateAction::Continue)
						}
					} else {
						Ok(crate::ui::UpdateAction::Continue)
					}
				} else {
					Ok(crate::ui::UpdateAction::Continue)
				}
			},
			_ => { Ok(crate::ui::UpdateAction::Continue) },
		}
	}
}