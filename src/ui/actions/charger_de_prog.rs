use std::sync::Arc;

use crate::{extract::{ExtractError, excel::into_string, prog::extract_group_info_from_prog}, ui::{AppState, UIError, UpdateAction, screens::{Desc}}};

pub fn charger_de_prog(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let filepath = state.get_in_xlsx("Sélectionnez le fichier de programmation");
	if let Some(filepath) = filepath {
		// update the old_in_dir to the parent of the selected file
		if let Some(parent) = filepath.parent() {
			let mut lock = state.old_in_dir.write().expect("Poisoned Lock");
			*lock = parent.to_path_buf();
		}

		let mut workbook = match office::Excel::open(filepath) {
			Ok(wb) => wb,
			Err(e) => return Ok(UpdateAction::ErrorPopUp(Box::new(UIError::from(e))).one()),
		};
		let rng = match workbook.worksheet_range("Groupes") {
			Ok(r) => r,
			Err(e) => return Ok(UpdateAction::ErrorPopUp(Box::new(UIError::from(e))).one()),
		};
		let l = rng.get_size().0.max(2) - 2; // there's two lines before the data starts, and we want to count the number of lines of data, not the total number of lines. Usage of max() is to avoid underflow in case there are less than 2 lines in the sheet, which would result in a negative number of lines of data.
		if l == 0 || rng.get_size().1 < 2 {
			return Ok(UpdateAction::ErrorPopUp(ExtractError::InvalidFormat.into()).one());
		}
		let saison = into_string(&rng.rows().next().unwrap()[0]);
		let config = crate::extract::prog::ProgLnConfig::guess(rng.rows().nth(1).unwrap());

		let screen = crate::ui::screens::ProgressLogScreen::new(" Chargement des listes de présence ".into(), l as u32);
		let progress_hook = screen.get_progress_hook();
		let cancel_hook = screen.get_cancel_hook();
		let log_hook = screen.get_logger();

		let thread_handle: std::thread::JoinHandle<Result<(), UIError>> = std::thread::spawn(move || {
			let mut flag = false;
			for (i, row) in rng.rows().skip(2).enumerate() {
				flag = true;
				 // early check for cancel
				if *cancel_hook.lock().expect("Poisoned Lock") {
					log_hook.lock().expect("Poisoned Lock").log(Desc::Warning("Importation annulée".into()));
					return Err(UIError::CancelAction { desc: "Importation annulée par l'utilisateur".into() });
				}

				log_hook.lock().expect("Poisoned Lock").log(Desc::Info(format!("Lecture du groupe {}", row.get(1).map(|s| into_string(s).unwrap_or("???".into())).unwrap_or("???".into()))));

				match extract_group_info_from_prog(row, &config, saison.as_deref()) {
					Ok(grp) => {
						// fix the id and add the group to the registry
						let mut groupes = state.groupes.write().expect("Poisoned Lock");
						let cap = grp.capacite;

						// Voir si le groupe existe déjà
						let existing_grp = groupes.groupes().filter(|g| g.equiv(&grp)).map(|g| g.id).collect::<Vec<_>>();
						let gid = if existing_grp.is_empty() {
							// Si non, rajouter le groupe
							let id = groupes.get_new_id_from_seed(grp.id.0);
							let mut grp = grp;
							grp.id = id;
							let _ = groupes.add(grp);
							log_hook.lock().expect("Poisoned Lock").log(Desc::Info(format!("Groupe ajouté avec l'ID {}", id)));
							id
						} else {
							existing_grp[0]
						};

						let groupe = groupes.get_mut(gid).unwrap();
						// mettre à jour certaines données
						if !existing_grp.is_empty() {
							groupe.capacite = match cap {
								None => groupe.capacite,
								Some(cap) => Some(cap),
							}
						}
					},
					Err(e) => {
						log_hook.lock().expect("Poisoned Lock").log(Desc::Error(format!("Erreur lors de l'extraction des données du groupe '{}': {}", row.first().map(|s| into_string(s).unwrap_or("???".into())).unwrap_or("???".into()), e)));
					},
				}
				
				// increment progress
				*progress_hook.lock().expect("Poisoned Lock") = i as u32;
				//std::thread::sleep(std::time::Duration::from_millis(500)); // simulate some work
			}

			//log_hook.lock().expect("Poisoned Lock").log(Desc::Info("Terminé".into()));
			log_hook.lock().expect("Poisoned Lock").log(Desc::Info(format!("Terminé {}", if flag { "" } else { " (aucune donnée trouvée)" })));
			Ok(())
		});
		let screen = screen.with_thread(thread_handle);

		return Ok(UpdateAction::Push(Box::new(screen) as Box<dyn crate::ui::Screen>).one());
	}

	Ok(crate::ui::UpdateAction::ErrorPopUp(Box::new(UIError::CancelAction { desc: String::from("Aucun fichier sélectionné") } )).one())
}