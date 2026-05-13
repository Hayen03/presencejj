use std::{cell::Cell, collections::HashSet, sync::{Arc, Mutex, RwLock}};

use lazy_static::lazy_static;
use ratatui::{
	buffer::Buffer,
	layout::{Constraint, Rect},
	style::{Color, Style, Stylize},
	text::Line,
	widgets::{Clear, Paragraph, Row, StatefulWidget, Table, TableState, Widget, WidgetRef, Wrap},
};

use crate::{
	cdj::{
		comptes::{Compte, CompteID},
		membres::{Membre, MembreID, MembreReg},
	}, data::Genre, prelude::AsStr, ui::{
		AppState, Screen, UIError, UpdateAction, fit_str_width, screens::{Field, FieldBlock, FieldBlockCluster, FieldType, VIEW_TABLE_BLOCK, VIEW_TABLE_HEADER_BLOCK, stylize_selection}
	}
};

lazy_static! {
	pub static ref PAGE_COMPTE_MEMBRE_TABLE_HEADER: Row<'static> = Row::new(vec![
		Line::from("Nom").white().bold(),
		Line::from("Prénom").white().bold(),
		Line::from("Naissance (âge)").white().bold(),
		Line::from("Genre").white().bold(),
		Line::default(),
	]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageCompteView {
	General,
	Membres,
}
impl<'a> AsStr<'static, 'a> for PageCompteView {
	fn as_str(&'a self) -> &'static str {
		match self {
			PageCompteView::General => "General",
			PageCompteView::Membres => "Membres",
		}
	}
}
impl PageCompteView {
	fn next(self) -> Self {
		match self {
			PageCompteView::General => PageCompteView::Membres,
			PageCompteView::Membres => PageCompteView::General,
		}
	}
}

#[derive(Debug)]
struct ViewGeneral {
	tel: Arc<RwLock<Field>>,
	email: Arc<RwLock<Field>>,
	adresse: Arc<RwLock<Field>>,
	ordre: Vec<Arc<RwLock<Field>>>,
	cluster: FieldBlockCluster,
	sel: Option<usize>,
	scroll: Cell<u16>,
	dirty_flag: Arc<Mutex<bool>>,
}
impl ViewGeneral {
	fn new(compte: &Compte, dirty_flag: Arc<Mutex<bool>>) -> Self {
		let mut block = FieldBlock::default();
		let tel = block.add_field(Field::from(compte.tel).with_label("Telephone".into()));
		let email = block.add_field(Field::from(compte.email.clone()).with_label("Email".into()));
		let adresse = block.add_field(Field::from(compte.adresse.clone()).with_label("Adresse".into()));
		let ordre = vec![tel.clone(), email.clone(), adresse.clone()];
		Self {
			tel,
			email,
			adresse,
			ordre,
			cluster: FieldBlockCluster::new(vec![block]),
			sel: None,
			scroll: Cell::new(0),
			dirty_flag,
		}
	}
	fn update(&mut self, compte: &Compte) {
		self.tel.write().expect("Poisoned Lock").set_value(FieldType::from(compte.tel));
		self.email.write().expect("Poisoned Lock").set_value(FieldType::from(compte.email.clone()));
		self.adresse.write().expect("Poisoned Lock").set_value(FieldType::from(compte.adresse.clone()));
	}
	fn build_compte(&self, compte: &mut Compte) {
		compte.tel = self.tel.read().expect("Poisoned Lock").get_tel().flatten().cloned();
		compte.email = self.email.read().expect("Poisoned Lock").get_email().flatten().cloned();
		compte.adresse = self.adresse.read().expect("Poisoned Lock").get_adresse().cloned();
	}
	fn reset(&mut self) {
		self.scroll.set(0);
		self.sel = None;
	}
}
impl WidgetRef for ViewGeneral {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		let selected = self.sel.map(|s| self.ordre[s].clone());
		let (text, scroll) = self.cluster.get_text_and_scroll(selected, area.height, area.width, self.scroll.get());
		Paragraph::new(text)
			.wrap(Wrap { trim: false })
			.scroll((scroll, 0))
			.render(area, buf);
		self.scroll.set(scroll);
	}
}
impl Screen for ViewGeneral {
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

#[derive(Debug, Clone)]
struct MembreLine {
	id: MembreID,
	nom: String,
	prenom: String,
	naissance_age: String,
	genre: Option<Genre>,
}
impl MembreLine {
	fn from_membre(membre: &Membre) -> Self {
		Self {
			id: membre.id,
			nom: membre.nom.clone(),
			prenom: membre.prenom.clone(),
			naissance_age: format!("{} ({} ans)", membre.naissance.format("%Y-%m-%d"), membre.age()),
			genre: membre.genre,
		}
	}
	fn key(&self) -> (&str, &str, &str) {
		(&self.nom, &self.prenom, &self.naissance_age)
	}
	fn to_row(&self) -> Row<'_> {
		Row::new(vec![
			Line::from(self.nom.as_str()),
			Line::from(self.prenom.as_str()),
			Line::from(self.naissance_age.as_str()),
			Line::from(self.genre.map_or("".to_string(), |g| g.to_string())),
			Line::default(),
		])
	}
}
impl PartialEq for MembreLine {
	fn eq(&self, other: &Self) -> bool {
		self.key() == other.key()
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
		self.key().cmp(&other.key())
	}
}

#[derive(Debug)]
struct ViewMembres {
	membres: Vec<MembreLine>,
	table_state: RwLock<TableState>,
	widths: [Constraint; 5],
}
impl ViewMembres {
	fn new(compte: &Compte, membres: &MembreReg) -> Self {
		let mut this = Self {
			membres: Vec::new(),
			table_state: RwLock::new(TableState::default()),
			widths: [Constraint::default(); 5],
		};
		this.update(compte, membres);
		this
	}
	fn update(&mut self, compte: &Compte, membres: &MembreReg) {
		let current_sel = self.table_state.read().expect("Poisoned Lock").selected().and_then(|i| self.membres.get(i).map(|m| m.id));
		let compte_membres = compte.membres.iter().copied().collect::<HashSet<_>>();
		self.membres = membres.membres()
			.filter(|m| compte_membres.contains(&m.id) || m.compte == Some(compte.id))
			.map(MembreLine::from_membre)
			.collect();
		self.membres.sort();
		self.table_state.write().expect("Poisoned Lock").select(current_sel.and_then(|mid| self.membres.iter().position(|m| m.id == mid)));
		self.fit_widths();
	}
	fn fit_widths(&mut self) {
		let nom = fit_str_width(self.membres.iter().map(|m| m.nom.as_str()).chain(std::iter::once("Nom")));
		let prenom = fit_str_width(self.membres.iter().map(|m| m.prenom.as_str()).chain(std::iter::once("Prénom")));
		let naissance = fit_str_width(self.membres.iter().map(|m| m.naissance_age.as_str()).chain(std::iter::once("Naissance (âge)")));
		let genre = fit_str_width(self.membres.iter().flat_map(|m| m.genre.map(|g| g.as_str())).chain(std::iter::once("Genre")));
		self.widths = [
			Constraint::Length(nom as u16 + 2),
			Constraint::Length(prenom as u16 + 2),
			Constraint::Length(naissance as u16 + 2),
			Constraint::Length(genre as u16 + 2),
			Constraint::Fill(1),
		];
	}
	fn reset(&mut self) {
		self.table_state = RwLock::new(TableState::default());
	}
}
impl WidgetRef for ViewMembres {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		let table = Table::new(self.membres.iter().map(MembreLine::to_row), &self.widths)
			.header(PAGE_COMPTE_MEMBRE_TABLE_HEADER.clone())
			.row_highlight_style(Style::new().white().on_dark_gray())
			.style(Style::new().gray());
		let mut lock = self.table_state.write().expect("Poisoned Lock");
		StatefulWidget::render(table, area, buf, &mut lock);
	}
}
impl Screen for ViewMembres {
	fn handle_event(&mut self, event: crate::ui::event::Event, _state: Arc<AppState>) -> Result<crate::ui::actions::UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Up => {
						let sel = self.table_state.read().expect("Poisoned Lock").selected().map(|s| s.saturating_sub(1)).unwrap_or(0);
						self.table_state.write().expect("Poisoned Lock").select(if self.membres.is_empty() { None } else { Some(sel) });
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Down => {
						let sel = self.table_state.read().expect("Poisoned Lock").selected().map(|s| s.saturating_add(1).min(self.membres.len() - 1)).unwrap_or(0);
						self.table_state.write().expect("Poisoned Lock").select(if self.membres.is_empty() { None } else { Some(sel) });
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Enter => {
						if let Some(sel) = self.table_state.read().expect("Poisoned Lock").selected() {
							Ok(UpdateAction::OpenMembre(self.membres[sel].id).one())
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
pub struct PageCompte {
	cid: CompteID,
	sel_view: PageCompteView,
	view_general: ViewGeneral,
	view_membres: ViewMembres,
	title: Line<'static>,
	dirty_flag: Arc<Mutex<bool>>,
}
impl PageCompte {
	pub fn try_new(compte: &Compte, membres: &MembreReg) -> Result<Self, UIError> {
		let dirty_flag = Arc::new(Mutex::new(false));
		Ok(Self {
			cid: compte.id,
			sel_view: PageCompteView::General,
			view_general: ViewGeneral::new(compte, dirty_flag.clone()),
			view_membres: ViewMembres::new(compte, membres),
			title: Line::from(format!(" {} ", compte.mandataire)).white().bold(),
			dirty_flag,
		})
	}
	fn build_compte(&self, compte: &mut Compte) {
		self.view_general.build_compte(compte);
	}
}
impl WidgetRef for PageCompte {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		Clear.render(area, buf);
		let block = VIEW_TABLE_BLOCK.clone()
			.title_top(self.title.clone())
			.bg(Color::Black);
		let inner = block.inner(area);
		block.render(area, buf);

		let header = Line::from(vec![
			stylize_selection(&self.sel_view, &PageCompteView::General),
			" | ".gray(),
			stylize_selection(&self.sel_view, &PageCompteView::Membres),
		]);
		let header_area = Rect {
			x: inner.x,
			y: inner.y,
			width: inner.width,
			height: 2,
		};
		Paragraph::new(header)
			.block(VIEW_TABLE_HEADER_BLOCK.clone())
			.centered()
			.render(header_area, buf);

		let view_area = Rect {
			x: inner.x,
			y: inner.y + 2,
			width: inner.width,
			height: inner.height.saturating_sub(2),
		};
		match self.sel_view {
			PageCompteView::General => self.view_general.render_ref(view_area, buf),
			PageCompteView::Membres => self.view_membres.render_ref(view_area, buf),
		}
	}
}
impl Screen for PageCompte {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<AppState>) -> Result<crate::ui::actions::UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Tab => {
						{
							let mut comptes = state.comptes.write().expect("Poisoned Lock");
							let compte = comptes.get_mut(self.cid)?;
							self.build_compte(compte);
						}
						self.sel_view = self.sel_view.next();
						match self.sel_view {
							PageCompteView::General => self.view_general.reset(),
							PageCompteView::Membres => self.view_membres.reset(),
						}
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Esc => {
						{
							let mut comptes = state.comptes.write().expect("Poisoned Lock");
							let compte = comptes.get_mut(self.cid)?;
							self.build_compte(compte);
						}
						Ok(UpdateAction::Pop.one())
					},
					_ => match self.sel_view {
						PageCompteView::General => self.view_general.handle_event(event, state),
						PageCompteView::Membres => self.view_membres.handle_event(event, state),
					},
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}

	fn on_refocus(&mut self, state: Arc<AppState>) {
		if *self.dirty_flag.lock().expect("Poisoned Lock") {
			let mut comptes = state.comptes.write().expect("Poisoned Lock");
			let compte = comptes.get_mut(self.cid).expect("Compte Inexistant");
			self.build_compte(compte); // rebuild to commit the changes to the state
			*self.dirty_flag.lock().expect("Poisoned Lock") = false;
		} else {
			let comptes = state.comptes.read().expect("Poisoned Lock");
			let membres = state.membres.read().expect("Poisoned Lock");
			let compte = comptes.get(self.cid).expect("Compte Inexistant");
			self.title = Line::from(format!(" {} ", compte.mandataire)).white().bold();
			self.view_general.update(compte);
			self.view_membres.update(compte, &membres);
		}
		
	}
}
