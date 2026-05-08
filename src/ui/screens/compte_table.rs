use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::{Color, Style, Stylize}, symbols::border, text::Line, widgets::{Block, Clear, Row, StatefulWidget, Table, TableState, Widget, WidgetRef}};

use crate::{cdj::comptes::{Compte, CompteID, CompteReg, NULL_COMPTE}, ui::{Screen, UpdateAction, fit_str_width}};

lazy_static!{
	pub static ref COMPTE_TABLE_HEADERS: Row<'static> = Row::new(vec![
		Line::from("Mandataire").white().bold(),
		Line::from("Tel").white().bold(),
		Line::from("Email").white().bold(),
		Line::from("Adresse").white().bold(),
		Line::default(), // extra column to fill space
	]);
	pub static ref COMPTE_TABLE_TITLE: Line<'static> = Line::from(" Comptes ").white().bold();
	pub static ref COMPTE_TABLE_BLOCK: Block<'static> = Block::bordered()
		.title_top(COMPTE_TABLE_TITLE.clone())
		.border_style(Style::new().white())
		.border_set(border::THICK)
		.bg(Color::Black);
}

#[derive(Debug, Clone)]
struct CTData {
	id: CompteID,
	mandataire: Arc<str>,
	tel: Arc<str>,
	email: Arc<str>,
	adresse: Arc<str>,
}
impl From<&Compte> for CTData {
	fn from(c: &Compte) -> Self {
		let mandataire = Arc::from(c.mandataire.as_str());
		let tel = Arc::from(c.tel.as_ref().map(|t| t.as_str()).unwrap_or(""));
		let email = Arc::from(c.email.as_ref().map(|e| e.as_str()).unwrap_or(""));
		let adr = c.adresse.as_ref().map(|a| a.full());
		let adresse = Arc::from(adr.as_deref().unwrap_or(""));
		CTData {
			id: c.id,
			mandataire,
			tel,
			email,
			adresse,
		}
	}
}
impl CTData {
	fn key(&self) -> (&str, &str, &str, &str) {
		(
			self.mandataire.as_ref(),
			self.tel.as_ref(),
			self.email.as_ref(),
			self.adresse.as_ref(),
		)
	}
	fn to_row<'a>(&'a self) -> Row<'a> {
		Row::new(vec![
			Line::from(self.mandataire.as_ref()),
			Line::from(self.tel.as_ref()),
			Line::from(self.email.as_ref()),
			Line::from(self.adresse.as_ref()),
			Line::default(), // fill space
		])
	}
}
impl PartialEq for CTData {
	fn eq(&self, other: &Self) -> bool {
		self.key() == other.key()
	}
}
impl Eq for CTData {}
impl PartialOrd for CTData {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for CTData {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.key().cmp(&other.key())
	}
}

#[derive(Debug, Default)]
pub struct CompteTable {
	data: Vec<CTData>,
	width: [Constraint; 5],
	state: RwLock<TableState>, // we actually need the state for smooth scrolling
}
impl CompteTable {
	pub fn update(&mut self, comptes: &CompteReg) {
		let current_sel = self.state.read().expect("Poisoned Lock").selected().map(|i| self.data.get(i).expect("Selection out of bounds").id);

		// 1. Gather the data and sort
		self.data = comptes.comptes().filter_map(|c| {
			if c.id == NULL_COMPTE.id {
				None
			} else {
				Some(CTData::from(c))
			}
		}).collect();
		self.data.sort();

		// restore selection
		self.state.write().expect("Poisoned Lock").select(current_sel.and_then(|sel| self.data.iter().position(|g| g.id == sel)));
	}
	pub fn update_compte(&mut self, compte: &Compte) {
		let data = CTData::from(compte);
		if let Some(i) = self.data.iter().position(|g| g.id == compte.id) {
			self.data[i] = data;
		} else {
			let current_sel = self.state.read().expect("Poisoned Lock").selected().map(|i| self.data.get(i).expect("Selection out of bounds").id);
			self.data.push(data);
			self.data.sort();
			self.state.write().expect("Poisoned Lock").select(current_sel.and_then(|sel| self.data.iter().position(|g| g.id == sel)));
		}
	}

	pub fn with_widths(mut self, widths: [Constraint; 4]) -> Self {
		let mut it = widths.into_iter().chain(std::iter::once(Constraint::Fill(1)));
		self.width = std::array::from_fn(|_| it.next().unwrap());
		self
	}
	pub fn fit_widths(&mut self) {
		let mandataire_w = fit_str_width(self.data.iter().map(|d| d.mandataire.as_ref()).chain(std::iter::once("Mandataire")));
		let tel_w = fit_str_width(self.data.iter().map(|d| d.tel.as_ref()).chain(std::iter::once("Téléphone")));
		let email_w = fit_str_width(self.data.iter().map(|d| d.email.as_ref()).chain(std::iter::once("Email")));
		let adresse_w = fit_str_width(self.data.iter().map(|d| d.adresse.as_ref()).chain(std::iter::once("Adresse")));
		self.width = [
			Constraint::Length(mandataire_w as u16 + 2),
			Constraint::Length(tel_w as u16 + 2),
			Constraint::Length(email_w as u16 + 2),
			Constraint::Length(adresse_w as u16 + 2),
			Constraint::Fill(1),
		];
	}
}
impl WidgetRef for CompteTable {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		// build the table widget
		let rows = self.data.iter().map(|g| g.to_row());
		let table = Table::new(rows, &self.width)
			.header(COMPTE_TABLE_HEADERS.clone())
			.row_highlight_style(Style::new().yellow().on_gray());

		Clear.render(area, buf);
		// render a black block
		Block::default().bg(Color::Black).render(area, buf);
		let mut lock = self.state.write().expect("Poisoned Lock");
		StatefulWidget::render(table, area, buf, &mut lock);
	}
}
impl Screen for CompteTable {
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
						Ok(UpdateAction::Continue.one())
					},
					cte::KeyCode::Down => {
						// incr selection
						let sel = self.state.read().expect("Poisoned Lock").selected();
						let sel = Some(sel.map(|s| s.saturating_add(1)).unwrap_or(0).min(self.data.len()));
						let sel = if self.data.is_empty() { None } else { sel };
						self.state.write().expect("Poisoned Lock").select(sel);
						Ok(UpdateAction::Continue.one())
					},
					cte::KeyCode::Enter => {
						// todo! select the compte and show detailed view
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