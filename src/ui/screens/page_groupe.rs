use std::{cell::Cell, collections::HashMap, sync::{Arc, Mutex, RwLock}};

use lazy_static::lazy_static;
use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::{Style, Stylize}, text::{Line, Span}, widgets::{Paragraph, Row, StatefulWidget, Table, TableState, Widget, WidgetRef, Wrap}};

use crate::{cdj::{groupes::{Groupe, GroupeID, SousGroupe}, membres::{Interet, Membre, MembreID, MembreReg}}, data::Genre, prelude::Date, ui::{AppState, Screen, UIError, UpdateAction, actions::Action, fit_str_width, screens::{Field, FieldBlock, FieldBlockCluster, FieldType, Menu, MenuItem, VIEW_TABLE_BLOCK, VIEW_TABLE_HEADER_BLOCK, stylize_selection}}};

lazy_static!{
	pub static ref PAGE_GROUPE_MEMBRE_TABLE_HEADERS: Row<'static> = Row::new(vec![
		Line::from("Nom").white().bold(),
		Line::from("Prenom").white().bold(),
		Line::from("Naissance").white().bold(),
		Line::from("Genre").white().bold(),
		Line::default(), // extra column to fill space
	]);
}

#[derive(Debug)]
struct GeneralView {
	saison: Arc<RwLock<Field>>,
	activite: Arc<RwLock<Field>>,
	site: Arc<RwLock<Field>>,
	categorie: Arc<RwLock<Field>>,
	semaine: Arc<RwLock<Field>>,
	discriminant: Arc<RwLock<Field>>,
	cluster: FieldBlockCluster,
	ordre: Vec<Arc<RwLock<Field>>>,
	sel: Option<usize>,
	scroll: Cell<u16>,
	dirty_flag: Arc<Mutex<bool>>,
}
impl GeneralView {
	fn new(groupe: &Groupe, dirty_flag: Arc<Mutex<bool>>) -> Self {
		let mut block = FieldBlock::default();
		let saison = block.add_field(Field::from(groupe.saison.as_deref()).with_label("Saison".into()));
		let activite = block.add_field(Field::from(groupe.activite.as_deref()).with_label("Activité".into()));
		let site = block.add_field(Field::from(groupe.site.as_deref()).with_label("Site".into()));
		let categorie = block.add_field(Field::from(groupe.category.as_deref()).with_label("Catégorie".into()));
		let semaine = block.add_field(Field::from(groupe.semaine.as_deref()).with_label("Semaine".into()));
		let discriminant = block.add_field(Field::from(groupe.discriminant.as_deref()).with_label("Discriminant".into()));
		let cluster = FieldBlockCluster::new(vec![block]);
		let ordre = vec![
			saison.clone(),
			activite.clone(),
			site.clone(),
			categorie.clone(),
			semaine.clone(),
			discriminant.clone(),
		];
		Self {
			saison,
			activite,
			site,
			categorie,
			semaine,
			discriminant,
			cluster,
			ordre,
			sel: None,
			scroll: Cell::new(0),
			dirty_flag,
		}
	}
	fn update(&self, groupe: &Groupe) {
		self.saison.write().expect("Poisoned Lock").set_value(FieldType::from(groupe.saison.as_deref()));
		self.activite.write().expect("Poisoned Lock").set_value(FieldType::from(groupe.activite.as_deref()));
		self.site.write().expect("Poisoned Lock").set_value(FieldType::from(groupe.site.as_deref()));
		self.categorie.write().expect("Poisoned Lock").set_value(FieldType::from(groupe.category.as_deref()));
		self.semaine.write().expect("Poisoned Lock").set_value(FieldType::from(groupe.semaine.as_deref()));
		self.discriminant.write().expect("Poisoned Lock").set_value(FieldType::from(groupe.discriminant.as_deref()));
	}
	fn reset(&mut self) {
		self.sel = None;
		self.scroll.set(0);
	}
	fn build_groupe(&self, groupe: &mut Groupe) {
		groupe.saison = self.saison.read().expect("Poisoned Lock").get_str().flatten().map(|s| s.to_string());
		groupe.activite = self.activite.read().expect("Poisoned Lock").get_str().flatten().map(|s| s.to_string());
		groupe.site = self.site.read().expect("Poisoned Lock").get_str().flatten().map(|s| s.to_string());
		groupe.category = self.categorie.read().expect("Poisoned Lock").get_str().flatten().map(|s| s.to_string());
		groupe.semaine = self.semaine.read().expect("Poisoned Lock").get_str().flatten().map(|s| s.to_string());
		groupe.discriminant = self.discriminant.read().expect("Poisoned Lock").get_str().flatten().map(|s| s.to_string());
	}
}
impl WidgetRef for GeneralView {
	fn render_ref(&self,area: ratatui::prelude::Rect,buf: &mut ratatui::prelude::Buffer) {
		let selected = self.sel.map(|s| self.ordre[s].clone());
		let (text, scroll) = self.cluster.get_text_and_scroll(selected, area.height, area.width, self.scroll.get());
		Paragraph::new(text)
			.wrap(Wrap { trim: false })
			.scroll((scroll, 0))
			.render(area, buf);
		self.scroll.set(scroll);
	}
}
impl Screen for GeneralView {
	fn handle_event(&mut self, event: crate::ui::event::Event, _state: Arc<AppState>) -> Result<crate::ui::actions::UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Up => {
						let sel = self.sel.unwrap_or(0).saturating_sub(1);
						self.sel = Some(sel);
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Down => {
						let sel = self.sel.map(|s| s.saturating_add(1).min(self.ordre.len() - 1)).unwrap_or(0);
						self.sel = Some(sel);
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Enter => {
						if let Some(sel) = self.sel {
							Ok(Field::on_action(self.ordre[sel].clone(), Some(self.dirty_flag.clone())))
						} else {
							Ok(UpdateAction::Continue.one())
						}
					},
					_ => Ok(UpdateAction::Continue.one()),
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}

#[derive(Debug)]
struct SousGroupeData {
	id: u32,
	profil: Arc<RwLock<Field>>,
	animateur: Arc<RwLock<Field>>,
	cluster: FieldBlockCluster,
	ordre: Vec<Arc<RwLock<Field>>>,
}
impl SousGroupeData {
	fn new(id: u32, profil: Option<Interet>, animateur: Option<&str>) -> Self {
		let mut block = FieldBlock::default();
		let profil = block.add_field(Field::from(profil).with_label("Profil".into()));
		let animateur = block.add_field(Field::from(animateur).with_label("Animateur".into()));
		let cluster = FieldBlockCluster::new(vec![block]);
		let ordre = vec![
			profil.clone(),
			animateur.clone(),
		];
		Self {
			id,
			profil,
			animateur,
			cluster,
			ordre,
		}
	}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SousGroupeSel {
	None,
	Fields(usize),
	Membres(usize),
}
impl SousGroupeSel {
	fn next(&mut self, fields: usize, membres: usize) {
		*self = match self {
			Self::None => {
				match (fields, membres) {
					(0, 0) => Self::None,
					(0, m) => Self::Membres(0),
					(f, _) => Self::Fields(0),
				}
			},
			Self::Fields(idx) => {
				let nidx = idx.saturating_add(1);
				if nidx < fields {
					Self::Fields(nidx)
				} else if membres > 0 {
					Self::Membres(0)
				} else {
					Self::Fields(*idx)
				}
			},
			Self::Membres(idx) => {
				let nidx = idx.saturating_add(1).min(membres - 1);
				Self::Membres(nidx)
			},
		};
	}
	fn previous(&mut self, fields: usize, membres: usize) {
		*self = match self {
			Self::None => {
				match (fields, membres) {
					(0, 0) => Self::None,
					(0, m) => Self::Membres(0),
					(f, _) => Self::Fields(0),
				}
			},
			Self::Fields(idx) => {
				let nidx = idx.saturating_sub(1);
				Self::Fields(nidx)
			},
			Self::Membres(idx) => {
				if *idx == 0 {
					Self::Fields(fields - 1)
				} else if fields > 0 {
					let nidx = idx.saturating_sub(1);
					Self::Membres(nidx)
				} else {
					Self::Membres(*idx)
				}
			},
		}
	}
}
#[derive(Debug)]
struct MembreLine {
	id: MembreID,
	nom: String,
	prenom: String,
	naissance: Date,
	age: u8,
	genre: Option<Genre>,
}
impl MembreLine {
	fn new(membre: &Membre) -> Self {
		let age = membre.age();
		Self {
			id: membre.id,
			nom: membre.nom.clone(),
			prenom: membre.prenom.clone(),
			naissance: membre.naissance,
			age,
			genre: membre.genre,
		}
	}
	fn to_row<'a>(&'a self) -> Row<'a> {
		Row::new([
			Line::from(self.nom.as_str()),
			Line::from(self.prenom.as_str()),
			Line::from(format!("{} ({} ans)", self.naissance.format("%Y-%m-%d"), self.age)),
			Line::from(self.genre.map_or("".to_string(), |g| g.to_string())),
			Line::default(), // fill space
		])
	}
}
impl PartialEq for MembreLine {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}
impl Eq for MembreLine {}
impl PartialOrd for MembreLine {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for MembreLine {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		(&self.nom, &self.prenom).cmp(&(&other.nom, &other.prenom))
	}
}
#[derive(Debug)]
struct MembreView {
	gid: GroupeID,
	sg: Option<SousGroupeData>,
	membres: Vec<MembreLine>,
	sel: SousGroupeSel,
	table_state: RwLock<TableState>,
	widths: [Constraint; 5],
	dirty_flag: Arc<Mutex<bool>>,
}
impl MembreView {
	fn new(gid: GroupeID, sg: Option<SousGroupeData>, membres: &[&Membre], dirty_flag: Arc<Mutex<bool>>) -> Self {
		let mut membres = membres.iter().map(|m| MembreLine::new(m)).collect::<Vec<_>>();
		membres.sort();
		Self {
			gid,
			sg,
			membres,
			sel: SousGroupeSel::None,
			table_state: RwLock::new(TableState::default()),
			widths: [Constraint::Percentage(20); 5],
			dirty_flag,
		}
	}
	fn next(&mut self) {
		let n_fields = if self.sg.is_some() { 2 } else { 0 };
		let n_membres = self.membres.len();
		self.sel.next(n_fields, n_membres);
		if let SousGroupeSel::Membres(n) = self.sel {
			self.table_state.write().expect("Poisoned Lock").select(Some(n));
		} else {
			self.table_state.write().expect("Poisoned Lock").select(None);
		}
	}
	fn previous(&mut self) {
		let n_fields = if self.sg.is_some() { 2 } else { 0 };
		let n_membres = self.membres.len();
		self.sel.previous(n_fields, n_membres);
		if let SousGroupeSel::Membres(n) = self.sel {
			self.table_state.write().expect("Poisoned Lock").select(Some(n));
		} else {
			self.table_state.write().expect("Poisoned Lock").select(None);
		}
	}
	fn reset(&mut self) {
		self.sel = SousGroupeSel::None;
		*self.table_state.write().expect("Poisoned Lock") = TableState::default();
	}
	fn fit_widths(&mut self) {
		let nom = fit_str_width(self.membres.iter().map(|m| m.nom.as_str()).chain(std::iter::once("Nom")));
		let prenom = fit_str_width(self.membres.iter().map(|m| m.prenom.as_str()).chain(std::iter::once("Prénom")));
		let naissance = self.membres.iter().map(|m| format!("{} ({} ans)", m.naissance.format("%Y-%m-%d"), m.age)).collect::<Vec<_>>();
		let naissance = fit_str_width(naissance.iter().map(|s| s.as_str()).chain(std::iter::once("Naissance")));
		let genre = fit_str_width(self.membres.iter().flat_map(|m| m.genre.map(|g| g.as_str())).chain(std::iter::once("Genre")));
		self.widths = [
			Constraint::Length(nom as u16 + 2),
			Constraint::Length(prenom as u16 + 2),
			Constraint::Length(naissance as u16 + 2),
			Constraint::Length(genre as u16 + 2),
			Constraint::Fill(1),
		];
	}
	fn build_sous_groupe(&self, groupe: &mut Groupe) {
		if let Some(sg) = &self.sg {
			let mut sous_groupe = SousGroupe {
				disc: sg.id,
				profil: sg.profil.read().expect("Poisoned Lock").get_interet().flatten().copied(),
				animateur: sg.animateur.read().expect("Poisoned Lock").get_str().flatten().map(|s| s.to_string()),
				..Default::default()
			};
			for membre in self.membres.iter() {
				sous_groupe.participants.insert(membre.id);
			}
			groupe.sous_groupe.push(sous_groupe);
		}
		for membre in self.membres.iter() {
			groupe.add_participant(membre.id);
		}
	}
}
impl WidgetRef for MembreView {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		let membres_area = {
			// if there is a sg, two lines of fields, else no lines of fields
			let field_lines = if self.sg.is_some() { 2 } else { 0 };
			if let Some(sg) = &self.sg {
				let selected = match self.sel {
					SousGroupeSel::Fields(idx) => Some(sg.ordre[idx].clone()),
					_ => None,
				};
				let fields_area = Rect {
					x: area.x,
					y: area.y,
					width: area.width,
					height: field_lines,
				};
				let (text, _scroll) = sg.cluster.get_text_and_scroll(selected, field_lines, area.width, 0);
				Paragraph::new(text)
					.render(fields_area, buf);
				Rect {
					x: area.x,
					y: area.y + field_lines,
					width: area.width,
					height: area.height - field_lines,
				}
			} else {
				area
			}
		};

		let table = Table::new(
			self.membres.iter().map(|m| m.to_row()), 
			&self.widths,)
			.header(PAGE_GROUPE_MEMBRE_TABLE_HEADERS.clone())
			.style(Style::new().gray())
			.row_highlight_style(Style::new().white().on_dark_gray());
		let mut lock = self.table_state.write().expect("Poisoned Lock");
		StatefulWidget::render(table, membres_area, buf, &mut lock);
	}
}
impl Screen for MembreView {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<AppState>) -> Result<crate::ui::actions::UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Up => {
						self.previous();
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Down => {
						self.next();
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Esc => Ok(UpdateAction::Pop.one()),
					cte::KeyCode::Enter => {
						match self.sel {
							SousGroupeSel::Fields(idx) => { // call the action of the field
								if let Some(sg) = &self.sg {
									Ok(Field::on_action(sg.ordre[idx].clone(), Some(self.dirty_flag.clone())))
								} else {
									Ok(UpdateAction::Continue.one())
								}
							},
							SousGroupeSel::Membres(idx) => {
								if let Some(membre) = self.membres.get(idx) {
									let mid = membre.id;
									let gid = self.gid;
									let sg = self.sg.as_ref().map(|sg| sg.id);
									let menu: Menu<'static, &'static str> = Menu::new(Box::new([
										MenuItem {id: "Voir le membre", action: Box::new(move |state| {
											Ok(vec![UpdateAction::Pop, UpdateAction::OpenMembre(mid)])
										})},
										MenuItem {id: "Changer de sous-groupe", action: Box::new(move |state| {
											let menu: Menu<'static, SousGroupeMenuItem> = {
												let groupes = state.groupes.read().expect("Poisoned Lock");
												let groupe = groupes.get(gid).expect("Groupe not found");
												let current_sg = sg;
												SousGroupeMenuItem::mk_menu(groupe, mid, current_sg)
											};
											Ok(vec![UpdateAction::Pop, UpdateAction::PushSub(Box::new(menu))])
										})},
									]));
									Ok(UpdateAction::PushSub(Box::new(menu)).one())
								} else {
									Ok(UpdateAction::Continue.one())
								}
							},
							SousGroupeSel::None => Ok(UpdateAction::Continue.one()),
						}
					},
					_ => Ok(UpdateAction::Continue.one()),
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PageGroupeView {
	General,
	Membre(Option<u32>)
}
impl std::fmt::Display for PageGroupeView {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::General => write!(f, "Général"),
			Self::Membre(None) => write!(f, "Sans sous-groupe"),
			Self::Membre(Some(id)) => write!(f, "Sous-groupe {}", id),
		}
	}
}
impl PartialOrd for PageGroupeView {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for PageGroupeView {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match (self, other) {
			(Self::General, Self::General) => std::cmp::Ordering::Equal,
			(Self::General, _) => std::cmp::Ordering::Less,
			(_, Self::General) => std::cmp::Ordering::Greater,
			(Self::Membre(None), Self::Membre(None)) => std::cmp::Ordering::Equal,
			(Self::Membre(None), Self::Membre(Some(_))) => std::cmp::Ordering::Greater,
			(Self::Membre(Some(_)), Self::Membre(None)) => std::cmp::Ordering::Less,
			(Self::Membre(Some(id1)), Self::Membre(Some(id2))) => id1.cmp(id2),
		}
	}
}

#[derive(Debug)]
pub struct PageGroupe {
	view: usize,
	general_view: GeneralView,
	membre_views: HashMap<Option<u32>, MembreView>,
	gid: GroupeID,
	title: Line<'static>,
	tabs: Vec<PageGroupeView>,
	dirty_flag: Arc<Mutex<bool>>,
}
impl PageGroupe {
	pub fn try_new(groupe: &Groupe, membres: &MembreReg, requested_sg: Option<u32>) -> Result<Self, UIError> {
		let dirty_flag = Arc::new(Mutex::new(false));
		let general_view = GeneralView::new(groupe, dirty_flag.clone());
		let membres_views = mk_sg(groupe, membres, dirty_flag.clone())?;
		let title = Line::from(format!(" {} ", groupe.short_desc())).white().bold();
		let tabs = mk_tabs(&membres_views);
		let view = if let Some(req) = requested_sg {
			tabs.iter().position(|tab| tab == &PageGroupeView::Membre(Some(req))).unwrap_or(0)
		} else { 0 };
		Ok(Self {
			view,
			general_view,
			membre_views: membres_views,
			gid: groupe.id,
			title,
			tabs,
			dirty_flag,
		})
	}
	pub fn update(&mut self, groupe: &Groupe, membres: &MembreReg) -> Result<(), UIError> {
		let old_tab = self.tabs.get(self.view).copied().expect("Invalid view index");
		self.general_view.update(groupe);
		self.membre_views = mk_sg(groupe, membres, self.dirty_flag.clone())?;
		self.title = Line::from(format!(" {} ", groupe.short_desc())).white().bold();
		self.gid = groupe.id;
		self.tabs = mk_tabs(&self.membre_views);
		// try to keep the same tab if possible
		if let Some(new_view) = self.tabs.iter().position(|tab| tab == &old_tab) {
			self.view = new_view;
		} else {
			self.view = 0;
		}
		Ok(())
	}
	pub fn reset(&mut self) {
		self.general_view.reset();
		for view in self.membre_views.values_mut() {
			view.reset();
		}
		self.view = 0;
	}
	pub fn build_group(&self) -> Groupe {
		let mut groupe = Groupe::new(self.gid);
		self.general_view.build_groupe(&mut groupe);
		for view in self.membre_views.values() {
			view.build_sous_groupe(&mut groupe);
		}
		groupe
	}

	fn get_current_view_mut(&mut self) -> Option<&mut dyn Screen> {
		match self.tabs.get(self.view) {
			Some(PageGroupeView::General) => Some(&mut self.general_view),
			Some(PageGroupeView::Membre(id)) => self.membre_views.get_mut(id).map(|v| v as &mut dyn Screen),
			None => None,
		}
	}
}
impl WidgetRef for PageGroupe {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		// render the border and title
		let block = VIEW_TABLE_BLOCK.clone().title(self.title.clone());
		let inner = block.inner(area);
		block.render(area, buf);
		// render the tabs and return the remaining area
		let view_area = {
			let tabs_raw = self.tabs.iter().map(PageGroupeView::to_string).collect::<Vec<_>>();
			let sel = tabs_raw.get(self.view).expect("Invalid view index").clone();
			let tabs = tabs_raw.iter().map(|t| stylize_selection(&sel, t)).collect::<Vec<_>>();
			let line = Line::from_iter(tabs.into_iter().intersperse(Span::from(" | ").white())).centered();
			let header_area = Rect {
				x: inner.x,
				y: inner.y,
				width: inner.width,
				height: 2,
			};
			Paragraph::new(line)
				.block(VIEW_TABLE_HEADER_BLOCK.clone())
				.render(header_area, buf);
			Rect {
				x: inner.x,
				y: inner.y + 2,
				width: inner.width,
				height: inner.height.saturating_sub(2),
			}
		};
		match self.tabs.get(self.view) {
			Some(PageGroupeView::General) => self.general_view.render_ref(view_area, buf),
			Some(PageGroupeView::Membre(id)) => {
				if let Some(view) = self.membre_views.get(id) {
					view.render_ref(view_area, buf);
				}
			}
			None => {}
		}
	}
}
impl Screen for PageGroupe {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<AppState>) -> Result<crate::ui::actions::UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Tab => {
						// change view
						let groupe = self.build_group();
						self.view = (self.view + 1) % self.tabs.len();
						Ok(vec![UpdateAction::UpdateGroupe(groupe), UpdateAction::Redraw])
					},
					cte::KeyCode::Esc => {
						// close the page
						let groupe = self.build_group();
						Ok(vec![UpdateAction::UpdateGroupe(groupe), UpdateAction::Pop])
					},
					_ => self.get_current_view_mut().map_or(Ok(UpdateAction::Continue.one()), |view| view.handle_event(event, state)),
				}
			},
			_ => self.get_current_view_mut().map_or(Ok(UpdateAction::Continue.one()), |view| view.handle_event(event, state)),
		}
	}
	fn on_refocus(&mut self, state: Arc<AppState>) {
		if *self.dirty_flag.lock().expect("Poisoned Lock") {
			let groupe = self.build_group();
			self.title = Line::from(format!(" {} ", groupe.short_desc())).white().bold();
			let mut groupes = state.groupes.write().expect("Poisoned Lock");
			*groupes.get_mut(self.gid).expect("Groupe not found") = groupe;
			*self.dirty_flag.lock().expect("Poisoned Lock") = false;
		} else {
			let groupes = state.groupes.read().expect("Poisoned Lock");
			let membres = state.membres.read().expect("Poisoned Lock");
			let groupe = groupes.get(self.gid).expect("Groupe not found");
			let _ = self.update(groupe, &membres);
		}
	}
}

fn mk_sg(groupe: &Groupe, membres: &MembreReg, dirty_flag: Arc<Mutex<bool>>) -> Result<HashMap<Option<u32>, MembreView>, UIError> {
	let mut membre_views = HashMap::new();
	let mut _membres = groupe.participants.clone();
	for sg in groupe.sous_groupe.iter() {
		let id = sg.disc;
		let animateur = sg.animateur.as_deref();
		let profil = sg.profil;
		let mut mbrs = Vec::new();
		for mid in sg.participants.iter() {
			if _membres.remove(mid) {
				let m = membres.get(*mid)?;
				mbrs.push(m);
			}
		}
		let sg_data = SousGroupeData::new(id, profil, animateur);
		let mut view = MembreView::new(groupe.id, Some(sg_data), &mbrs, dirty_flag.clone());
		view.fit_widths();
		membre_views.insert(Some(id), view);
	}
	{
		let mut mbrs = Vec::new();
		for mid in _membres.iter() {
			let m = membres.get(*mid)?;
			mbrs.push(m);
		}
		let mut view = MembreView::new(groupe.id, None, &mbrs, dirty_flag);
		view.fit_widths();
		membre_views.insert(None, view);
	}
	Ok(membre_views)
}

fn mk_tabs(sous_groupes: &HashMap<Option<u32>, MembreView>) -> Vec<PageGroupeView> {
	let mut tabs = [
		vec![PageGroupeView::General],
		sous_groupes.keys().map(|k| PageGroupeView::Membre(*k)).collect(),
	].concat();
	tabs.sort();
	tabs
}

#[derive(Debug, Clone)]
pub struct SousGroupeMenuItem {
	pub gid: GroupeID,
	pub mid: MembreID,
	pub sg: Option<u32>,
	pub label: String,
}
impl std::fmt::Display for SousGroupeMenuItem {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.label.fmt(f)
	}
}
impl SousGroupeMenuItem {
	pub fn new(gid: GroupeID, mid: MembreID, sg: Option<u32>) -> Self {
		let label = if let Some(sg) = sg {
			format!("Sous-groupe {sg}")
		} else {
			"Aucun".into()
		};
		Self {
			gid,
			mid,
			sg,
			label,
		}
	}
	pub fn with_label(mut self, label: String) -> Self {
		self.label = label;
		self
	}
	pub fn mk_action(&self) -> Box<Action> {
		let gid = self.gid;
		let sg = self.sg;
		let mid = self.mid;
		Box::new(move |state| {
			let mut groupes = state.groupes.write().expect("Poisoned Lock");
			let groupe = groupes.get_mut(gid).expect("Groupe not found");
			let _ = groupe.change_subgroup_for(mid, sg);

			Ok(UpdateAction::Pop.one())
		})
	}

	pub fn mk_menu(groupe: &Groupe, mid: MembreID, current_sg: Option<u32>) -> Menu<'static, SousGroupeMenuItem> {
		let mut items = vec![];
		if current_sg.is_some() {
			items.push(SousGroupeMenuItem::new(groupe.id, mid, None));
		}
		for sg in groupe.sous_groupe.iter() {
			if Some(sg.disc) != current_sg {
				items.push(SousGroupeMenuItem::new(groupe.id, mid, Some(sg.disc)));
			}
		}
		let max_sg = groupe.sous_groupe.iter().map(|sg| sg.disc).max().unwrap_or(0);
		let nouveau_mi = SousGroupeMenuItem::new(groupe.id, mid, Some(max_sg + 1)).with_label("Nouveau".into());
		items.push(nouveau_mi);
		let items = items.into_iter().map(|mi| {
			let action = mi.mk_action();
			MenuItem {
				id: mi,
				action,
			}
		}).collect();
		Menu::new(items)
	}
}