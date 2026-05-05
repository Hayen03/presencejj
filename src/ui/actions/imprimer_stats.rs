use std::{cell::Cell, collections::HashMap, sync::{Arc, Condvar, Mutex}, thread::JoinHandle};

use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::{Line, Span, Text}, widgets::WidgetRef};
use ratatui_textarea::TextArea;

use crate::{cdj::groupes::{self, Groupe, GroupeID, NULL_GROUPE}, data::stats::{AgeStats, GroupeStats, SemStats, SiteStats, Stats, StatsError, UniqueStats, VilleStats, fill_stats, get_unique_stats, print_stats_to_excel}, ui::{AppState, PollMenu, Screen, UIError, UpdateAction, screens::Desc}};
use crossterm::event as cte;

pub fn imprimer_stats(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let out_file = state.get_out_xlsx("Fichier de sortie");
	let out_file = if let Some(f) = out_file {
		f
	} else {
		return Ok(UpdateAction::ErrorPopUp(Box::new(UIError::CancelAction { desc: "Aucun Fichier Sélectionné".into() })).one());
	};

	let screen = StatsScreen::default();
	let step = screen.get_step();
	let signal = screen.get_signal();

	let thread = std::thread::spawn(move || {

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
				let new_step = Step::QueryCap { query: Query::new(grps) };
				*step.lock().expect("Poisoned Lock") = new_step;
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

		let do_annulation = PollMenu::poll_bool("Voulez-vous rentrez les nombres d'annulation?".into(), state.clone()).unwrap_or(false);
		let annulations: HashMap<GroupeID, usize> = if do_annulation {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			let grps = groupes.groupes().filter(|g| g.id != NULL_GROUPE.id && g.capacite.is_none()).collect::<Vec<_>>();
			if grps.is_empty() {
				HashMap::new()
			} else {
				let new_step = Step::QueryAnnulation { query: Query::new(grps) };
				*step.lock().expect("Poisoned Lock") = new_step;
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
			let grps = groupes.groupes().filter(|g| g.id != NULL_GROUPE.id && g.capacite.is_none()).collect::<Vec<_>>();
			if grps.is_empty() {
				HashMap::new()
			} else {
				let new_step = Step::QueryAttente { query: Query::new(grps) };
				*step.lock().expect("Poisoned Lock") = new_step;
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
		let (stats, gstats) = {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			let membres = state.membres.read().expect("Poisoned Lock");
			let comptes = state.comptes.read().expect("Poisoned Lock");
			fill_stats(groupes.groupes(), &membres, &comptes, &get_annulations, &get_attente, &get_missing_capacite)
		};
		let ustats = {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			get_unique_stats(groupes.groupes())
		};
		let logger = |msg: Desc| {};
		let print_result = {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			let config = state.config.read().expect("Poisoned Lock");
			print_stats_to_excel(&stats, &gstats, &groupes, &ustats, "stats.xlsx", &logger)
		};

		Ok(StatsResult { stats, gstats, ustats, status: print_result.is_ok(), print_err: print_result.err()})
	});

	Ok(crate::ui::UpdateAction::PushSub(Box::new(screen.with_thread(thread))).one())
}

#[derive(Debug, Default)]
struct Query<'a, T> {
	data: HashMap<GroupeID, T>,
	descs: HashMap<GroupeID, Arc<str>>,
	morph: HashMap<GroupeID, TextArea<'a>>,
	order: Vec<GroupeID>,
	at: usize,
	completed: bool,
}
impl<'a, T> Query<'a, T> where T: Default + ToString {
	fn new(mut groupes: Vec<&Groupe>) -> Self {
		groupes.sort_by_key(|a| a.desc());
		let data: HashMap<GroupeID, T> = groupes.iter().map(|g| (g.id, T::default())).collect();
		let descs = groupes.iter().map(|g| (g.id, g.desc().into())).collect();
		let morph = data.iter().map(|p| {
			let mut text_area = TextArea::default();
			text_area.set_yank_text(p.1.to_string());
			text_area.paste();
			(*p.0, text_area)
		}).collect();
		let order = groupes.iter().map(|g| g.id).collect();
		Self { data, descs, morph, order, at: 0, completed: false }
	}
}

#[derive(Debug, Default)]
enum Step<'a> {
	#[default]
	Start,
	Work,
	QueryCap{
		query: Query<'a, usize>,
	},
	//QueryDoAnnulation(Option<bool>),
	QueryAnnulation {
		query: Query<'a, usize>,
	},
	//QueryDoAttente(Option<bool>),
	QueryAttente {
		query: Query<'a, usize>,
	},
	Print,
	Done {
		result: Box<StatsResult>,
		text: Text<'a>,
		scroll: usize,
		current_max_scroll: Cell<Option<usize>>,
	},
}
impl<'a> Step<'a> {
	fn is_completed(&self) -> bool {
		match self {
			Self::QueryAnnulation { query, .. } | Self::QueryAttente { query, .. } | Self::QueryCap { query, .. } => query.completed,
			_ => true,
		}
	}
	fn replace(&mut self, new_step: Self) -> Self {
		std::mem::replace(self, new_step)
	}
}

#[derive(Debug)]
struct StatsResult {
	stats: Stats,
	#[allow(dead_code)]
	gstats: HashMap<GroupeID, GroupeStats>,
	ustats: UniqueStats,
	status: bool,
	print_err: Option<StatsError>,
}

#[derive(Debug, Default)]
struct StatsScreen<'a> {
	thread: Option<std::thread::JoinHandle<Result<StatsResult, UIError>>>,
	step: Arc<Mutex<Step<'a>>>,
	signal: Arc<Condvar>,
}
impl<'a> StatsScreen<'a> {
	fn with_thread(mut self, thread: std::thread::JoinHandle<Result<StatsResult, UIError>>) -> Self {
		self.thread = Some(thread);
		self
	}
	fn get_step(&self) -> Arc<Mutex<Step<'a>>> {
		self.step.clone()
	}
	fn get_signal(&self) -> Arc<Condvar> {
		self.signal.clone()
	}
}
impl<'a> WidgetRef for StatsScreen<'a> {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		match &*self.step.lock().expect("Poisoned Lock") {
			Step::QueryCap { query } => {
				show_query("Entrez les Capacités Manquantes", query, area, buf);
			},
			Step::QueryAnnulation { query } => {
				show_query("Entrez les Nombres d'Annulation", query, area, buf);
			},
			Step::QueryAttente { query } => {
				show_query("Entrez les Nombres de Participants en Attente", query, area, buf);
			},
			Step::Done { text, scroll, current_max_scroll, .. } => {},
			_ => {},
		}
	}
}
impl<'a> Screen for StatsScreen<'a> {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<AppState>) -> Result<super::UpdateActions, UIError> {
		match &mut *self.step.lock().expect("Poisoned Lock") {
			Step::QueryCap { query } => {
				handle_query(query, self.signal.clone(), event)
			},
			Step::QueryAnnulation { query } => {
				handle_query(query, self.signal.clone(), event)
			},
			Step::QueryAttente { query } => {
				handle_query(query, self.signal.clone(), event)
			},
			Step::Done { result, scroll, current_max_scroll, .. } => {
				match event {
					crate::ui::event::Event::Key(key) => {
						match key.code {
							cte::KeyCode::Esc | cte::KeyCode::Enter => {
								// close the screen
								Ok(UpdateAction::Pop.one())
							},
							cte::KeyCode::Up => {
								// scroll up
								*scroll = scroll.saturating_sub(1).min(current_max_scroll.get().unwrap_or(0));
								Ok(UpdateAction::Continue.one())
							},
							cte::KeyCode::Down => {
								// scroll down
								*scroll = scroll.saturating_add(1).min(current_max_scroll.get().unwrap_or(0));
								Ok(UpdateAction::Continue.one())
							},
							_ => Ok(UpdateAction::Continue.one()),
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
			_ => {
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
								let text = make_text(&result);
								*self.step.lock().expect("Poisoned Lock") = Step::Done { result: Box::new(result), text, scroll: 0, current_max_scroll: Cell::new(None) };
								Ok(UpdateAction::Continue.one())
							},
						}
					},
					_ => Ok(UpdateAction::Continue.one()),
				}
			},
		}
	}
}

fn handle_query(query: &mut Query<usize>, signal: Arc<Condvar>, event: crate::ui::event::Event) -> Result<super::UpdateActions, UIError> {
	match event {
		crate::ui::event::Event::Key(key) => {
			match key.code {
				cte::KeyCode::Enter => {
					// step 1: verify all inputs are valid
					for (gid, text_area) in &query.morph {
						let ln = text_area.lines().first().map(String::as_str).unwrap_or("0");
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
					// mark the query as completed without updating the data
					query.completed = true;
					// notify
					signal.notify_all();
					Ok(UpdateAction::Continue.one())
				},
				cte::KeyCode::Up => {
					query.at = query.at.saturating_sub(1).min(query.order.len());
					Ok(UpdateAction::Continue.one())
				},
				cte::KeyCode::Down => {
					query.at = query.at.saturating_add(1).min(query.order.len());
					Ok(UpdateAction::Continue.one())
				},
				e => {
					// pass the event to the current text area
					let current_gid = query.order.get(query.at).expect("Query order is empty") ;
					if let Some(text_area) = query.morph.get_mut(current_gid) {
						text_area.input(key);
					}
					Ok(UpdateAction::Continue.one())
				},
			}
		},
		_ => {
			Ok(UpdateAction::Continue.one())
		},
	}
}

fn show_query(title: &str, query: &Query<usize>, area: Rect, buf: &mut Buffer) {}

fn make_text<'a>(result: &StatsResult) -> Text<'a> {
	let mut text = Text::default();

	mk_unique_stats(&mut text, &result.ustats);
	text.push_line(Line::default());
	mk_stats(&mut text, &result.stats);

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
		Span::from("Inscriptions Totale: ").bold().light_blue(),
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