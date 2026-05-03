use std::{collections::HashMap, ffi::os_str::Display, sync::{Arc, Mutex}, thread::JoinHandle};

use ratatui::{buffer::Buffer, layout::Rect, style::{Style, Stylize}, symbols::border, text::{Line, Text}, widgets::{Block, Clear, Paragraph, Widget, WidgetRef, Wrap}};

use crossterm::event as cte;

use crate::{data::Taille, stats::{calcul_chandail, calcul_chandail_complex}, ui::{AppState, Poll, Screen, UIError, UpdateAction, actions::UpdateActions, screens::{InfoScreen, LineInputScreen, Menu, MenuItem}}};

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
			let screen = ChandailScreen::default();
			let cancel_hook = screen.get_cancel_hook();
			let thread_handle = std::thread::spawn(move || {
				let groupes = state.groupes.read().expect("Poisoned Lock");
				let membres = state.membres.read().expect("Poisoned Lock");
				Ok(calcul_chandail(&groupes, &membres))
			});
			Ok(vec![UpdateAction::Pop, UpdateAction::Push(Box::new(screen.with_thread(thread_handle)))])
		}) },
		MenuItem { id: EstimeChandailMethod::Complexe, action: Box::new(move |state| {
			let screen = ChandailScreen::default();
			let cancel_hook = screen.get_cancel_hook();
			let thread_handle = std::thread::spawn(move || {
				// gather the estimations from the user for each group category
				let cats = state.groupes.read().expect("Poisoned Lock").list_used_category();
				let mut estimations: HashMap<String, usize> = HashMap::new();
				let validation_fn = Arc::new(|s: &str| s.parse::<usize>().is_ok());
				for cat in cats {
					// early cancel if requested
					if *cancel_hook.lock().expect("Poisoned Lock") {
						return Ok(HashMap::new());
					}

					let prompt = format!("Estimation pour la catégorie '{}': ", cat);
					let poll = Poll {
						title: "Estimation de nombre de chandails".into(),
						prompt: Text::from(prompt),
						validation: Some(validation_fn.clone()),
						show_error: false,
					}.poll(state.clone());
					match poll {
						Err(e) => {
							// receiving failed, show error and quit action
							return Err(UIError::Runtime { src: Box::new(e) });
						},
						Ok(None) => {
							// user cancelled, stop the action but don't show an error
							return Ok(HashMap::new());
						},
						Ok(Some(s)) => {
							estimations.insert(cat.clone(), s.parse::<usize>().expect("Validation should have prevented this"));
						},
					}
				}

				let groupes = state.groupes.read().expect("Poisoned Lock");
				let membres = state.membres.read().expect("Poisoned Lock");
				Ok(calcul_chandail_complex(&groupes, &membres, &estimations))
			});
			Ok(vec![UpdateAction::Pop, UpdateAction::Push(Box::new(screen.with_thread(thread_handle)))])
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
	results: Option<Vec<(Taille, usize)>>,
	thread_handle: Option<std::thread::JoinHandle<Result<HashMap<Taille, usize>, UIError>>>,
}
impl ChandailScreen {
	fn get_cancel_hook(&self) -> Arc<Mutex<bool>> {
		self.cancel_hook.clone()
	}
	fn with_thread(mut self, thread_handle: std::thread::JoinHandle<Result<HashMap<Taille, usize>, UIError>>) -> Self {
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
							Ok(Ok(hash_map)) => {
								let mut results = hash_map.into_iter().collect::<Vec<_>>();
								results.sort_by(|a, b| a.cmp(b));
								self.results = Some(results);
							},
							Ok(Err(e)) => {
								return Ok(crate::ui::UpdateAction::ErrorReplace(Box::new(e)).one());
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