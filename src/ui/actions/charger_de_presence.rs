use std::sync::Arc;

use crate::{extract::{ExtractError, excel::DataColConfig}, cdj::{comptes::CompteID, groupes::GroupeID, membres::MembreID}, ui::{AppState, UIError, screens::{Desc}}};

pub fn charger_de_presence(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let filepath = state.get_in_xlsx("Sélectionner le fichier de présence");
	if let Some(filepath) = filepath {
		// update the old_in_dir to the parent of the selected file
		if let Some(parent) = filepath.parent() {
			let mut lock = state.old_in_dir.write().expect("Poisoned Lock");
			*lock = parent.to_path_buf();
		}

		let screen = crate::ui::screens::ProgressLogScreen::new("Chargement des listes de présence".into(), 1);
		let target_hook = screen.get_target_hook();
		let progress_hook = screen.get_progress_hook();
		let cancel_hook = screen.get_cancel_hook();
		let log_hook = screen.get_logger();

		let thread_handle: std::thread::JoinHandle<Result<(), UIError>> = std::thread::spawn(move || {
			log_hook.lock().expect("Poisoned Lock").log(Desc::Info("Ouverture du fichier".into()));
			let mut workbook = match office::Excel::open(filepath) {
				Ok(wb) => wb,
				Err(_e) => return Err(ExtractError::CouldNotReadFile.into()),
			};
			let sheets = get_sheets(&mut workbook);
			*target_hook.lock().expect("Poisoned Lock") = sheets.len() as u32;

			let mut dc: Option<DataColConfig> = None;
			let logger = log_hook.clone();
			let logger = move |err: &str| {
				logger.lock().expect("Poisoned Lock").log(Desc::Error(err.into()));
			};
			for sheet in sheets {
				log_hook.lock().expect("Poisoned Lock").log(Desc::Info(format!("Lecture de la feuille '{}'", sheet)));

				// early check for cancel
				if *cancel_hook.lock().expect("Poisoned Lock") {
					log_hook.lock().expect("Poisoned Lock").log(Desc::Warning("Importation annulée".into()));
					return Err(UIError::CancelAction { desc: "Importation annulée par l'utilisateur".into() });
				}

				let rng = match workbook.worksheet_range(&sheet) {
					Ok(r) => r,
					Err(e) => {
						log_hook.lock().expect("Poisoned Lock").log(Desc::Error(format!("Erreur lors de la lecture de la feuille '{}': {}", sheet, e)));
						continue;
					},
				};
				let mut grp = match crate::extract::excel::extract_group_info(&rng) {
					Ok(g) => g,
					Err(e) => {
						log_hook.lock().expect("Poisoned Lock").log(Desc::Error(format!("Erreur lors de l'extraction des données de la feuille '{}': {}", sheet, e)));
						continue;
					},
				};
				let existing_grps = state.groupes.read().expect("Poisoned Lock").groupes().filter(|g| g.equiv(&grp)).map(|g| g.id).collect::<Vec<GroupeID>>();
				// get or generate gid
				let gid = if existing_grps.is_empty() {
					let mut groupes = state.groupes.write().expect("Poisoned Lock");
					let id = groupes.get_new_id_from_seed(grp.id.0);
					grp.id = id;
					if groupes.add(grp).is_ok() { // ignore error since we check for existing group before
						log_hook.lock().expect("Poisoned Lock").log(Desc::Info(format!("Groupe ajouté avec l'ID: {}", id)));
					}
					id
				} else {
					// prendre le premier groupe (devrait être le seul)
					log_hook.lock().expect("Poisoned Lock").log(Desc::Warning("Groupe déjà existant".into()));
					existing_grps[0]
				};

				// construire la configuration des colones si ce n'est pas déjà fait
				let (data_ln, ln_skip) = {
					let lock = state.config.read().expect("Poisoned Lock");
					(lock.excel.data_ln, lock.excel.ln_skip)
				};
				if dc.is_none() {
					dc = Some(DataColConfig::new(&rng, data_ln));
				}
				let dcc = dc.as_ref().expect("what?");

				// boucler sur le reste des lignes pour rajouter les membres au groupe
				{
					let mut groupes = state.groupes.write().expect("Poisoned Lock");
					let grp = groupes.get_mut(gid)?;
					let rows = rng.rows().skip(ln_skip);
					for (i, ln) in rows.enumerate() {
						match crate::extract::excel::extract_compte_info(ln, dcc, Some(&logger)) {
							Err(e) => {
								log_hook.lock().expect("Poisoned Lock").log(Desc::Error(format!("Erreur en lisant le compte (feuille '{}', ligne {}): {e}", sheet, i + ln_skip + 1)));
							},
							Ok(mut c) => {
								// trouver si le compte existe déjà dans le groupe
								let cid = {
									let mut comptes = state.comptes.write().expect("Poisoned Lock");
									let existing_compte = comptes.comptes().filter(|cc| cc.equiv(&c)).map(|c| c.id).collect::<Vec<CompteID>>();
									if existing_compte.is_empty() {
										let id = comptes.get_new_id_from_seed(c.id.0);
										c.id = id;
										if comptes.add(c).is_ok() { // ignore error since we check for existing compte before
											log_hook.lock().expect("Poisoned Lock").log(Desc::Info(format!("Compte ajouté avec l'ID: {}", id)));
										}
										id
									} else {
										existing_compte[0]
									}
								};
								
								// extraire les infos du membre
								match crate::extract::excel::extract_membre_info(ln, dcc) {
									Err(e) => {
										log_hook.lock().expect("Poisoned Lock").log(Desc::Error(format!("Erreur en lisant le membre (feuille '{}', ligne {}): {e}", sheet, i + ln_skip + 1)));
										continue;
									},
									Ok(mut m) => {
										m.compte = Some(cid);
										let mut membres = state.membres.write().expect("Poisoned Lock");
										let mid = {
											let existing_membres = membres.membres().filter(|mm| mm.equiv(&m)).map(|m| m.id).collect::<Vec<MembreID>>();
											if existing_membres.is_empty() {
												let id = membres.get_new_id_from_seed(m.id.0);
												m.id = id;
												crate::extract::excel::fill_membre_info(ln, dcc, &mut m, Some(&logger));
												if membres.add(m).is_ok() { // ignore error since we check for existing membre before
													log_hook.lock().expect("Poisoned Lock").log(Desc::Info(format!("Membre ajouté avec l'ID: {}", id)));
													//std::thread::sleep(Duration::from_millis(500));
												}
												id
											} else {
												existing_membres[0]
											}
										};
										let membre = membres.get_mut(mid)?;
										/*
										if let Err(e) = state.comptes.write().expect("Poisoned Lock").get_mut(cid)?.add_membre(membre) {
											log_hook.lock().expect("Poisoned Lock").log(Desc::Error(format!("Erreur en ajoutant le membre au compte (feuille '{}', ligne {}): {e}", sheet, i + ln_skip + 1)));
										}
										*/
										grp.add_participant(mid);
									},
								}
							},
						}
					}
				}

				// incrémenter le progrès
				{
					let mut progress = progress_hook.lock().expect("Poisoned Lock");
					*progress += 1;
				}
			}
			log_hook.lock().expect("Poisoned Lock").log(Desc::Info("Terminé".into()));
			Ok(())
		});
		let screen = screen.with_thread(thread_handle);


		Ok(crate::ui::UpdateAction::Push(Box::new(screen)).one())
	} else {
		Ok(crate::ui::UpdateAction::ErrorPopUp(Box::new(UIError::CancelAction { desc: String::from("Aucun fichier sélectionné") } )).one())
	}
}

fn get_sheets(workbook: &mut office::Excel) -> Vec<String> {
	workbook.sheet_names().expect("Failed to get sheet names").into_iter().filter(|sn| sn.to_lowercase() != "groupes vides").collect()
}
