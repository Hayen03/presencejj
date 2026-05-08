use std::cell::Cell;

use lazy_static::lazy_static;
use ratatui::{style::{Color, Style, Stylize}, text::{Line, Text}, widgets::{Block, Clear, Paragraph, Widget, WidgetRef, Wrap}};
use ratatui::symbols::border;

use crate::{cdj::{comptes::{Compte, CompteReg}, groupes::{GroupeID, GroupeReg}, membres::{Contact, Membre}}, data::{BoolJustifie, adresse::Adresse, email::Email}, ui::{Screen, UIError, UpdateAction, screens::PageError}};
use crate::data::Genre;

lazy_static!{
	pub static ref PAGE_MEMBRE_TITLE: Line<'static> = Line::from(" Fiche Membre ").white().bold().centered();
	pub static ref PAGE_MEMBRE_BLOCK: Block<'static> = Block::bordered()
		.title_top(PAGE_MEMBRE_TITLE.clone())
		.border_style(Style::new().white())
		.border_set(border::THICK)
		.bg(Color::Black);
}

#[derive(Debug)]
struct GroupeLine {
	#[allow(dead_code)]
	id: GroupeID,
	desc: String,
}

#[derive(Debug)]
pub struct PageMembre {
	data: Membre,
	compte: Option<Compte>,
	#[allow(dead_code)]
	groupes: Vec<GroupeLine>,
	text: Text<'static>,
	scroll: Cell<usize>,
}
impl PageMembre {
	pub fn try_new(membre: Membre, comptes: &CompteReg, groupes: &GroupeReg) -> Result<Self, UIError> {
		let compte = if let Some(cid) = membre.compte {
			match comptes.get(cid) {
				Ok(c) => Some(c.clone()),
				Err(_) => return Err(UIError::Others { src: Box::new(PageError::MissingData { msg: "Le compte du membre n'existe pas".into() }) }),
			}
		} else {
			None
		};
		let mut grps: Vec<GroupeLine> = groupes.groupes().filter_map(|g| {
			g.desc_for(membre.id).map(|d| GroupeLine { id: g.id, desc: d })
		}).collect();
		grps.sort_by(|a, b| a.desc.cmp(&b.desc));
		let text = membre_to_text(&membre, compte.as_ref(), &grps);
		Ok(Self {
			data: membre,
			compte,
			groupes: grps,
			text,
			scroll: Cell::new(0),
		})
	}
}
impl WidgetRef for PageMembre {
	fn render_ref(&self,area: ratatui::prelude::Rect,buf: &mut ratatui::prelude::Buffer) {
		let block = PAGE_MEMBRE_BLOCK.clone();
		let inner = block.inner(area);

		let par = Paragraph::new(self.text.clone())
			.wrap(Wrap { trim: false });
		let h = par.line_count(inner.width);
		let max_scroll = h.saturating_sub(inner.height as usize);
		let scroll = self.scroll.get().min(max_scroll);
		self.scroll.set(scroll);
		
		Clear.render(area, buf);
		block.render(area, buf);
		par.scroll((scroll as u16, 0)).render(inner, buf);
	}
}
impl Screen for PageMembre {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: std::sync::Arc<crate::ui::AppState>) -> Result<crate::ui::actions::UpdateActions, UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Up => {
						self.scroll.set(self.scroll.get().saturating_sub(1));
						Ok(UpdateAction::Continue.one())
					},
					cte::KeyCode::Down => {
						self.scroll.set(self.scroll.get().saturating_add(1));
						Ok(UpdateAction::Continue.one())
					},
					cte::KeyCode::Esc => {
						Ok(vec![
							Some(UpdateAction::UpdateMembre(self.data.clone())),
							self.compte.as_ref().map(|c| UpdateAction::UpdateCompte(c.clone())),
							Some(UpdateAction::Pop),
						].into_iter().flatten().collect())
					},
					_ => Ok(UpdateAction::Continue.one()),
				}
			},
			_ => {
				Ok(UpdateAction::Continue.one())
			},
		}
	}
}

fn membre_to_text(m: &Membre, compte: Option<&Compte>, grps: &[GroupeLine]) -> Text<'static> {
	let mut lines = vec![
		get_name_line(m),
		Line::default(), // empty line
		get_naissance_line(m),
		get_genre_line(m),
		get_acc_line(m),
		get_auth_photo_line(m),
		get_comment_line(m),
		Line::default(),
	];
	lines.extend(get_compte_block(compte).into_iter().chain(std::iter::once(Line::default())));
	lines.extend(get_block_fiche_sante(m).into_iter().chain(std::iter::once(Line::default())));
	lines.extend(get_block_contacts(m).into_iter().chain(std::iter::once(Line::default())));
	lines.extend(get_block_depart(m).into_iter().chain(std::iter::once(Line::default())));
	lines.extend(get_block_piscine(m).into_iter().chain(std::iter::once(Line::default())));
	lines.extend(get_block_interets(m).into_iter().chain(std::iter::once(Line::default())));
	lines.extend(get_block_inscriptions(grps).into_iter().chain(std::iter::once(Line::default())));
	Text::from(lines)
}
fn get_name_line(m: &Membre) -> Line<'static> {
	Line::from(format!("{}, {}", m.nom, m.prenom)).blue().bold().centered().underlined()
}
fn get_naissance_line(m: &Membre) -> Line<'static> {
	Line::from(vec![
		"Naissance: ".white().bold(),
		m.naissance.format("%Y-%m-%d").to_string().gray(),
		" (".white().bold(),
		m.age().to_string().gray(),
		" ans)".white().bold(),
	])
}
fn get_genre_line(m: &Membre) -> Line<'static> {
	Line::from(vec![
		"Genre: ".white().bold(),
		m.genre.map(Genre::as_str).unwrap_or("").gray(),
	])
}
fn get_acc_line(m: &Membre) -> Line<'static> {
	Line::from(vec![
		"Accompagnement: ".white().bold(),
		match m.accompagnement {
			Some(true) => "Oui".green(),
			_ => "Non".gray(),
		}
	])
}
fn get_auth_photo_line(m: &Membre) -> Line<'static> {
	Line::from(vec![
		"Autorisation de photo: ".white().bold(),
		match m.auth_photo {
			Some(true) => "Oui".green(),
			_ => "Non".red(),
		}
	])
}
fn get_comment_line(m: &Membre) -> Line<'static> {
	Line::from(vec![
		"Commentaire: ".white().bold(),
		m.commentaire.clone().unwrap_or_default().gray(),
	])
}

fn get_compte_block(compte: Option<&Compte>) -> Vec<Line<'static>> {
	vec![
		get_mandataire_line(compte),
		get_tel_line(compte),
		get_email_line(compte),
		get_adresse_line(compte),
	]
}
fn get_mandataire_line(compte: Option<&Compte>) -> Line<'static> {
	Line::from(vec![
		"Mandataire: ".white().bold(),
		compte.map(|c| c.mandataire.clone()).unwrap_or_default().gray(),
	])
}
fn get_tel_line(compte: Option<&Compte>) -> Line<'static> {
	Line::from(vec![
		"Téléphone: ".white().bold(),
		compte.and_then(|c| c.tel).map(|t| t.to_string()).unwrap_or_default().gray(),
	])
}
fn get_email_line(compte: Option<&Compte>) -> Line<'static> {
	Line::from(vec![
		"Email: ".white().bold(),
		compte.and_then(|c| c.email.as_ref()).map(Email::to_string).unwrap_or_default().gray(),
	])
}
fn get_adresse_line(compte: Option<&Compte>) -> Line<'static> {
	Line::from(vec![
		"Adresse: ".white().bold(),
		compte.and_then(|c| c.adresse.as_ref()).map(Adresse::full).unwrap_or_default().gray(),
	])
}

fn get_block_fiche_sante(m: &Membre) -> Vec<Line<'static>> {
	vec![
		Line::from("Fiche santé").white().bold().centered().underlined(),
		Line::from(vec![
			"Assurance Maladie: ".white().bold(),
			m.fiche_sante.cam.map(|c| c.to_string()).unwrap_or_default().gray(),
		]),
		Line::from(vec![
			"Autorisation de soins: ".white().bold(),
			match m.fiche_sante.auth_soins {
				Some(true) => "Oui".green(),
				_ => "Non".red(),
			}
		]),
		Line::from(vec![
			"Allergies: ".white().bold(),
			m.fiche_sante.allergies.join(", ").gray(),
		]),
		Line::from(vec![
			"Maladies: ".white().bold(),
			m.fiche_sante.maladies.join(", ").gray(),
		]),
		Line::from(vec![
			"Problémes de comportement: ".white().bold(),
			m.fiche_sante.probleme_comportement.as_ref().map(BoolJustifie::to_string).unwrap_or_default().gray(),
		]),
		Line::from(vec![
			"Prise de Médicaments: ".white().bold(),
			m.fiche_sante.prise_med.as_ref().map(BoolJustifie::to_string).unwrap_or_default().gray(),
		]),
		Line::from(vec![
			"Autorisation de médicaments: ".white().bold(),
			m.fiche_sante.auth_medicaments.ls().gray(),
		]),
	]
}
fn get_block_contacts(m: &Membre) -> Vec<Line<'static>> {
	vec![
		vec![Line::from("Contacts").white().bold().centered().underlined()],
		get_block_contact_inner(m.contacts[0].as_ref()),
		get_block_contact_inner(m.contacts[1].as_ref()),
	].into_iter().flatten().collect()
}
fn get_block_contact_inner(c: Option<&Contact>) -> Vec<Line<'static>> {
	if let Some(c) = c {
		vec![
			Line::from(vec![
				"Nom: ".white().bold(),
				c.nom.clone().gray(),
			]),
			Line::from(vec![
				"Tel: ".white().bold(),
				c.tel.map(|t| t.to_string()).unwrap_or_default().gray(),
			]),
			Line::from(vec![
				"Lien: ".white().bold(),
				c.lien.clone().unwrap_or_default().gray(),
			]),
		]
	} else {
		Vec::new()
	}
}
fn get_block_depart(m: &Membre) -> Vec<Line<'static>> {
	vec![
		Line::from("Départ").white().bold().centered().underlined(),
		Line::from(vec![
			"Quitte avec: ".white().bold(),
			m.quitte.avec.join(", ").gray(),
		]),
		Line::from(vec![
			"Mot de passe: ".white().bold(),
			m.quitte.mdp.clone().unwrap_or_default().gray(),
		]),
	]
}
fn get_block_piscine(m: &Membre) -> Vec<Line<'static>> {
	vec![
		Line::from("Piscine").white().bold().centered().underlined(),
		Line::from(vec![
			"Autorisation de partage avec les sauveteurs: ".white().bold(),
			match m.piscine.partage {
				Some(true) => "Oui".green(),
				_ => "Non".red(),
			}
		]),
		Line::from(vec![
			"VFI: ".white().bold(),
			match m.piscine.vfi {
				Some(true) => "Oui".green(),
				_ => "Non".red(),
			}
		]),
		Line::from(vec![
			"Peut mettre la tête sous l'eau: ".white().bold(),
			match m.piscine.tete_sous_eau {
				Some(true) => "Oui".green(),
				_ => "Non".red(),
			}
		]),
	]
}
fn get_block_interets(m: &Membre) -> Vec<Line<'static>> {
	vec![
		Line::from("Intérêts").white().bold().centered().underlined(),
		Line::from(vec![
			"1. ".white().bold(),
			m.interets[0].map(|i| i.to_string()).unwrap_or_default().gray(),
		]),
		Line::from(vec![
			"2. ".white().bold(),
			m.interets[1].map(|i| i.to_string()).unwrap_or_default().gray(),
		]),
		Line::from(vec![
			"3. ".white().bold(),
			m.interets[2].map(|i| i.to_string()).unwrap_or_default().gray(),
		]),
		Line::from(vec![
			"4. ".white().bold(),
			m.interets[3].map(|i| i.to_string()).unwrap_or_default().gray(),
		]),
	]
}
fn get_block_inscriptions(grps: &[GroupeLine]) -> Vec<Line<'static>> {
	vec![
		Line::from("Inscriptions").white().bold().centered().underlined(),
	]
	.into_iter()
	.chain(grps.iter().map(|g| Line::from(g.desc.clone()).gray()))
	.collect()
}