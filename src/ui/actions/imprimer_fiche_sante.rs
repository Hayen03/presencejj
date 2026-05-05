use std::{collections::{HashMap, HashSet}, sync::Arc};


use crate::{cdj::{comptes::NULL_COMPTE, groupes::NULL_GROUPE, membres::{MembreID, NULL_MEMBRE}}, print::typst::print_fiche_med, ui::{AppState, UIError, screens::{Desc, ProgressLogScreen}}};

pub fn imprimer_fiche_sante(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let out_dir = state.get_out_dir("Sélectionnez le dossier de sortie");
	if let Some(out_dir) = out_dir {
		let target = state.membres.read().expect("Poisoned Lock").membres().filter(|m| m.id != NULL_MEMBRE.id).count() as u32;
		let screen = ProgressLogScreen::new("Impression des fiches santés".into(), target);
		let progress_hook = screen.get_progress_hook();
		let cancel_hook = screen.get_cancel_hook();
		let log_hook = screen.get_logger();
		let logger = move |msg: Desc| { log_hook.lock().expect("Poisoned Lock").log(msg); };
		let thread_handle: std::thread::JoinHandle<Result<(), UIError>> = std::thread::spawn(move || {
			// Faire le plan: quel membre sont sur quel site, pour faire les lots d'impression par site
			let mut site_mbrs: HashMap<MembreID, HashSet<(&str, &str)>> = HashMap::new();
			let groupes = state.groupes.read().expect("Poisoned Lock");
			for grp in groupes.groupes().filter(|g| g.id != NULL_GROUPE.id) {
				let site = grp.get_site().unwrap_or("None");
				let saison = grp.get_saison().unwrap_or("None");
				for participant in grp.participants.iter() {
					site_mbrs.entry(*participant).or_default().insert((saison, site));
				}
			}
			// imprimer les fiches santés
			for (mid, _sites) in site_mbrs {
				// check for early cancel
				if *cancel_hook.lock().expect("Poisoned Lock") {
					return Err(UIError::CancelAction { desc: "La tâche a été annulée.".into() });
				}
				{ // block to auto drop the locks after usage
					let disc = _sites.into_iter().collect::<Vec<(&str, &str)>>();
					let membres = state.membres.read().expect("Poisoned Lock");
					let comptes = state.comptes.read().expect("Poisoned Lock");
					let config = state.config.read().expect("Poisoned Lock");
					if let Ok(membre) = membres.get(mid) {
						let compte = comptes.get(membre.compte.unwrap_or_default()).unwrap_or(&NULL_COMPTE);
						let _res = print_fiche_med(membre, compte, &config, &disc, false, out_dir.to_str(), &logger);
						if let Err(err) = _res {
							logger(Desc::Error(format!("Erreur lors de l'impression de la fiche santé pour {mid}: {err}")));
						}
					} else {
						logger(Desc::Error(format!("Membre {mid} inexistant")));
					}
				}
				// incrementer la barre de progression
				*progress_hook.lock().expect("Poisoned Lock") += 1;
			}
			logger(Desc::Info("Impression terminée".into()));
			Ok(())
		});

		Ok(crate::ui::UpdateAction::Push(Box::new(screen)).one())
	} else {
		Ok(crate::ui::UpdateAction::ErrorPopUp(Box::new(UIError::CancelAction { desc: String::from("Aucun fichier sélectionné") } )).one())
	}
}