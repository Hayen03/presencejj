use std::{cell::Cell, collections::HashMap, sync::{Arc, Condvar, Mutex, RwLock}};

use lazy_static::lazy_static;
use ratatui::{buffer::Buffer, layout::{HorizontalAlignment, Rect}, style::{Color, Style, Stylize}, symbols::border, text::{Line, Span, Text}, widgets::{Block, Clear, Paragraph, Widget, WidgetRef, Wrap}};
use ratatui_textarea::TextArea;
use unicode_segmentation::UnicodeSegmentation;

use crate::{cdj::groupes::{Groupe, GroupeID, NULL_GROUPE}, data::stats::{AgeStats, FillStats, GroupeStats, SemStats, SiteStats, Stats, StatsError, StatsToExcel, UniqueStats, VilleStats, get_unique_stats}, ui::{AppState, FilePoll, PollMenu, Screen, UIError, UpdateAction, screens::{Desc, VIEW_TABLE_HEADER_BLOCK}}};
use crossterm::event as cte;

static QUERY_COL_GUTTER: u16 = 1;
static QUERY_INPUT_WIDTH: u16 = 8;
static QUERY_DESC_MIN_WIDTH: u16 = 20;

lazy_static!{
	pub static ref STATS_SCREEN_DEFAULT_TITLE: Line<'static> = Line::from(" Statistiques ").white().bold().centered();
	pub static ref STATS_SCREEN_INSTRUCTIONS: Line<'static> = Line::from(vec![
		" Appuyez sur ".gray(),
		"Esc".light_blue().bold(),
		" ou ".gray(),
		"Enter".light_blue().bold(),
		" pour fermer ".gray(),
	]).centered();
	pub static ref STATS_WORK_INSTRUCTIONS: Line<'static> = Line::from(vec![
		" Génération des statistiques en cours... Appuyez sur ".gray(),
		"Esc".light_blue().bold(),
		" pour annuler ".gray(),
	]).centered();
	pub static ref STATS_SCREEN_BLOCK: Block<'static> = Block::bordered()
		.border_set(border::THICK)
		.border_style(Style::new().white())
		.bg(Color::Black);
}

pub fn imprimer_stats(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let screen = StatsScreen::default();
	let step = screen.get_step();
	let signal = screen.get_signal();
	let new_query_flag = screen.get_new_query_flag();
	let cancel_hook = screen.get_cancel_hook();
	let progress_hook = screen.get_progress_hook();

	let thread = std::thread::spawn(move || {
		let out_file = FilePoll::save("Sélectionnez le fichier de sortie".into())
			.with_filter("xlsx", &["xlsx"])
			.poll(state.clone());
		let out_file = if let Some(out_file) = out_file {
			out_file
		} else {
			return Err(UIError::CancelAction { desc: String::from("Aucun fichier sélectionné") });
		};

		let descs: HashMap<GroupeID, Arc<str>> = {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			groupes.groupes().map(|g| (g.id, g.desc().into())).collect::<HashMap<_, _>>()
		};

		// query for missing capacities
		let missing_caps: HashMap<GroupeID, usize> = {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			let grps = groupes.groupes().filter(|g| g.id != NULL_GROUPE.id && g.capacite.is_none()).collect::<Vec<_>>();
			if grps.is_empty() { // skip if empty to avoid the round trip
				HashMap::new()
			} else {
				let query = Query::new(grps);
				// update the text area with the first value
				let new_step = Step::QueryCap { query };
				*step.lock().expect("Poisoned Lock") = new_step;
				*new_query_flag.lock().expect("Poisoned Lock") = true;
				// wait for completion
				let mut lock = step.lock().expect("Poisoned Lock");
				while !lock.is_completed() {
					lock = signal.wait(lock).expect("Poisoned Lock");
				}
				// retrieve and return the data
				match lock.replace(Step::Work) {
					Step::QueryCap { query } => query.data,
					_ => return Err(UIError::UnexpectedState { desc: format!("Expected Step::QueryCap, found {:?}", lock) }),
				}
			}
		};
		let get_missing_capacite = move |gid: GroupeID, _desc: &str| {
			missing_caps.get(&gid).copied().unwrap_or_default()
		};

		let do_annulation = PollMenu::poll_bool("Voulez-vous rentrez les nombres d'annulation?".into(), state.clone()).unwrap_or(false); // unwrap_or(false)
		let annulations: HashMap<GroupeID, usize> = if do_annulation {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			let grps = groupes.groupes().filter(|g| g.id != NULL_GROUPE.id).collect::<Vec<_>>();
			if grps.is_empty() {
				HashMap::new()
			} else {
				let new_step = Step::QueryAnnulation { query: Query::new(grps) };
				*step.lock().expect("Poisoned Lock") = new_step;
				*new_query_flag.lock().expect("Poisoned Lock") = true;
				// wait for completion
				let mut lock = step.lock().expect("Poisoned Lock");
				while !lock.is_completed() {
					lock = signal.wait(lock).expect("Poisoned Lock");
				}
				match lock.replace(Step::Work) {
					Step::QueryAnnulation { query } => query.data,
					_ => return Err(UIError::UnexpectedState { desc: format!("Expected Step::QueryAnnulation, found {:?}", lock) }),
				}
			}
		} else { HashMap::new() };
		let get_annulations = move |gid: GroupeID, _desc: &str| {
			annulations.get(&gid).copied()
		};

		let do_attente = PollMenu::poll_bool("Voulez-vous rentrez les nombres d'attente?".into(), state.clone()).unwrap_or(false);
		let attentes: HashMap<GroupeID, usize> = if do_attente {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			let grps = groupes.groupes().filter(|g| g.id != NULL_GROUPE.id).collect::<Vec<_>>();
			if grps.is_empty() {
				HashMap::new()
			} else {
				let new_step = Step::QueryAttente { query: Query::new(grps) };
				*step.lock().expect("Poisoned Lock") = new_step;
				*new_query_flag.lock().expect("Poisoned Lock") = true;
				// wait for completion
				let mut lock = step.lock().expect("Poisoned Lock");
				while !lock.is_completed() {
					lock = signal.wait(lock).expect("Poisoned Lock");
				}
				match lock.replace(Step::Work) {
					Step::QueryAttente { query } => query.data,
					_ => return Err(UIError::UnexpectedState { desc: format!("Expected Step::QueryAttente, found {:?}", lock) }),
				}
			}
		} else { HashMap::new() };
		let get_attente = move |gid: GroupeID, _desc: &str| {
			attentes.get(&gid).copied()
		};

		*step.lock().expect("Poisoned Lock") = Step::Print;
		*progress_hook.lock().expect("Poisoned Lock") = 0;
		let res = {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			let membres = state.membres.read().expect("Poisoned Lock");
			let comptes = state.comptes.read().expect("Poisoned Lock");
			let mut grps = groupes.groupes();
			FillStats {
				groupes: &mut grps,
				membres: &membres,
				comptes: &comptes,
				do_annulation: &get_annulations,
				do_attente: &get_attente,
				get_missing_capacite: &get_missing_capacite,
				progress: Some(progress_hook.clone()),
				cancel: Some(cancel_hook.clone()),
			}.fill()
		};
		let (stats, gstats) = match res {
			Some(res) => res,
			None => {
				return Err(UIError::CancelAction { desc: "Génération des statistiques annulée par l'utilisateur".into() });
			}
		};
		// check for cancel before next step
		if *cancel_hook.lock().expect("Poisoned Lock") {
			return Err(UIError::CancelAction { desc: "Génération des statistiques annulée par l'utilisateur".into() });
		}
		*progress_hook.lock().expect("Poisoned Lock") = 0;
		let ustats = {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			get_unique_stats(groupes.groupes(), Some(progress_hook.clone()), Some(cancel_hook.clone()))
		};
		let ustats = match ustats {
			Some(ustats) => ustats,
			None => {
				return Err(UIError::CancelAction { desc: "Génération des statistiques annulée par l'utilisateur".into() });
			}
		};
		// check for cancel before next step
		if *cancel_hook.lock().expect("Poisoned Lock") {
			return Err(UIError::CancelAction { desc: "Génération des statistiques annulée par l'utilisateur".into() });
		}
		*progress_hook.lock().expect("Poisoned Lock") = 0;
		let logger = |msg: Desc| {};
		let print_result = {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			let config = state.config.read().expect("Poisoned Lock");
			StatsToExcel {
				stats: &stats,
				gstats: &gstats,
				groupes: &groupes,
				ustats: &ustats,
				out: out_file.to_str().expect("Invalid Path"),
				logger: &logger,
				progress: Some(progress_hook.clone()),
				cancel: Some(cancel_hook.clone()),
			}.print()
		};

		Ok(StatsResult { stats, gstats, ustats, status: print_result.is_ok(), print_err: print_result.err()})
	});

	Ok(crate::ui::UpdateAction::Push(Box::new(screen.with_thread(thread))).one())
}

#[derive(Debug, Default)]
struct Query<T> {
	data: HashMap<GroupeID, T>,
	descs: HashMap<GroupeID, Arc<str>>,
	morph: HashMap<GroupeID, String>,
	order: Vec<GroupeID>,
	at: usize,
	completed: bool,
	scroll: Cell<usize>,
}
impl<T> Query<T> where T: Default + ToString {
	fn new(mut groupes: Vec<&Groupe>) -> Self {
		groupes.sort_by_key(|a| a.desc());
		let data: HashMap<GroupeID, T> = groupes.iter().map(|g| (g.id, T::default())).collect();
		let descs = groupes.iter().map(|g| (g.id, g.desc().into())).collect();
		let morph = data.iter().map(|p| {
			(*p.0, p.1.to_string())
		}).collect();
		let order = groupes.iter().map(|g| g.id).collect();
		Self { 
			data, 
			descs, 
			morph, 
			order, 
			at: 0, 
			completed: false, 
			scroll: Cell::new(0),
		}
	}
}

#[derive(Debug, Default)]
enum Step {
	#[default]
	Start,
	Work,
	QueryCap{
		query: Query<usize>,
	},
	//QueryDoAnnulation(Option<bool>),
	QueryAnnulation {
		query: Query<usize>,
	},
	//QueryDoAttente(Option<bool>),
	QueryAttente {
		query: Query<usize>,
	},
	Print,
	Done {
		result: Box<StatsResult>,
	},
}
impl Step {
	fn is_completed(&self) -> bool {
		match self {
			Self::QueryAnnulation { query, .. } | Self::QueryAttente { query, .. } | Self::QueryCap { query, .. } => query.completed,
			_ => true,
		}
	}
	fn replace(&mut self, new_step: Self) -> Self {
		std::mem::replace(self, new_step)
	}
	fn get_query(&self) -> Option<&Query<usize>> {
		match self {
			Self::QueryAnnulation { query, .. } | Self::QueryAttente { query, .. } | Self::QueryCap { query, .. } => Some(query),
			_ => None,
		}
	}
}

#[derive(Debug)]
struct StatsResult {
	stats: Stats,
	#[allow(dead_code)]
	gstats: HashMap<GroupeID, GroupeStats>,
	ustats: UniqueStats,
	#[allow(dead_code)]
	status: bool,
	print_err: Option<StatsError>,
}

#[derive(Debug, Default)]
struct StatsScreen {
	thread: Option<std::thread::JoinHandle<Result<StatsResult, UIError>>>,
	input: RwLock<TextArea<'static>>,
	step: Arc<Mutex<Step>>,
	signal: Arc<Condvar>,
	new_query_flag: Arc<Mutex<bool>>,
	cancel_hook: Arc<Mutex<bool>>,
	progress_hook: Arc<Mutex<u32>>,
	previous_progress: Cell<u32>,
	page: Option<Page>,
}
impl StatsScreen {
	fn with_thread(mut self, thread: std::thread::JoinHandle<Result<StatsResult, UIError>>) -> Self {
		self.thread = Some(thread);
		self
	}
	fn get_step(&self) -> Arc<Mutex<Step>> {
		self.step.clone()
	}
	fn get_signal(&self) -> Arc<Condvar> {
		self.signal.clone()
	}
	fn get_new_query_flag(&self) -> Arc<Mutex<bool>> {
		self.new_query_flag.clone()
	}
	fn get_cancel_hook(&self) -> Arc<Mutex<bool>> {
		self.cancel_hook.clone()
	}
	fn get_progress_hook(&self) -> Arc<Mutex<u32>> {
		self.progress_hook.clone()
	}

	fn sync_query_first_input(&self) {
		let mut lock = self.new_query_flag.lock().expect("Poisoned Lock");
		if *lock {
			*lock = false;
			if let Some(query) = self.step.lock().expect("Poisoned Lock").get_query() {
				if !query.order.is_empty() {
					let val = query.morph.get(query.order.first().expect("Query order is empty")).map(String::as_str).expect("Morph should have an entry for all gids");
					*self.input.write().expect("Poisoned Lock") = new_input(val);
				}
			}
		}
	}
}
impl WidgetRef for StatsScreen {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		self.sync_query_first_input();

		Clear.render(area, buf);
		let block = STATS_SCREEN_BLOCK.clone();
		match &*self.step.lock().expect("Poisoned Lock") {
			Step::QueryCap { query } => {
				show_query(" Entrez les Capacités Manquantes ", query, &self.input, block, area, buf);
			},
			Step::QueryAnnulation { query } => {
				show_query(" Entrez les Nombres d'Annulation ", query, &self.input, block, area, buf);
			},
			Step::QueryAttente { query } => {
				show_query(" Entrez les Nombres de Participants en Attente ", query, &self.input, block, area, buf);
			},
			Step::Done { result } => {
				if let Some(view) = &self.page {
					view.render_ref(area, buf);
				} else {
					let block = block
						.title_top(STATS_SCREEN_DEFAULT_TITLE.clone())
						.title_bottom(STATS_SCREEN_INSTRUCTIONS.clone());
					let inner = block.inner(area);
					block.render(area, buf);
					Paragraph::new("Génération des statistiques terminée!")
						.wrap(Wrap{trim: false})
						.render(inner, buf);
				}
			},
			_ => {
				// show loading screen
				let block = STATS_SCREEN_BLOCK.clone()
					.title_top(STATS_SCREEN_DEFAULT_TITLE.clone())
					.title_bottom(STATS_WORK_INSTRUCTIONS.clone());
				let loading = Line::from(vec![
					"Génération des statistiques en cours... ".yellow(),
					format!("Progess: {}/?", *self.progress_hook.lock().expect("Poisoned Lock")).light_blue(),
				]);
				Paragraph::new(loading).block(block).wrap(Wrap { trim: false }).render(area, buf);
			},
		}

		// update previous progress
		self.previous_progress.set(*self.progress_hook.lock().expect("Poisoned Lock"));
	}
}
impl Screen for StatsScreen {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<AppState>) -> Result<super::UpdateActions, UIError> {
		self.sync_query_first_input();

		let current_progress = *self.progress_hook.lock().expect("Poisoned Lock");
		let previous_progress = self.previous_progress.get();

		match &mut *self.step.lock().expect("Poisoned Lock") {
			Step::QueryCap { query } => {
				// update the current morph with the input
				let current_gid = query.order.get(query.at);
				if let Some(current_gid) = current_gid {
					let line = self.input.read().expect("Poisoned Lock").lines().first().cloned();
					if let Some(line) = line {
						query.morph.insert(*current_gid, line);
					}
				}
				handle_query(query, self.signal.clone(), &mut self.input.write().expect("Poisoned Lock"), event, current_progress, previous_progress)
			},
			Step::QueryAnnulation { query } => {
				let current_gid = query.order.get(query.at);
				if let Some(current_gid) = current_gid {
					let line = self.input.read().expect("Poisoned Lock").lines().first().cloned();
					if let Some(line) = line {
						query.morph.insert(*current_gid, line);
					}
				}
				handle_query(query, self.signal.clone(), &mut self.input.write().expect("Poisoned Lock"), event, current_progress, previous_progress)
			},
			Step::QueryAttente { query } => {
				let current_gid = query.order.get(query.at);
				if let Some(current_gid) = current_gid {
					let line = self.input.read().expect("Poisoned Lock").lines().first().cloned();
					if let Some(line) = line {
						query.morph.insert(*current_gid, line);
					}
				}
				handle_query(query, self.signal.clone(), &mut self.input.write().expect("Poisoned Lock"), event, current_progress, previous_progress)
			},
			Step::Done { result } => {
				match event {
					crate::ui::event::Event::Key(key) => {
						if let Some(view) = &mut self.page {
							view.handle_event(event, state)
						} else {
							Ok(UpdateAction::Continue.one())
						}
					},
					crate::ui::event::Event::Tick => {
						if let Some(err) = result.print_err.take() {
							return Ok(UpdateAction::ErrorPopUp(Box::new(err)).one());
						}
						Ok(UpdateAction::Continue.one())
					},
					_ => Ok(UpdateAction::Continue.one()),
				}
			},
			step => {
				match event {
					crate::ui::event::Event::Tick => {
						// check if the thread has finished
						if let Some(thread) = &mut self.thread {
							if !thread.is_finished() {
								return Ok(UpdateAction::Continue.one());
							}
						} else {
							return Ok(vec![UpdateAction::Pop, UpdateAction::ErrorPopUp(Box::new(UIError::UnexpectedState { desc: "Thread not set".into() }))]);
						}
						let thread = self.thread.take().expect("Thread should be set");
						match thread.join() {
							Err(err) => { // thread panicked
								Ok(vec![UpdateAction::Pop, UpdateAction::ErrorPopUp(Box::new(UIError::Runtime { src: err }))])
							},
							Ok(Err(err)) => { // thread returned an error
								Ok(vec![UpdateAction::Pop, UpdateAction::ErrorPopUp(Box::new(err))])
							},
							Ok(Ok(result)) => { // thread completed successfully
								let page = Page::new(&result);
								*step = Step::Done { result: Box::new(result) };
								let _ = self.page.insert(page);
								Ok(UpdateAction::Redraw.one())
							},
						}
					},
					_ => Ok(UpdateAction::Continue.one()),
				}
			},
		}
	}
}

fn handle_query(query: &mut Query<usize>, signal: Arc<Condvar>, input: &mut TextArea, event: crate::ui::event::Event, current_progress: u32, previous_progress: u32) -> Result<super::UpdateActions, UIError> {
	match event {
		crate::ui::event::Event::Key(key) => {
			match key.code {
				cte::KeyCode::Enter => {
					// step 1: verify all inputs are valid
					for (gid, ln) in &query.morph {
						let ln = ln.trim();
						let ln = if ln.is_empty() {"0"} else {ln};
						if let Ok(n) = ln.parse::<usize>() {
							query.data.insert(*gid, n);
						} else {
							return Ok(UpdateAction::ErrorPopUp(Box::new(UIError::Runtime { src: Box::new(format!("{ln} n'est pas un nombre (groupe {})", query.descs.get(gid).map(Arc::as_ref).unwrap_or("???"))) })).one());
						}
					}
					// mark the query as completed
					query.completed = true;
					// notify
					signal.notify_all();
					Ok(UpdateAction::Continue.one())
				},
				cte::KeyCode::Esc => {
					// mark the query as completed without updating the data (in fact, reset it in case enter was pressed before)
					for (gid, val) in &mut query.data {
						*val = 0;
					}
					query.completed = true;
					// notify
					signal.notify_all();
					Ok(UpdateAction::Continue.one())
				},
				cte::KeyCode::Up => {
					let new_pos = query.at.saturating_sub(1);
					move_cursor(query, new_pos, input);
					Ok(UpdateAction::Redraw.one())
				},
				cte::KeyCode::Down => {
					let new_pos = query.at.saturating_add(1);
					move_cursor(query, new_pos, input);
					Ok(UpdateAction::Redraw.one())
				},
				_e => {
					// pass the event to the current text area
					let current_gid = query.order.get(query.at).expect("Query order is empty") ;
					input.input(key);
					Ok(UpdateAction::Redraw.one())
				},
			}
		},
		crate::ui::event::Event::Tick
			if current_progress != previous_progress => {
				Ok(UpdateAction::Redraw.one())
			},
		_ => {
			Ok(UpdateAction::Continue.one())
		},
	}
}

fn show_query(title: &str, query: &Query<usize>, input: &RwLock<TextArea>, block: Block, area: Rect, buf: &mut Buffer) {
	let block = block
		.title_top(Line::from(title).white().bold().centered())
		.title_bottom(Line::from(vec![
			" Appuyez sur ".gray(),
			"Esc".light_blue().bold(),
			" pour annuler ou ".gray(),
			"Enter".light_blue().bold(),
			" pour valider ".gray(),
		]).centered());
	let inner = block.inner(area);
	block.render(area, buf);

	// get scroll value
	let current_scroll = query.scroll.get();
	let current_scroll_end = current_scroll.saturating_add(inner.height as usize).min(query.order.len());
	// clamp/move the view window to fit the cursor if needed
	let scroll = {
		if query.at < current_scroll {
			query.at
		} else if query.at >= current_scroll_end {
			query.at.saturating_sub(inner.height as usize - 1)
		} else {
			current_scroll
		}
	};
	query.scroll.set(scroll);
	let scroll_end = scroll.saturating_add(inner.height as usize).min(query.order.len());
	let view = &query.order[scroll..scroll_end];

	let desc_width = inner.width.saturating_sub(QUERY_INPUT_WIDTH).saturating_sub(QUERY_COL_GUTTER);
	if desc_width < QUERY_DESC_MIN_WIDTH {
		// special rendering: show only the desc, but when the cursor is on the line show the input instead
		for (i, gid) in view.iter().enumerate() {
			let real_i = scroll + i;
			let desc_area = Rect {
				x: inner.x,
				y: inner.y + i as u16,
				width: inner.width,
				height: 1,
			};
			let mini_desc_area = Rect {
				x: inner.x,
				y: inner.y + i as u16,
				width: desc_width,
				height: 1,
			};
			let input_area = Rect {
				x: inner.x + desc_width + QUERY_COL_GUTTER,
				y: inner.y + i as u16,
				width: QUERY_INPUT_WIDTH,
				height: 1,
			};
			let desc = query.descs.get(gid).map(Arc::as_ref).unwrap_or("???");
			if real_i == query.at {
				let par = Paragraph::new(desc);
				let par = if par.line_width() > mini_desc_area.width as usize - 3 {
					let s = desc.grapheme_indices(true).nth(mini_desc_area.width as usize - 3);
					if let Some((idx, _)) = s {
						Paragraph::new(format!("{}...", &desc[..idx]))
					} else { par }
				} else { par };
				par.gray().on_black().render(mini_desc_area, buf);
				// render the input on top of the desc
				let mut input = input.write().expect("Poisoned Lock");
				let style = if input.lines().first().map(|s| {
					let trimed = s.trim();
					trimed.is_empty() || trimed.parse::<usize>().is_ok()
				}).unwrap_or(false) {
					Style::new().green().on_gray()
				} else {
					Style::new().red().on_gray()
				};
				input.set_style(style);
				input.render(input_area, buf);
			} else {
				let par = Paragraph::new(desc);
				let par = if par.line_width() > desc_area.width as usize - 3 {
					let s = desc.grapheme_indices(true).nth(desc_area.width as usize - 3);
					if let Some((idx, _)) = s {
						Paragraph::new(format!("{}...", &desc[..idx]))
					} else { par }
				} else { par };
				par.gray().on_black().render(desc_area, buf);
			}
		}
	} else {
		// normal rendering: show the desc on the left and the input on the right
		for (i, gid) in view.iter().enumerate() {
			let real_i = scroll + i;
			let desc_area = Rect {
				x: inner.x,
				y: inner.y + i as u16,
				width: desc_width,
				height: 1,
			};
			let input_area = Rect {
				x: desc_area.x + desc_area.width + QUERY_COL_GUTTER,
				y: desc_area.y,
				width: QUERY_INPUT_WIDTH,
				height: 1,
			};
			let desc = query.descs.get(gid).map(Arc::as_ref).unwrap_or("???");
			let par = Paragraph::new(desc);
			let par = if par.line_width() > desc_area.width as usize - 3 {
				let s = desc.grapheme_indices(true).nth(desc_area.width as usize - 3);
				if let Some((idx, _)) = s {
					Paragraph::new(format!("{}...", &desc[..idx]))
				} else { par }
			} else { par };
			if real_i == query.at {
				let desc_area = Rect {
					width: desc_area.width + QUERY_COL_GUTTER,
					..desc_area
				}; // to color the gutter as well
				par.white().on_gray().render(desc_area, buf);
				let mut input = input.write().expect("Poisoned Lock");
				let style = if input.lines().first().map(|s| {
					let trimed = s.trim();
					trimed.is_empty() || trimed.parse::<usize>().is_ok()
				}).unwrap_or(false) {
					Style::new().green().on_gray()
				} else {
					Style::new().red().on_gray()
				};
				input.set_style(style);
				input.render(input_area, buf);
			} else { // normal line
				par.gray().on_black().render(desc_area, buf);
				let val = query.morph.get(gid).map(String::as_str).expect("Group should exists").gray().on_black().into_right_aligned_line();
				Paragraph::new(val).render(input_area, buf);
			}
		}
	}
}

fn move_cursor(query: &mut Query<usize>, new_pos: usize, input: &mut TextArea) {
	if query.order.is_empty() {
		return;
	}
	let new_pos = new_pos.min(query.order.len()-1);
	if query.at != new_pos {
		let gid = query.order.get(query.at).expect("Query order is empty");
		// put the current input into the morph map
		let line = input.lines().first().map(|s| s.trim().into()).unwrap_or_default();
		query.morph.insert(*gid, line);
		// move the cursor
		query.at = new_pos;
		// update the input with the new morph
		let gid = query.order.get(query.at).expect("Query order is empty");
		let line = query.morph.get(gid).expect("Morph should have an entry for all gids").clone();
		*input = new_input(&line);
	}
}

fn new_input<'a>(starting_text: &str) -> TextArea<'a> {
	let mut input = TextArea::new(vec![starting_text.into()]);
	input.set_style(Style::new().white().on_gray().bold());
	input.set_alignment(HorizontalAlignment::Right);
	input.move_cursor(ratatui_textarea::CursorMove::End);
	input
}

#[allow(dead_code)]
fn make_text<'a>(result: &StatsResult) -> Text<'a> {
	let mut text = Text::default();

	mk_unique_stats(&mut text, &result.ustats);
	text.push_line(Line::default());
	mk_stats(&mut text, &result.stats);

	text.push_line(Line::default());
	if result.status {
		text.push_line(Line::from("Le fichier a été enregistré!").green());
	} else {
		text.push_line(Line::from("Le fichier n'a pu être enregistré...").red());
	}
	
	text
}

fn mk_unique_stats(text: &mut Text, ustats: &UniqueStats) {
	text.push_line(Line::from("STATISTIQUES GÉNÉRALES").green());
	text.push_line(Line::from("======================").green());
	text.push_line(Line::from(vec![
		Span::from("Nombre de Semaine de Camp: ").bold().light_blue(),
		Span::from(ustats.sems.len().to_string()),
	]));
	text.push_line(Line::from(vec![
		Span::from("Enfants Uniques: ").bold().light_blue(),
		Span::from(ustats.total.to_string()),
	]));
	text.push_line(Line::from("Participants Total Par Groupe d'Âge: ").bold().blue());
	for (age, count) in &ustats.ages {
		text.push_line(Line::from(vec![
			format!("\t{age}: ").bold().white(),
			Span::from(count.to_string()),
		]));
	}
	text.push_line(Line::from("Participants Total Par Site: ").bold().blue());
	for (site, count) in &ustats.sites {
		text.push_line(Line::from(vec![
			format!("\t{site}: ").bold().white(),
			Span::from(count.to_string()),
		]));
	}
}
fn mk_stats(text: &mut Text, stats: &Stats) {

	mk_ville_stats(text, &stats.villes);
	
	text.push_line(Line::default());
	text.push_line(Line::from("Statistiques Par Site").green());
	text.push_line(Line::from("=====================").green());
	mk_site_stats(text, &stats.sites);
	text.push_line(Line::default());
	mk_site_stats_inner(text, "Total", &stats.total());
}
fn mk_ville_stats(text: &mut Text, stats: &VilleStats) {
	text.push_line(Line::from("Statistiques des Villes").green());
	text.push_line(Line::from("=======================").green());
	for (ville, vstats) in &stats.villes {
		text.push_line(Line::from(vec![
			format!("{ville}: ").white(),
			vstats.to_string().into(),
		]));
	}
	text.push_line(Line::from(vec![
		Span::from("Autres: ").white(),
		stats.autres.to_string().into(),
	]));
	text.push_line(Line::from(vec![
		Span::from("Inconnu: ").white(),
		stats.inconnues.to_string().into(),
	]));
	text.push_line(Line::from(vec![
		Span::from("Total: ").yellow(),
		stats.total.to_string().into(),
	]));
}
fn mk_site_stats(text: &mut Text, stats: &HashMap<String, SiteStats>) {
	for (site, stats) in stats {
		mk_site_stats_inner(text, site, stats);
		text.push_line(Line::default());
	}
}
fn mk_site_stats_inner(text: &mut Text, site: &str, stats: &SiteStats) {
	text.push_line(Line::from(site.to_string()).bold().light_blue());
	let mut v = stats.villes.villes.iter().map(|p| (p.0.as_str(), *p.1)).collect::<Vec<_>>();
	v.sort_by_key(|p| p.0);
	if stats.villes.autres > 0 {
		v.push(("Autres", stats.villes.autres));
	}
	if stats.villes.inconnues > 0 {
		v.push(("Inconnues", stats.villes.inconnues));
	}
	v.push(("Total", stats.villes.total));
	let mut first = true;
	let mut line = vec![
		Span::from("Répartition par ville: ").white().bold(),
	];
	for (ville, val) in v {
		if !first {
			line.push(Span::from(", "));
		}
		first = false;
		line.push(Span::from(format!("{ville} (")).white());
		line.push(Span::from(val.to_string()));
		line.push(Span::from(")").white());
	}
	text.push_line(Line::from(line));

	for (age, astats) in &stats.ages {
		mk_age_stats(text, age, astats);
	}
	mk_age_stats(text, "Total", &stats.total());

}
fn mk_age_stats(text: &mut Text, age: &str, astats: &AgeStats) {
	text.push_line(Line::from(format!("\t{age}")).bold().cyan());
	let mut sems = astats.semaines.keys().map(|s| s.as_str()).collect::<Vec<_>>();
	sems.sort();
	for sem in sems {
		if let Some(stats) = astats.semaines.get(sem) {
			mk_sem_stats(text, sem, stats);
		}
	}
	mk_sem_stats(text, "Total", &astats.total());
}
fn mk_sem_stats(text: &mut Text, sem: &str, stats: &SemStats) {
	text.push_line(Line::from(vec![
		format!("\t\t{sem}: ").white().bold(),
		"Capacite (".white(),
		stats.capacite.to_string().into(),
		"), Inscriptions (".white(),
		stats.inscriptions.to_string().into(),
		"), Annulations (".white(),
		stats.annulations.to_string().into(),
		"), Attente (".white(),
		stats.liste_attente.to_string().into(),
		")".white(),
	]));
}

#[derive(Debug)]
struct View {
	text: Text<'static>,
	scroll: Cell<u16>,
}
impl View {
	fn general(stats: &Stats, ustats: &UniqueStats, status: bool) -> Self {
		let mut text = Text::default();
		mk_unique_stats(&mut text, ustats);
		text.push_line(Line::default());
		mk_ville_stats(&mut text, &stats.villes);

		text.push_line(Line::default());
		if status {
			text.push_line(Line::from("Le fichier a été enregistré!").green());
		} else {
			text.push_line(Line::from("Le fichier n'a pu être enregistré...").red());
		}

		Self {
			text,
			scroll: Cell::new(0),
		}
	}
	fn site(stats: &SiteStats, site: &str) -> Self {
		let mut text = Text::default();
		mk_site_stats_inner(&mut text, site, stats);
		Self {
			text,
			scroll: Cell::new(0),
		}
	}
	fn reset(&mut self) {
		self.scroll.set(0);
	}
}
impl WidgetRef for View {
	fn render_ref(&self,area: Rect,buf: &mut Buffer) {
		let par = Paragraph::new(self.text.clone()).wrap(Wrap { trim: false });
		let h = par.line_count(area.width);
		let max_scroll = h.saturating_sub(area.height as usize);
		let scroll = self.scroll.get().min(max_scroll as u16);
		par.scroll((scroll, 0)).render(area, buf);
		self.scroll.set(scroll);
	}
}
impl Screen for View {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<AppState>) -> Result<super::UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Esc => {
						// close the screen
						Ok(UpdateAction::Pop.one())
					},
					cte::KeyCode::Up => {
						// scroll up
						self.scroll.set(self.scroll.get().saturating_sub(1));
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Down => {
						// scroll down
						self.scroll.set(self.scroll.get().saturating_add(1));
						Ok(UpdateAction::Redraw.one())
					},
					_ => Ok(UpdateAction::Continue.one()),
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}

#[derive(Debug)]
struct Tab {
	label: String,
	view: View,
}

#[derive(Debug)]
struct Page {
	tabs: Vec<Tab>,
	current_tab: usize,
}
impl Page {
	fn new(result: &StatsResult) -> Self {
		let mut tabs = vec![
			Tab {
				label: "Général".into(),
				view: View::general(&result.stats, &result.ustats, result.status),
			}
		];
		let mut sites = result.stats.sites.keys().cloned().collect::<Vec<_>>();
		sites.sort();
		for site in sites {
			if let Some(sstats) = result.stats.sites.get(&site) {
				tabs.push(Tab {
					label: site.clone(),
					view: View::site(sstats, &site),
				});
			}
		}
		// total
		tabs.push(Tab {
			label: "Total".into(),
			view: View::site(&result.stats.total(), "Total"),
		});
		Self {
			tabs,
			current_tab: 0,
		}
	}

	fn stylize_tab<'a>(label: &'a str, selected: &str) -> Span<'a> {
		if label == selected {
			Span::from(label).light_blue().bold().on_dark_gray()
		} else {
			Span::from(label).white()
		}
	}
}
impl WidgetRef for Page {
	fn render_ref(&self,area: Rect,buf: &mut Buffer) {
		let block = STATS_SCREEN_BLOCK.clone();
		let inner = block.inner(area);

		let label_block = VIEW_TABLE_HEADER_BLOCK.clone();
		let tabs = self.tabs.iter().map(|t| Self::stylize_tab(&t.label, &self.tabs[self.current_tab].label)).intersperse(Span::from(" | ").gray()).collect::<Vec<_>>();
		let tabs = Line::from(tabs).centered();
		let tabs = Paragraph::new(tabs).block(label_block);
		let label_area = Rect {
			x: inner.x,
			y: inner.y,
			width: inner.width,
			height: 2,
		};

		Clear.render(area, buf);
		block.render(area, buf);
		tabs.render(label_area, buf);
		let view_area = Rect {
			x: inner.x,
			y: inner.y + label_area.height,
			width: inner.width,
			height: inner.height.saturating_sub(label_area.height),
		};
		self.tabs[self.current_tab].view.render_ref(view_area, buf);
	}
}
impl Screen for Page {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<AppState>) -> Result<super::UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Esc => Ok(UpdateAction::Pop.one()),
					cte::KeyCode::Tab => {
						self.current_tab = (self.current_tab + 1) % self.tabs.len();
						self.tabs[self.current_tab].view.reset();
						Ok(UpdateAction::Redraw.one())
					},
					_ => self.tabs[self.current_tab].view.handle_event(event, state),
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}