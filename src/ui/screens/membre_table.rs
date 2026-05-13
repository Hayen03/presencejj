use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::{Color, Style, Stylize}, symbols::border, text::Line, widgets::{Block, Clear, Row, StatefulWidget, Table, TableState, Widget, WidgetRef}};

use crate::{cdj::{comptes::NULL_COMPTE, membres::{Membre, MembreID, MembreReg, NULL_MEMBRE}}, ui::{Screen, UpdateAction, fit_str_width, screens::{InfoScreen, Menu, MenuItem}}};

lazy_static!{
	pub static ref MEMBRE_TABLE_HEADERS: Row<'static> = Row::new(vec![
		Line::from("Nom").white().bold(),
		Line::from("Prenom").white().bold(),
		Line::from("Naissance").white().bold(),
		Line::from("Genre").white().bold(),
		Line::default(), // extra column to fill space
	]);
	pub static ref MEMBRE_TABLE_TITLE: Line<'static> = Line::from(" Membres ").white().bold();
	pub static ref MEMBRE_TABLE_BLOCK: Block<'static> = Block::bordered()
		.title_top(MEMBRE_TABLE_TITLE.clone())
		.border_style(Style::new().white())
		.border_set(border::THICK)
		.bg(Color::Black);
}

#[derive(Debug, Clone)]
struct MTData {
	id: MembreID,
	nom: Arc<str>,
	prenom: Arc<str>,
	naissance: Arc<str>,
	genre: &'static str,
}
impl From<&Membre> for MTData {
	fn from(m: &Membre) -> Self {
		let nom = Arc::from(m.nom.as_str());
		let prenom = Arc::from(m.prenom.as_str());
		let naissance = Arc::from(m.naissance.format("%Y-%m-%d").to_string().as_str());
		MTData {
			id: m.id,
			nom,
			prenom,
			naissance,
			genre: m.genre.map(|g| g.as_str()).unwrap_or(""),
		}
	}
}
impl MTData {
	fn key(&self) -> (&str, &str, &str) {
		(
			self.nom.as_ref(),
			self.prenom.as_ref(),
			self.naissance.as_ref(),
		)
	}
	fn to_row<'a>(&'a self) -> Row<'a> {
		Row::new(vec![
			Line::from(self.nom.as_ref()),
			Line::from(self.prenom.as_ref()),
			Line::from(self.naissance.as_ref()),
			Line::from(self.genre),
			Line::default(), // fill space
		])
	}
}
impl PartialEq for MTData {
	fn eq(&self, other: &Self) -> bool {
		self.key() == other.key()
	}
}
impl Eq for MTData {}
impl PartialOrd for MTData {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for MTData {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.key().cmp(&other.key())
	}
}

#[derive(Debug, Default)]
pub struct MembreTable {
	data: Vec<MTData>,
	width: [Constraint; 5],
	state: RwLock<TableState>, // we actually need the state for smooth scrolling
}
impl MembreTable {
	pub fn update(&mut self, membres: &MembreReg) {
		let current_sel = self.state.read().expect("Poisoned Lock").selected().map(|i| self.data.get(i).expect("Selection out of bounds").id);

		// 1. Gather the data and sort
		self.data = membres.membres().filter_map(|m| {
			if m.id == NULL_MEMBRE.id {
				None
			} else {
				Some(MTData::from(m))
			}
		}).collect();
		self.data.sort();

		// restore selection
		self.state.write().expect("Poisoned Lock").select(current_sel.and_then(|sel| self.data.iter().position(|g| g.id == sel)));
	}
	pub fn update_membre(&mut self, membre: &Membre) {
		let data = MTData::from(membre);
		if let Some(i) = self.data.iter().position(|g| g.id == membre.id) {
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
		let nom_w = fit_str_width(self.data.iter().map(|d| d.nom.as_ref()).chain(std::iter::once("Nom")));
		let prenom_w = fit_str_width(self.data.iter().map(|d| d.prenom.as_ref()).chain(std::iter::once("Prénom")));
		let naissance_w = fit_str_width(self.data.iter().map(|d| d.naissance.as_ref()).chain(std::iter::once("Naissance")));
		let genre_w = fit_str_width(self.data.iter().map(|d| d.genre).chain(std::iter::once("Genre")));
		self.width = [
			Constraint::Length(nom_w as u16 + 2),
			Constraint::Length(prenom_w as u16 + 2),
			Constraint::Length(naissance_w as u16 + 2),
			Constraint::Length(genre_w as u16 + 2),
			Constraint::Fill(1),
		];
	}
}
impl WidgetRef for MembreTable {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		// build the table widget
		let rows = self.data.iter().map(|g| g.to_row());
		let table = Table::new(rows, &self.width)
			.header(MEMBRE_TABLE_HEADERS.clone())
			.row_highlight_style(Style::new().white().on_dark_gray());

		Clear.render(area, buf);
		// render a black block
		Block::default().bg(Color::Black).render(area, buf);
		let mut lock = self.state.write().expect("Poisoned Lock");
		StatefulWidget::render(table, area, buf, &mut lock);
	}
}
impl Screen for MembreTable {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: Arc<crate::ui::AppState>) -> Result<crate::ui::actions::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Up => {
						// decr selection
						let sel = self.state.read().expect("Poisoned Lock").selected();
						let sel = Some(sel.map(|s| s.saturating_sub(1)).unwrap_or(0).min(self.data.len()-1));
						let sel = if self.data.is_empty() { None } else { sel };
						self.state.write().expect("Poisoned Lock").select(sel);
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Down => {
						// incr selection
						let sel = self.state.read().expect("Poisoned Lock").selected();
						let sel = Some(sel.map(|s| s.saturating_add(1)).unwrap_or(0).min(self.data.len()-1));
						let sel = if self.data.is_empty() { None } else { sel };
						self.state.write().expect("Poisoned Lock").select(sel);
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Enter => {
						if let Some(sel) = self.state.read().expect("Poisoned Lock").selected() {
							let sel = self.data.get(sel).expect("Selection out of bounds");
							let mid1 = sel.id;
							let mid2 = sel.id;
							let menu = Menu::new(Box::new([
								MenuItem {id: "Ouvrir", action: Box::new(move |state| Ok(vec![UpdateAction::Pop, UpdateAction::OpenMembre(mid1)]))},
								MenuItem {id: "Imprimer Fiche Santé", action: Box::new(move |state| {
									let out_dir = state.get_out_dir("Sélectionnez le dossier de sortie");
									if let Some(out_dir) = out_dir {
										let membres = state.membres.read().expect("Poisoned Lock");
										let membre = membres.get(mid2)?;
										let comptes = state.comptes.read().expect("Poisoned Lock");
										let compte = membre.compte.and_then(|cid| comptes.get(cid).ok()).unwrap_or(&NULL_COMPTE);
										let groupes_reg = state.groupes.read().expect("Poisoned Lock");
										let groupes = groupes_reg.groupes().filter(|g| g.participants.contains(&mid2)).collect::<Vec<_>>();
										let sites = groupes.iter().map(|g| (g.saison.as_deref().unwrap_or("None"), g.site.as_deref().unwrap_or("None"))).collect::<Vec<_>>();
										let logger = |msg| {}; // just void it for this task, we don't care about the logs
										let _res = crate::print::typst::print_fiche_med(membre, compte, &state.config.read().expect("Poisoned Lock"), &sites, true, out_dir.to_str(), &logger);
										if let Err(err) = _res {
											return Ok(vec![UpdateAction::Pop, UpdateAction::ErrorPopUp(Box::new(err))]);
										} else {
											return Ok(vec![UpdateAction::Pop, UpdateAction::PushSub(Box::new(InfoScreen::new("Succès".into(), "Fiche santé imprimée avec succès".into())))]);
										}
									}
									Ok(UpdateAction::Pop.one())
								})},
							]));
							Ok(UpdateAction::PushSub(Box::new(menu)).one())
						} else {
							Ok(UpdateAction::Continue.one())
						}
					},
					cte::KeyCode::Esc => Ok(UpdateAction::Pop.one()),
					_ => Ok(UpdateAction::Continue.one()),
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}