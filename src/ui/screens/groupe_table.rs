use std::{collections::HashSet, sync::{Arc, RwLock}};

use lazy_static::lazy_static;
use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::{Color, Style, Stylize}, symbols::border, text::Line, widgets::{Block, Clear, Row, StatefulWidget, Table, TableState, Widget, WidgetRef}};

use crate::{cdj::groupes::{Groupe, GroupeID, GroupeReg}, prelude::get_from_reg, ui::{Screen, UpdateAction, fit_str_width}};

lazy_static!{
	pub static ref GROUP_TABLE_HEADERS: Row<'static> = Row::new(vec![
		Line::from("Saison").white().bold(),
		Line::from("Activité").white().bold(),
		Line::from("Site").white().bold(),
		Line::from("Catégorie").white().bold(),
		Line::from("Semaine").white().bold(),
		Line::from("Discriminant").white().bold(),
		Line::from("Inscriptions").white().bold(),
		Line::from("Capacité").white().bold(),
		Line::default(), // extra column to fill space
	]);
	pub static ref GROUP_TABLE_TITLE: Line<'static> = Line::from(" Groupes ").white().bold();
	pub static ref GROUP_TABLE_BLOCK: Block<'static> = Block::bordered()
		.title_top(GROUP_TABLE_TITLE.clone())
		.border_style(Style::new().white())
		.border_set(border::THICK)
		.bg(Color::Black);
}

#[derive(Debug, Clone)]
struct GTData {
	id: GroupeID,
	saison: Arc<str>,
	activite: Arc<str>,
	site: Arc<str>,
	categorie: Arc<str>,
	semaine: Arc<str>,
	discriminant: Arc<str>,
	inscriptions: usize,
	capacite: usize,
}
impl From<&Groupe> for GTData {
	fn from(value: &Groupe) -> Self {
		Self {
			id: value.id,
			saison: Arc::from(value.saison.as_deref().unwrap_or("")),
			activite: Arc::from(value.activite.as_deref().unwrap_or("")),
			site: Arc::from(value.site.as_deref().unwrap_or("")),
			categorie: Arc::from(value.category.as_deref().unwrap_or("")),
			semaine: Arc::from(value.semaine.as_deref().unwrap_or("")),
			discriminant: Arc::from(value.discriminant.as_deref().unwrap_or("")),
			inscriptions: value.participants.len(),
			capacite: value.capacite.unwrap_or(0),
		}
	}
}
impl GTData {
	fn from_reg(g: &Groupe, strreg: &mut HashSet<Arc<str>>) -> Self {
		let saison = get_from_reg(strreg, g.saison.as_deref().unwrap_or(""));
		let activite = get_from_reg(strreg, g.activite.as_deref().unwrap_or(""));
		let site = get_from_reg(strreg, g.site.as_deref().unwrap_or(""));
		let categorie = get_from_reg(strreg, g.category.as_deref().unwrap_or(""));
		let semaine = get_from_reg(strreg, g.semaine.as_deref().unwrap_or(""));
		let discriminant = get_from_reg(strreg, g.discriminant.as_deref().unwrap_or(""));
		let inscriptions = g.participants.len();
		let capacite = g.capacite.unwrap_or(0);
		GTData {
			id: g.id,
			saison,
			activite,
			site,
			categorie,
			semaine,
			discriminant,
			inscriptions,
			capacite,
		}
	}
	fn key(&self) -> (&str, &str, &str, &str, &str, &str) {
		(
			self.saison.as_ref(),
			self.activite.as_ref(),
			self.site.as_ref(),
			self.categorie.as_ref(),
			self.semaine.as_ref(),
			self.discriminant.as_ref(),
		)
	}
	fn to_row<'a>(&'a self) -> Row<'a> {
		Row::new(vec![
			Line::from(self.saison.as_ref()),
			Line::from(self.activite.as_ref()),
			Line::from(self.site.as_ref()),
			Line::from(self.categorie.as_ref()),
			Line::from(self.semaine.as_ref()),
			Line::from(self.discriminant.as_ref()),
			Line::from(self.inscriptions.to_string()),
			Line::from(self.capacite.to_string()),
			Line::default(), // fill space
		])
	}
}
impl PartialEq for GTData {
	fn eq(&self, other: &Self) -> bool {
		self.key() == other.key()
	}
}
impl Eq for GTData {}
impl PartialOrd for GTData {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for GTData {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.key().cmp(&other.key())
	}
}

#[derive(Debug, Default)]
pub struct GroupeTable {
	data: Vec<GTData>,
	width: [Constraint; 9],
	state: RwLock<TableState>, // we actually need the state for smooth scrolling
}
impl GroupeTable {
	pub fn update(&mut self, groupes: &GroupeReg) {
		let current_sel = self.state.read().expect("Poisoned Lock").selected().map(|i| self.data.get(i).expect("Selection out of bounds").id);

		// 1. Gather the data and sort
		let mut strreg: HashSet<Arc<str>> = HashSet::new();
		self.data = groupes.groupes().filter_map(|g| {
			if g.is_null() {
				None
			} else {
				Some(GTData::from_reg(g, &mut strreg))
			}
		}).collect();
		self.data.sort();

		// restore selection
		self.state.write().expect("Poisoned Lock").select(current_sel.and_then(|sel| self.data.iter().position(|g| g.id == sel)));
	}
	pub fn update_groupe(&mut self, groupe: &Groupe) {
		let data = GTData::from(groupe);
		if let Some(i) = self.data.iter().position(|g| g.id == groupe.id) {
			self.data[i] = data;
		} else {
			let current_sel = self.state.read().expect("Poisoned Lock").selected().map(|i| self.data.get(i).expect("Selection out of bounds").id);
			self.data.push(data);
			self.data.sort();
			self.state.write().expect("Poisoned Lock").select(current_sel.and_then(|sel| self.data.iter().position(|g| g.id == sel)));
		}
	}

	pub fn with_widths(mut self, widths: [Constraint; 8]) -> Self {
		let mut it = widths.into_iter().chain(std::iter::once(Constraint::Fill(1)));
		self.width = std::array::from_fn(|_| it.next().unwrap());
		self
	}
	pub fn fit_widths(&mut self) {
		let saison_w = fit_str_width(self.data.iter().map(|d| d.saison.as_ref()).chain(std::iter::once("Saison")));
		let activite_w = fit_str_width(self.data.iter().map(|d| d.activite.as_ref()).chain(std::iter::once("Activité")));
		let site_w = fit_str_width(self.data.iter().map(|d| d.site.as_ref()).chain(std::iter::once("Site")));
		let categorie_w = fit_str_width(self.data.iter().map(|d| d.categorie.as_ref()).chain(std::iter::once("Catégorie")));
		let semaine_w = fit_str_width(self.data.iter().map(|d| d.semaine.as_ref()).chain(std::iter::once("Semaine")));
		let discriminant_w = fit_str_width(self.data.iter().map(|d| d.discriminant.as_ref()).chain(std::iter::once("Discriminant")));
		let inscriptions_w = {
			let inscriptions = self.data.iter().map(|d| d.inscriptions.to_string()).collect::<Vec<_>>();
			fit_str_width(inscriptions.iter().map(|s| s.as_str()).chain(std::iter::once("Inscriptions")))
		};
		let capacite_w = {
			let capacites = self.data.iter().map(|d| d.capacite.to_string()).collect::<Vec<_>>();
			fit_str_width(capacites.iter().map(|s| s.as_str()).chain(std::iter::once("Capacité")))
		};
		self.width = [
			Constraint::Length(saison_w as u16 + 2),
			Constraint::Length(activite_w as u16 + 2),
			Constraint::Length(site_w as u16 + 2),
			Constraint::Length(categorie_w as u16 + 2),
			Constraint::Length(semaine_w as u16 + 2),
			Constraint::Length(discriminant_w as u16 + 2),
			Constraint::Length(inscriptions_w as u16 + 2),
			Constraint::Length(capacite_w as u16 + 2),
			Constraint::Fill(1),
		];
	}

}
impl WidgetRef for GroupeTable {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		// build the table widget
		let rows = self.data.iter().map(|d| d.to_row());
		let table = Table::new(rows, &self.width)
			.header(GROUP_TABLE_HEADERS.clone())
			.row_highlight_style(Style::new().white().on_dark_gray());

		Clear.render(area, buf);
		// render a black block
		Block::default().bg(Color::Black).render(area, buf);
		let mut lock = self.state.write().expect("Poisoned Lock");
		StatefulWidget::render(table, area, buf, &mut lock);
	}
}
impl Screen for GroupeTable {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<crate::ui::AppState>) -> Result<crate::ui::actions::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Up => {
						// decr selection
						let sel = self.state.read().expect("Poisoned Lock").selected();
						let sel = Some(sel.map(|s| s.saturating_sub(1)).unwrap_or(0).min(self.data.len()));
						let sel = if self.data.is_empty() { None } else { sel };
						self.state.write().expect("Poisoned Lock").select(sel);
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Down => {
						// incr selection
						let sel = self.state.read().expect("Poisoned Lock").selected();
						let sel = Some(sel.map(|s| s.saturating_add(1)).unwrap_or(0).min(self.data.len()));
						let sel = if self.data.is_empty() { None } else { sel };
						self.state.write().expect("Poisoned Lock").select(sel);
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Enter => {
						// todo! select the group and show detailed view
						Ok(UpdateAction::Continue.one())
					},
					cte::KeyCode::Esc => Ok(UpdateAction::Pop.one()),
					_ => Ok(UpdateAction::Continue.one()),
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}
