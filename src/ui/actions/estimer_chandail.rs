use std::{collections::HashMap, ffi::os_str::Display, sync::{Arc, Mutex}, thread::JoinHandle};

use ratatui::{buffer::Buffer, layout::Rect, style::{Style, Stylize}, symbols::border, text::{Line, Text}, widgets::{Block, Clear, Paragraph, Widget, WidgetRef, Wrap}};

use crossterm::event as cte;

use crate::{data::Taille, ui::{AppState, Screen, UIError, UpdateAction, actions::UpdateActions, screens::{InfoScreen, LineInputScreen, Menu, MenuItem}}};

pub fn estimer_chandail(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let menu_size = {
		let lock = state.theme.read().expect("Poisoned Lock");
		(lock.popup_menu_width, lock.popup_menu_height)
	};
	let info_size = {
		let lock = state.theme.read().expect("Poisoned Lock");
		(lock.info_box_max_width, lock.info_box_max_height)
	};
	let chandail_mode_menu = Menu::new(vec![
		MenuItem { id: EstimeChandailMethod::Simple, action: Box::new(move |state| {
			let info_box = InfoScreen::new(
				"Résultat".into(),
				Text::from("Vous avez choisi la méthode simple"),
			).with_size(info_size.0, info_size.1);
			Ok(UpdateAction::ReplaceSub(Box::new(info_box)).one())
		}) },
		MenuItem { id: EstimeChandailMethod::Complexe, action: Box::new(move |state| {
			let info_box = InfoScreen::new(
				"Résultat".into(),
				Text::from("Vous avez choisi la méthode complexe"),
			).with_size(info_size.0, info_size.1);
			Ok(UpdateAction::ReplaceSub(Box::new(info_box)).one())
		}) },
	].into_boxed_slice())
		.with_title("Mode d'estimation".into())
		.with_size(menu_size.0, menu_size.1);

	Ok(UpdateAction::PushSub(Box::new(chandail_mode_menu)).one())
}

#[derive(Debug, Clone, Copy)]
enum EstimeChandailMethod {
	Simple,
	Complexe,
}
impl EstimeChandailMethod {
	fn as_str(&self) -> &'static str {
		match self {
			EstimeChandailMethod::Simple => "Simple",
			EstimeChandailMethod::Complexe => "Complexe",
		}
	}
}
impl std::fmt::Display for EstimeChandailMethod {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

#[derive(Default, Debug)]
struct ChandailScreen {
	cancel_hook: Arc<Mutex<bool>>,
	results: Option<HashMap<Taille, usize>>,
	thread_handle: Option<std::thread::JoinHandle<HashMap<Taille, usize>>>,
}
impl ChandailScreen {
	fn get_cancel_hook(&self) -> Arc<Mutex<bool>> {
		self.cancel_hook.clone()
	}
	fn with_thread(mut self, thread_handle: std::thread::JoinHandle<HashMap<Taille, usize>>) -> Self {
		self.thread_handle = Some(thread_handle);
		self
	}
}
impl WidgetRef for ChandailScreen {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.title_top(Line::from(" Estimation de nombre de chandails ").white().centered())
			.border_set(border::THICK)
			.border_style(Style::new().white())
			.bg(ratatui::style::Color::Black)
			.title_bottom(Line::from(vec![" Appuyez sur ".gray(), "Esc".light_blue(), " pour annuler, ou sur ".gray(), "Entrée".light_blue(), " pour continuer ".gray()]).centered());
		let inner = block.inner(area);
		Clear.render(area, buf);
		block.render(area, buf);

		if let Some(results) = &self.results {
			let mut lines = Vec::new();
			for (taille, count) in results {
				lines.push(Line::from(vec![
					taille.to_string().light_blue(),
					": ".white(),
					count.to_string().green(),
				]));
			}
			Paragraph::new(lines)
				.wrap(Wrap { trim: false })
				.render(inner, buf);
		}
	}
}
impl Screen for ChandailScreen {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<AppState>) -> Result<UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				match key.code {
					cte::KeyCode::Esc => {
						// set the cancel flag and join the thread
						*self.cancel_hook.lock().expect("Poisoned Lock") = true;
						if let Some(handle) = self.thread_handle.take() {
							let res = handle.join();
							match res {
								Err(e) => {
									Ok(crate::ui::UpdateAction::ErrorPopUp(Box::new(UIError::Runtime { src: e })).one())
								},
								Ok(_hash_map) => {
									Ok(crate::ui::UpdateAction::Pop.one())
								},
							}
						} else {
							Ok(crate::ui::UpdateAction::Pop.one())
						}
					},
					cte::KeyCode::Enter => {
						// only exit if the results are in
						if self.results.is_some() {
							// set the cancel flag and join the thread just to make sure
							*self.cancel_hook.lock().expect("Poisoned Lock") = true;
							if let Some(handle) = self.thread_handle.take() {
								let res = handle.join();
								match res {
									Err(e) => {
										Ok(crate::ui::UpdateAction::ErrorPopUp(Box::new(UIError::Runtime { src: e })).one())
									},
									Ok(_hash_map) => {
										Ok(crate::ui::UpdateAction::Pop.one())
									},
								}
							} else {
								Ok(crate::ui::UpdateAction::Pop.one())
							}
						} else {
							Ok(crate::ui::UpdateAction::Continue.one())
						}
					},
					_ => {
						Ok(crate::ui::UpdateAction::Continue.one())
					},
				}
			},
			crate::ui::event::Event::Tick => {
				// check if the thread is done to take the results
				let is_done = self.thread_handle.as_ref().map(JoinHandle::is_finished).unwrap_or(false);
				if is_done {
					if let Some(handle) = self.thread_handle.take() {
						let res = handle.join();
						match res {
							Err(e) => {
								return Ok(crate::ui::UpdateAction::ErrorReplace(Box::new(UIError::Runtime { src: e })).one());
							},
							Ok(hash_map) => {
								self.results = Some(hash_map);
							},
						}
					}
				}
				Ok(crate::ui::UpdateAction::Continue.one())
			},
			_ => {
				Ok(crate::ui::UpdateAction::Continue.one())
			},
		}
	}
}