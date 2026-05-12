use std::{cell::Cell, sync::{Arc, RwLock}};

use lazy_static::lazy_static;
use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::{Color, Style, Stylize}, text::Line, widgets::{Clear, Paragraph, Row, StatefulWidget, Table, TableState, Widget, WidgetRef, Wrap}};

use crate::{cdj::{comptes::{Compte, CompteID, CompteReg}, groupes::{GroupeID, GroupeKey, GroupeReg}, membres::{Membre, MembreID}}, data::adresse::Adresse, prelude::AsStr, ui::{Screen, UIError, UpdateAction, fit_str_width, screens::{Field, FieldBlock, FieldBlockCluster, FieldType, Menu, MenuItem, PageError, VIEW_TABLE_BLOCK, VIEW_TABLE_HEADER_BLOCK, stylize_selection}}};

lazy_static!{
	pub static ref PAGE_MEMBRE_GROUP_TABLE_HEADER: Row<'static> = Row::new(vec![
		Line::from("Saison").white().bold(),
		Line::from("Activité").white().bold(),
		Line::from("Site").white().bold(),
		Line::from("Catégorie").white().bold(),
		Line::from("Semaine").white().bold(),
		Line::from("Discriminant").white().bold(),
		Line::from("Sous-Groupe").white().bold(),
		Line::default(), // extra column to fill space
	]);
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
enum PageMembreView {
	General,
	FicheSante,
	Groupes,
}
impl<'a> AsStr<'static, 'a> for PageMembreView {
	fn as_str(&'a self) -> &'static str {
		match self {
			PageMembreView::General => "Général",
			PageMembreView::FicheSante => "Fiche Santé",
			PageMembreView::Groupes => "Groupes",
		}
	}
}
impl std::fmt::Display for PageMembreView {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
impl PageMembreView {
	fn next(self) -> Self {
		match self {
			PageMembreView::General => PageMembreView::FicheSante,
			PageMembreView::FicheSante => PageMembreView::Groupes,
			PageMembreView::Groupes => PageMembreView::General,
		}
	}
}

#[derive(Debug)]
struct ViewGeneral {
	naissance: Arc<RwLock<Field>>,
	genre: Arc<RwLock<Field>>,
	accompagnement: Arc<RwLock<Field>>,
	auth_photo: Arc<RwLock<Field>>,
	taille: Arc<RwLock<Field>>,
	commentaire: Arc<RwLock<Field>>,
	interets: [Arc<RwLock<Field>>; 4],
	#[allow(dead_code)]
	mandataire: Arc<RwLock<Field>>,
	email: Arc<RwLock<Field>>,
	tel: Arc<RwLock<Field>>,
	adresse: Arc<RwLock<Field>>,
	contact_1: Arc<RwLock<Field>>,
	contact_2: Arc<RwLock<Field>>,
	quitte_avec: Arc<RwLock<Field>>,
	mdp: Arc<RwLock<Field>>,
	ordre: Vec<Arc<RwLock<Field>>>,
	cluster: FieldBlockCluster,
	sel: Option<usize>,
	scroll: Cell<u16>,
	//cid: Option<CompteID>,
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
	fn handle_event(&mut self, event: crate::ui::event::Event, state: std::sync::Arc<crate::ui::AppState>) -> Result<crate::ui::actions::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Up => {
						// decrement sel
						let sel = self.sel.unwrap_or(0).saturating_sub(1);
						self.sel = Some(sel);
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Down => {
						// increment sel
						let sel = if let Some(sel) = self.sel { sel.saturating_add(1).min(self.ordre.len()-1) } else { 0 };
						self.sel = Some(sel);
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Enter => {
						if let Some(sel) = self.sel {
							let field = self.ordre[sel].clone();

							/*
							// check if it's the compte mandataire for special behaviour
							if let Some(cid) = self.cid {
								// open the compte view for the mandataire
								if Arc::ptr_eq(&field, &self.mandataire) {
									return Ok(UpdateAction::OpenCompte(cid).one());
								}
							}
							*/

							Ok(Field::on_action(field, None))
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
impl ViewGeneral {
	fn try_new(membre: &Membre, _compte: Option<&Compte>, _groupes: &[GroupeLine]) -> Result<Self, UIError> {
		let cid = _compte.map(|c| c.id);

		let mut block_intro = FieldBlock::default();
		let naissance = block_intro.add_field(Field::from(membre.naissance)
			.with_label("Naissance".into()));
		let genre = block_intro.add_field(Field::from(membre.genre)
			.with_label("Genre".into()));
		let accompagnement = block_intro.add_field(Field::from(membre.accompagnement)
			.with_label("Accompagnement".into()));
		let auth_photo = block_intro.add_field(Field::from(membre.auth_photo)
			.with_label("Photo autorisée".into()));
		let taille = block_intro.add_field(Field::from(membre.taille)
			.with_label("Taille".into()));
		let commentaire = block_intro.add_field(Field::from(membre.commentaire.as_deref())
			.with_label("Commentaire".into()));

		let mut block_compte = FieldBlock::default();
		let (mandataire, email, tel, adresse) = if let Some(compte) = _compte {
			(
				block_compte.add_field(Field::from(compte.mandataire.as_str()).with_label("Mandataire".into()).set_editable(false).with_associated_page(super::AssociatedPage::Compte { cid: compte.id })),
				block_compte.add_field(Field::from(compte.email.clone()).with_label("Email".into()).with_associated_page(super::AssociatedPage::Compte { cid: compte.id })),
				block_compte.add_field(Field::from(compte.tel).with_label("Téléphone".into()).with_associated_page(super::AssociatedPage::Compte { cid: compte.id })),
				block_compte.add_field(Field::from(compte.adresse.clone()).with_label("Adresse".into()).with_associated_page(super::AssociatedPage::Compte { cid: compte.id })),
			)
		} else {
			(
				block_compte.add_field(Field::from(FieldType::Str(None)).with_label("Mandataire".into()).set_editable(false)),
				block_compte.add_field(Field::from(FieldType::Email(None)).with_label("Email".into())),
				block_compte.add_field(Field::from(FieldType::Tel(None)).with_label("Téléphone".into())),
				block_compte.add_field(Field::from(FieldType::Adresse(Adresse::default())).with_label("Adresse".into())),
			)
		};

		let mut block_contacts = FieldBlock::default().with_title(Line::from("Contacts").white().bold().centered().underlined());
		let contact_1 = block_contacts.add_field(Field::from(membre.contacts[0].clone()).with_label("Contact 1".into()));
		let contact_2 = block_contacts.add_field(Field::from(membre.contacts[1].clone()).with_label("Contact 2".into()));

		let mut block_depart = FieldBlock::default().with_title(Line::from("Départ").white().bold().centered().underlined());
		let quitte_avec = membre.quitte.avec.iter().map(String::as_str).collect::<Vec<_>>().join(", ");
		let quitte_avec = block_depart.add_field(Field::from(quitte_avec.as_str()).with_label("Quitte Avec".into()));
		let mdp = block_depart.add_field(Field::from(membre.quitte.mdp.as_deref()).with_label("Mot de Passe".into()));

		let mut block_interets = FieldBlock::default().with_title(Line::from("Intérêts").white().bold().centered().underlined());
		let interets = [
			block_interets.add_field(Field::from(membre.interets[0]).with_label("Interet 1".into())),
			block_interets.add_field(Field::from(membre.interets[1]).with_label("Interet 2".into())),
			block_interets.add_field(Field::from(membre.interets[2]).with_label("Interet 3".into())),
			block_interets.add_field(Field::from(membre.interets[3]).with_label("Interet 4".into())),
		];

		let cluster = if cid.is_some() { // on ne montre pas le compte s'il n'y en a pas, pour éviter de faire croire à l'utilisateur qu'il peut en ajouter un
			FieldBlockCluster::new(vec![block_intro, block_compte, block_contacts, block_depart, block_interets])
		} else {
			FieldBlockCluster::new(vec![block_intro, block_contacts, block_depart, block_interets])
		};
		let ordre = if cid.is_some() {
			vec![
				naissance.clone(),
				genre.clone(),
				accompagnement.clone(),
				auth_photo.clone(),
				taille.clone(),
				commentaire.clone(),
				mandataire.clone(),
				email.clone(),
				tel.clone(),
				adresse.clone(),
				contact_1.clone(),
				contact_2.clone(),
				quitte_avec.clone(),
				mdp.clone(),
				interets[0].clone(),
				interets[1].clone(),
				interets[2].clone(),
				interets[3].clone(),
			]
		} else {
			vec![
				naissance.clone(),
				genre.clone(),
				accompagnement.clone(),
				auth_photo.clone(),
				taille.clone(),
				commentaire.clone(),
				contact_1.clone(),
				contact_2.clone(),
				quitte_avec.clone(),
				mdp.clone(),
				interets[0].clone(),
				interets[1].clone(),
				interets[2].clone(),
				interets[3].clone(),
			]
		};
		Ok(Self {
			naissance,
			genre,
			accompagnement,
			auth_photo,
			taille,
			commentaire,
			interets,
			mandataire,
			email,
			tel,
			adresse,
			contact_1,
			contact_2,
			quitte_avec,
			mdp,
			ordre,
			cluster,
			sel: None,
			scroll: Cell::new(0),
			//cid,
		})
	}
	fn update(&mut self, membre: &Membre, compte: Option<&Compte>) {
		self.naissance.write().expect("Poisoned Lock").set_value(FieldType::from(membre.naissance));
		self.genre.write().expect("Poisoned Lock").set_value(FieldType::from(membre.genre));
		self.accompagnement.write().expect("Poisoned Lock").set_value(FieldType::from(membre.accompagnement));
		self.auth_photo.write().expect("Poisoned Lock").set_value(FieldType::from(membre.auth_photo));
		self.taille.write().expect("Poisoned Lock").set_value(FieldType::from(membre.taille));
		self.commentaire.write().expect("Poisoned Lock").set_value(FieldType::from(membre.commentaire.as_deref()));
		for (i, interet) in membre.interets.iter().enumerate() {
			self.interets[i].write().expect("Poisoned Lock").set_value(FieldType::from(*interet));
		}
		if let Some(compte) = compte {
			self.mandataire.write().expect("Poisoned Lock").set_value(FieldType::from(compte.mandataire.as_str()));
			self.email.write().expect("Poisoned Lock").set_value(FieldType::from(compte.email.clone()));
			self.tel.write().expect("Poisoned Lock").set_value(FieldType::from(compte.tel));
			self.adresse.write().expect("Poisoned Lock").set_value(FieldType::Adresse(compte.adresse.clone().unwrap_or_default()));
		} else {
			self.mandataire.write().expect("Poisoned Lock").set_value(FieldType::Str(None));
			self.email.write().expect("Poisoned Lock").set_value(FieldType::Email(None));
			self.tel.write().expect("Poisoned Lock").set_value(FieldType::Tel(None));
			self.adresse.write().expect("Poisoned Lock").set_value(FieldType::Adresse(Adresse::default()));
		}
		self.contact_1.write().expect("Poisoned Lock").set_value(FieldType::from(membre.contacts[0].clone()));
		self.contact_2.write().expect("Poisoned Lock").set_value(FieldType::from(membre.contacts[1].clone()));
		let quitte_avec = membre.quitte.avec.iter().map(String::as_str).collect::<Vec<_>>().join(", ");
		self.quitte_avec.write().expect("Poisoned Lock").set_value(FieldType::from(quitte_avec.as_str()));
		self.mdp.write().expect("Poisoned Lock").set_value(FieldType::from(membre.quitte.mdp.as_deref()));
	}
	fn build_membre(&self, membre: &mut Membre, compte: &mut Option<&mut Compte>) {
		membre.naissance = self.naissance.read().expect("Poisoned Lock").get_date().flatten().copied().unwrap_or(membre.naissance);
		membre.genre = self.genre.read().expect("Poisoned Lock").get_genre().flatten().copied();
		membre.accompagnement = self.accompagnement.read().expect("Poisoned Lock").get_bool().flatten();
		membre.auth_photo = self.auth_photo.read().expect("Poisoned Lock").get_bool().flatten();
		membre.taille = self.taille.read().expect("Poisoned Lock").get_taille().flatten().copied();
		membre.commentaire = self.commentaire.read().expect("Poisoned Lock").get_str().flatten().map(str::to_string);
		membre.contacts[0] = self.contact_1.read().expect("Poisoned Lock").get_contact().flatten().cloned();
		membre.contacts[1] = self.contact_2.read().expect("Poisoned Lock").get_contact().flatten().cloned();
		membre.quitte.avec = self.quitte_avec.read().expect("Poisoned Lock").get_str().flatten().map(|s| s.split(',').map(str::trim).map(str::to_string).collect()).unwrap_or(membre.quitte.avec.clone());
		membre.quitte.mdp = self.mdp.read().expect("Poisoned Lock").get_str().flatten().map(str::to_string);
		for i in 0..4 {
			membre.interets[i] = self.interets[i].read().expect("Poisoned Lock").get_interet().flatten().copied();
		}

		if let Some(compte) = compte {
			compte.email = self.email.read().expect("Poisoned Lock").get_email().flatten().cloned();
			compte.tel = self.tel.read().expect("Poisoned Lock").get_tel().flatten().cloned();
			compte.adresse = Some(self.adresse.read().expect("Poisoned Lock").get_adresse().cloned().expect("Should be an address"));
		}
	}
	fn reset(&mut self) {
		self.scroll.set(0);
		self.sel = None;
	}
}

#[derive(Debug)]
struct ViewFicheSante {
	allergies: Arc<RwLock<Field>>,
	medicaments: Arc<RwLock<Field>>,
	maladies: Arc<RwLock<Field>>,
	auth_soins: Arc<RwLock<Field>>,
	probleme_comportement: Arc<RwLock<Field>>,
	cam: Arc<RwLock<Field>>,
	auth_medicament_sirop_toux: Arc<RwLock<Field>>,
	auth_medicament_anti_emetique: Arc<RwLock<Field>>,
	auth_medicament_ibuprofene: Arc<RwLock<Field>>,
	auth_medicament_anti_inflamatoire: Arc<RwLock<Field>>,
	auth_medicament_anti_biotique: Arc<RwLock<Field>>,
	auth_medicament_acetaminophene: Arc<RwLock<Field>>,
	partage_sauveteur: Arc<RwLock<Field>>,
	vfi: Arc<RwLock<Field>>,
	tete_sous_eau: Arc<RwLock<Field>>,
	cluster: FieldBlockCluster,
	ordre: Vec<Arc<RwLock<Field>>>,
	sel: Option<usize>,
	scroll: Cell<u16>,
}
impl WidgetRef for ViewFicheSante {
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
impl Screen for ViewFicheSante {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: std::sync::Arc<crate::ui::AppState>) -> Result<crate::ui::actions::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Up => {
						// decrement sel
						let sel = self.sel.unwrap_or(0).saturating_sub(1);
						self.sel = Some(sel);
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Down => {
						// increment sel
						let sel = if let Some(sel) = self.sel { sel.saturating_add(1).min(self.ordre.len()-1) } else { 0 };
						self.sel = Some(sel);
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Enter => {
						if let Some(sel) = self.sel {
							let field = self.ordre[sel].clone();
							Ok(Field::on_action(field, None))
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
impl ViewFicheSante {
	fn try_new(membre: &Membre, _compte: Option<&Compte>, _groupes: &[GroupeLine]) -> Result<Self, UIError> {
		let mut general_block = FieldBlock::default();
		let cam = general_block.add_field(Field::from(membre.fiche_sante.cam).with_label("Assurance Maladie".into()));
		let auth_soins = general_block.add_field(Field::from(membre.fiche_sante.auth_soins).with_label("Soins autorisés".into()));
		let allergies = membre.fiche_sante.allergies.iter().map(String::as_str).collect::<Vec<_>>().join(", ");
		let allergies = general_block.add_field(Field::from(allergies.as_str()).with_label("Allergies".into()));
		let maladies = membre.fiche_sante.maladies.iter().map(String::as_str).collect::<Vec<_>>().join(", ");
		let maladies = general_block.add_field(Field::from(maladies.as_str()).with_label("Maladies".into()));
		let medicaments = general_block.add_field(Field::from(membre.fiche_sante.prise_med.clone()).with_label("Prise de Médicament".into()));
		let probleme_comportement = general_block.add_field(Field::from(membre.fiche_sante.probleme_comportement.clone()).with_label("Problème de Comportement".into()));

		let mut block_auth_meds = FieldBlock::default().with_title(Line::from("Médicaments autorisés").white().bold().centered().underlined());
		let auth_medicament_acetaminophene = block_auth_meds.add_field(Field::from(membre.fiche_sante.auth_medicaments.acetaminophene).with_label("Acétaminophène".into()));
		let auth_medicament_anti_biotique = block_auth_meds.add_field(Field::from(membre.fiche_sante.auth_medicaments.anti_biotique).with_label("Anti-Biotique".into()));
		let auth_medicament_anti_emetique = block_auth_meds.add_field(Field::from(membre.fiche_sante.auth_medicaments.anti_emetique).with_label("Anti-Emétique".into()));
		let auth_medicament_anti_inflamatoire = block_auth_meds.add_field(Field::from(membre.fiche_sante.auth_medicaments.anti_inflamatoire).with_label("Anti-Inflamatoire".into()));
		let auth_medicament_ibuprofene = block_auth_meds.add_field(Field::from(membre.fiche_sante.auth_medicaments.ibuprofene).with_label("Ibuprofène".into()));
		let auth_medicament_sirop_toux = block_auth_meds.add_field(Field::from(membre.fiche_sante.auth_medicaments.sirop_toux).with_label("Sirop Pour la Toux".into()));
		
		let mut block_piscine = FieldBlock::default().with_title(Line::from("Piscine").white().bold().centered().underlined());
		let partage_sauveteur = block_piscine.add_field(Field::from(membre.piscine.partage).with_label("Authorisation de Partage Avec les Sauveteurs".into()));
		let vfi = block_piscine.add_field(Field::from(membre.piscine.vfi).with_label("Doit Mettre un VFI".into()));
		let tete_sous_eau = block_piscine.add_field(Field::from(membre.piscine.tete_sous_eau).with_label("Peut Mettre laTête sous l'eau".into()));

		let cluster = FieldBlockCluster::new(vec![general_block, block_auth_meds, block_piscine]);
		let ordre = vec![
			cam.clone(),
			auth_soins.clone(),
			allergies.clone(),
			maladies.clone(),
			medicaments.clone(),
			probleme_comportement.clone(),
			auth_medicament_acetaminophene.clone(),
			auth_medicament_anti_biotique.clone(),
			auth_medicament_anti_emetique.clone(),
			auth_medicament_anti_inflamatoire.clone(),
			auth_medicament_ibuprofene.clone(),
			auth_medicament_sirop_toux.clone(),
			partage_sauveteur.clone(),
			vfi.clone(),
			tete_sous_eau.clone(),
		];

		Ok(Self {
			allergies,
			medicaments,
			maladies,
			auth_soins,
			probleme_comportement,
			cam,
			auth_medicament_sirop_toux,
			auth_medicament_anti_emetique,
			auth_medicament_ibuprofene,
			auth_medicament_anti_inflamatoire,
			auth_medicament_anti_biotique,
			auth_medicament_acetaminophene,
			partage_sauveteur,
			vfi,
			tete_sous_eau,
			cluster,
			ordre,
			sel: None,
			scroll: Cell::new(0),
		})
	}
	fn update(&mut self, membre: &Membre) {
		let allergies = membre.fiche_sante.allergies.iter().map(String::as_str).collect::<Vec<_>>().join(", ");
		self.allergies.write().expect("Poisoned Lock").set_value(FieldType::from(allergies.as_str()));
		let maladies = membre.fiche_sante.maladies.iter().map(String::as_str).collect::<Vec<_>>().join(", ");
		self.maladies.write().expect("Poisoned Lock").set_value(FieldType::from(maladies.as_str()));
		self.auth_soins.write().expect("Poisoned Lock").set_value(FieldType::from(membre.fiche_sante.auth_soins));
		self.probleme_comportement.write().expect("Poisoned Lock").set_value(FieldType::from(membre.fiche_sante.probleme_comportement.clone()));
		self.cam.write().expect("Poisoned Lock").set_value(FieldType::from(membre.fiche_sante.cam));
		self.medicaments.write().expect("Poisoned Lock").set_value(FieldType::from(membre.fiche_sante.prise_med.clone()));
		self.auth_medicament_acetaminophene.write().expect("Poisoned Lock").set_value(FieldType::from(membre.fiche_sante.auth_medicaments.acetaminophene));
		self.auth_medicament_anti_biotique.write().expect("Poisoned Lock").set_value(FieldType::from(membre.fiche_sante.auth_medicaments.anti_biotique));
		self.auth_medicament_anti_emetique.write().expect("Poisoned Lock").set_value(FieldType::from(membre.fiche_sante.auth_medicaments.anti_emetique));
		self.auth_medicament_anti_inflamatoire.write().expect("Poisoned Lock").set_value(FieldType::from(membre.fiche_sante.auth_medicaments.anti_inflamatoire));
		self.auth_medicament_ibuprofene.write().expect("Poisoned Lock").set_value(FieldType::from(membre.fiche_sante.auth_medicaments.ibuprofene));
		self.auth_medicament_sirop_toux.write().expect("Poisoned Lock").set_value(FieldType::from(membre.fiche_sante.auth_medicaments.sirop_toux));
		self.partage_sauveteur.write().expect("Poisoned Lock").set_value(FieldType::from(membre.piscine.partage));
		self.vfi.write().expect("Poisoned Lock").set_value(FieldType::from(membre.piscine.vfi));
		self.tete_sous_eau.write().expect("Poisoned Lock").set_value(FieldType::from(membre.piscine.tete_sous_eau));
	}
	fn build_membre(&self, membre: &mut Membre, _compte: &mut Option<&mut Compte>) {
		membre.fiche_sante.allergies = self.allergies.read().expect("Poisoned Lock").get_str().flatten().map(|s| s.split(',').map(str::trim).map(str::to_string).collect()).unwrap_or(membre.fiche_sante.allergies.clone());
		membre.fiche_sante.maladies = self.maladies.read().expect("Poisoned Lock").get_str().flatten().map(|s| s.split(',').map(str::trim).map(str::to_string).collect()).unwrap_or(membre.fiche_sante.maladies.clone());
		membre.fiche_sante.auth_soins = self.auth_soins.read().expect("Poisoned Lock").get_bool().flatten();
		membre.fiche_sante.probleme_comportement = self.probleme_comportement.read().expect("Poisoned Lock").get_bool_justifie().flatten().cloned();
		membre.fiche_sante.cam = self.cam.read().expect("Poisoned Lock").get_cam().flatten().copied();
		membre.fiche_sante.prise_med = self.medicaments.read().expect("Poisoned Lock").get_bool_justifie().flatten().cloned();
		membre.fiche_sante.auth_medicaments.acetaminophene = self.auth_medicament_acetaminophene.read().expect("Poisoned Lock").get_bool().flatten();
		membre.fiche_sante.auth_medicaments.anti_biotique = self.auth_medicament_anti_biotique.read().expect("Poisoned Lock").get_bool().flatten();
		membre.fiche_sante.auth_medicaments.anti_emetique = self.auth_medicament_anti_emetique.read().expect("Poisoned Lock").get_bool().flatten();
		membre.fiche_sante.auth_medicaments.anti_inflamatoire = self.auth_medicament_anti_inflamatoire.read().expect("Poisoned Lock").get_bool().flatten();
		membre.fiche_sante.auth_medicaments.ibuprofene = self.auth_medicament_ibuprofene.read().expect("Poisoned Lock").get_bool().flatten();
		membre.fiche_sante.auth_medicaments.sirop_toux = self.auth_medicament_sirop_toux.read().expect("Poisoned Lock").get_bool().flatten();
		membre.piscine.partage = self.partage_sauveteur.read().expect("Poisoned Lock").get_bool().flatten();
		membre.piscine.vfi = self.vfi.read().expect("Poisoned Lock").get_bool().flatten();
		membre.piscine.tete_sous_eau = self.tete_sous_eau.read().expect("Poisoned Lock").get_bool().flatten();
	}
	fn reset(&mut self) {
		self.scroll.set(0);
		self.sel = None;
	}
}

#[derive(Debug, Clone)]
struct GroupeLine {
	id: GroupeID,
	key: GroupeKey,
}

#[derive(Debug)]
struct ViewGroupes {
	groupes: Vec<GroupeLine>,
	table_state: RwLock<TableState>,
	widths: [Constraint; 8],
	_mid: MembreID,
}
impl WidgetRef for ViewGroupes {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		let table = Table::new(
			self.groupes.iter().map(|g| group_key_to_row(&g.key)),
			&self.widths,
		)
			.header(PAGE_MEMBRE_GROUP_TABLE_HEADER.clone())
			.row_highlight_style(Style::new().white().on_dark_gray())
			.style(Style::new().gray());
		let mut lock = self.table_state.write().expect("Poisoned Lock");
		StatefulWidget::render(table, area, buf, &mut lock);
	}
}
impl Screen for ViewGroupes {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: std::sync::Arc<crate::ui::AppState>) -> Result<crate::ui::actions::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Up => {
						// decrement sel
						let mut lock = self.table_state.write().expect("Poisoned Lock");
						let sel = lock.selected().map(|s| s.saturating_sub(1)).unwrap_or(0);
						lock.select(Some(sel));
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Down => {
						// increment sel
						let mut lock = self.table_state.write().expect("Poisoned Lock");
						let sel = lock.selected().map(|s| s.saturating_add(1).min(self.groupes.len()-1)).unwrap_or(0);
						lock.select(Some(sel));
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Enter => {
						if let Some(sel) = self.table_state.read().expect("Poisoned Lock").selected() {
							let g = self.groupes[sel].clone();

							let menu = Menu::new(Box::new([
								MenuItem {id: "Voir le groupe", action: Box::new(move |state| {
									Ok(vec![
										UpdateAction::Pop,
										UpdateAction::OpenGroupe(g.id, g.key.sous_groupe),
									])
								})},
								MenuItem {id: "Changer le membre de sous-groupe", action: Box::new(move |state| {
									Ok(UpdateAction::Pop.one())
								})},
							]));

							Ok(UpdateAction::PushSub(Box::new(menu)).one())
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
impl ViewGroupes {
	fn try_new(_membre: &Membre, _compte: Option<&Compte>, groupes: &[GroupeLine]) -> Result<Self, UIError> {
		Ok(Self {
			groupes: groupes.to_vec(),
			table_state: RwLock::new(TableState::default()),
			widths: [Constraint::default(); 8],
			_mid: _membre.id,
		})
	}
	fn update(&mut self, groupes: &[GroupeLine]) {
		self.groupes = groupes.to_vec();
		self.reset();
		self.fit_widths();
	}
	fn reset(&mut self) {
		self.table_state = RwLock::new(TableState::default());
	}
	fn fit_widths(&mut self) {
		let saison = fit_str_width(self.groupes.iter().map(|g| g.key.saison.as_deref().unwrap_or("")).chain(std::iter::once("Saison")));
		let activite = fit_str_width(self.groupes.iter().map(|g| g.key.activite.as_deref().unwrap_or("")).chain(std::iter::once("Activité")));
		let site = fit_str_width(self.groupes.iter().map(|g| g.key.site.as_deref().unwrap_or("")).chain(std::iter::once("Site")));
		let semaine = fit_str_width(self.groupes.iter().map(|g| g.key.semaine.as_deref().unwrap_or("")).chain(std::iter::once("Semaine")));
		let category = fit_str_width(self.groupes.iter().map(|g| g.key.category.as_deref().unwrap_or("")).chain(std::iter::once("Catégorie")));
		let discriminant = fit_str_width(self.groupes.iter().map(|g| g.key.discriminant.as_deref().unwrap_or("")).chain(std::iter::once("Discriminant")));
		let sous_groupe = self.groupes.iter().filter_map(|g| g.key.sous_groupe.map(|sg| sg.to_string())).collect::<Vec<_>>();
		let sous_groupe = fit_str_width(sous_groupe.iter().map(|s| s.as_str()).chain(std::iter::once("Sous-Groupe")));
		self.widths = [
			Constraint::Length(saison as u16 + 2),
			Constraint::Length(activite as u16 + 2),
			Constraint::Length(site as u16 + 2),
			Constraint::Length(semaine as u16 + 2),
			Constraint::Length(category as u16 + 2),
			Constraint::Length(discriminant as u16 + 2),
			Constraint::Length(sous_groupe as u16 + 2),
			Constraint::Fill(1),
		];
	}
}

#[derive(Debug)]
pub struct PageMembre {
	sel_view: PageMembreView,
	view_general: ViewGeneral,
	view_fiche_sante: ViewFicheSante,
	view_groupes: ViewGroupes,
	mid: MembreID,
	cid: Option<CompteID>,
	title: Line<'static>,
}
impl PageMembre {
	pub fn try_new(membre: &Membre, comptes: &CompteReg, groupes: &GroupeReg) -> Result<Self, UIError> {
		let compte = if let Some(cid) = membre.compte {
			match comptes.get(cid) {
				Ok(c) => Some(c.clone()),
				Err(_) => return Err(UIError::Others { src: Box::new(PageError::MissingData { msg: "Le compte du membre n'existe pas".into() }) }),
			}
		} else {
			None
		};
		let mut grps: Vec<GroupeLine> = groupes.groupes().filter_map(|g| {
			g.get_key_for(membre.id).map(|key| GroupeLine { id: g.id, key })
		}).collect();
		grps.sort_by(|a, b| a.key.cmp(&b.key));
		let title = Line::from(format!(" {}, {} ", membre.nom, membre.prenom).white().bold());
		let mut s = Self {
			view_general: ViewGeneral::try_new(membre, compte.as_ref(), &grps)?,
			view_fiche_sante: ViewFicheSante::try_new(membre, compte.as_ref(), &grps)?,
			view_groupes: ViewGroupes::try_new(membre, compte.as_ref(), &grps)?,
			mid: membre.id,
			sel_view: PageMembreView::General,
			title,
			cid: compte.map(|c| c.id),
		};
		s.view_groupes.fit_widths();
		Ok(s)
	}
	fn build_membre(&self, membre: &mut Membre, mut compte: Option<&mut Compte>) {
		self.view_general.build_membre(membre, &mut compte);
		self.view_fiche_sante.build_membre(membre, &mut compte);
	}
}
impl WidgetRef for PageMembre {
	fn render_ref(&self,area: ratatui::prelude::Rect,buf: &mut ratatui::prelude::Buffer) {
		Clear.render(area, buf);

		// render the block (border + title + instructions)
		let block = VIEW_TABLE_BLOCK.clone()
			.title_top(self.title.clone())
			.bg(Color::Black);
		let inner = block.inner(area);
		block.render(area, buf);

		// create the header
		let header = Line::from(vec![
			stylize_selection(&self.sel_view, &PageMembreView::General),
			" | ".gray(),
			stylize_selection(&self.sel_view, &PageMembreView::FicheSante),
			" | ".gray(),
			stylize_selection(&self.sel_view, &PageMembreView::Groupes),
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

		// render the selected table
		let table_area = Rect {
			x: inner.x,
			y: inner.y + 2, // header + separator
			width: inner.width,
			height: inner.height.saturating_sub(2),
		};
		match self.sel_view {
			PageMembreView::General => self.view_general.render_ref(table_area, buf),
			PageMembreView::FicheSante => self.view_fiche_sante.render_ref(table_area, buf),
			PageMembreView::Groupes => self.view_groupes.render_ref(table_area, buf),
		}
	}
}
impl Screen for PageMembre {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: std::sync::Arc<crate::ui::AppState>) -> Result<crate::ui::actions::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Tab => {
						self.sel_view = self.sel_view.next();
						// update the membre and compte
						{
							let mut membres = state.membres.write().expect("Poisoned Lock");
							let mut comptes = state.comptes.write().expect("Poisoned Lock");
							let membre = membres.get_mut(self.mid).expect("Membre Inexistant");
							let compte = self.cid.map(|cid| comptes.get_mut(cid).expect("Compte Inexistant"));
							self.build_membre(membre, compte);
						}
						match self.sel_view {
							PageMembreView::General => self.view_general.reset(),
							PageMembreView::FicheSante => self.view_fiche_sante.reset(),
							PageMembreView::Groupes => self.view_groupes.reset(),
						}
						Ok(UpdateAction::Redraw.one())
					},
					cte::KeyCode::Esc => {
						{
							let mut membres = state.membres.write().expect("Poisoned Lock");
							let mut comptes = state.comptes.write().expect("Poisoned Lock");
							let membre = membres.get_mut(self.mid).expect("Membre Inexistant");
							let compte = self.cid.map(|cid| comptes.get_mut(cid).expect("Compte Inexistant"));
							self.build_membre(membre, compte);
						}
						Ok(UpdateAction::Pop.one())
					},
					_ => {
						// pass the event to the selected table
						match self.sel_view {
							PageMembreView::General => self.view_general.handle_event(event, state.clone()),
							PageMembreView::FicheSante => self.view_fiche_sante.handle_event(event, state.clone()),
							PageMembreView::Groupes => self.view_groupes.handle_event(event, state.clone()),
						}
					},
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}

	fn on_refocus(&mut self, state: std::sync::Arc<crate::ui::AppState>) {
		let membres = state.membres.read().expect("Poisoned Lock");
		let comptes = state.comptes.read().expect("Poisoned Lock");
		let groupes = state.groupes.read().expect("Poisoned Lock");
		let membre = membres.get(self.mid).expect("Membre Inexistant");
		let compte = self.cid.map(|cid| comptes.get(cid).expect("Compte Inexistant"));
		let mut grps: Vec<GroupeLine> = groupes.groupes().filter_map(|g| {
			g.get_key_for(membre.id).map(|key| GroupeLine { id: g.id, key })
		}).collect();
		grps.sort_by(|a, b| a.key.cmp(&b.key));
		// update the tables with the latest data from the state
		self.view_general.update(membre, compte);
		self.view_fiche_sante.update(membre);
		self.view_groupes.update(&grps);
	}

}

fn group_key_to_row<'a>(key: &'a GroupeKey) -> Row<'a> {
	Row::new(vec![
		Line::from(key.saison.as_deref().unwrap_or("")),
		Line::from(key.activite.as_deref().unwrap_or("")),
		Line::from(key.site.as_deref().unwrap_or("")),
		Line::from(key.category.as_deref().unwrap_or("")),
		Line::from(key.semaine.as_deref().unwrap_or("")),
		Line::from(key.discriminant.as_deref().unwrap_or("")),
		Line::from(key.sous_groupe.as_ref().map(u32::to_string).unwrap_or_default()),
		Line::default(), // extra line to fill space
	])
}