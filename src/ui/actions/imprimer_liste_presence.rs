use std::{collections::HashSet, sync::Arc};

use crate::{cdj::groupes::NULL_GROUPE, print::typst::{print_presence_anim, print_presence_sdj}, ui::{AppState, UIError, screens::Desc}};

pub fn imprimer_liste_presence(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let out_dir = state.get_out_dir("Choisissez le dossier de sortie");

	let n_anim = state.groupes.read().expect("Poisoned Lock").groupes().filter(|g| g.id != NULL_GROUPE.id).count() as u32;
	let n_sdg = {
		let groupes = state.groupes.read().expect("Poisoned Lock");
		let mut grp_info = HashSet::new();
		for grp in groupes.groupes().filter_map(|g| if g.id == NULL_GROUPE.id {None} else {Some(g.get_sdj_info())}) {
			grp_info.insert(grp);
		}
		grp_info.len() as u32
	};
	let screen = crate::ui::screens::ProgressLogScreen::new("Impression des listes de présence".into(), n_anim + n_sdg);
	let progress_hook = screen.get_progress_hook();
	let cancel_hook = screen.get_cancel_hook();
	let logger = screen.get_logger();
	let logger = move |msg: Desc| {
		logger.lock().expect("Poisoned Lock").log(msg);
	};
	let thread_handle = std::thread::spawn(move || {
		let out_dir = out_dir.as_ref().and_then(|p| p.to_str());

		// FICHE ANIM
		{ // block to auto release the lock afterwards
			let groupes = state.groupes.read().expect("Poisoned Lock");
			for groupe in groupes.groupes().filter(|g| g.id != NULL_GROUPE.id) {
				// check for early cancellation
				if *cancel_hook.lock().expect("Poisoned Lock") {
					return Err(UIError::CancelAction { desc: "Tâche annulée".into() });
				}

				let membres = state.membres.read().expect("Poisoned Lock");
				let comptes = state.comptes.read().expect("Poisoned Lock");
				let config = state.config.read().expect("Poisoned Lock");
				if groupe.sous_groupe.is_empty() {
					if let Err(err) = print_presence_anim(groupe, None, &membres, &comptes, &config, out_dir, &logger) {
						logger(Desc::Error(format!("Erreur lors de l'impression du groupe {}: {}", groupe.desc(), err)));
					}
				} else {
					for sg in &groupe.sous_groupe {
						if let Err(err) = print_presence_anim(groupe, Some(sg), &membres, &comptes, &config, out_dir, &logger) {
							logger(Desc::Error(format!("Erreur lors de l'impression du groupe {} - {}: {}", groupe.desc(), sg.disc, err)));
						}
					}
				}

				// increment progress
				*progress_hook.lock().expect("Poisoned Lock") += 1;
			}
		}
		// FICHE SDG
		{
			let groupes = state.groupes.read().expect("Poisoned Lock");
			let mut grp_info = HashSet::new();
			for grp in groupes.groupes().filter_map(|g| if g.id == NULL_GROUPE.id {None} else {Some(g.get_sdj_info())}) {
				grp_info.insert(grp);
			}
			for grp in grp_info {
				// check for early cancellation
				if *cancel_hook.lock().expect("Poisoned Lock") {
					return Err(UIError::CancelAction { desc: "Tâche annulée".into() });
				}

				let membres = state.membres.read().expect("Poisoned Lock");
				let comptes = state.comptes.read().expect("Poisoned Lock");
				let config = state.config.read().expect("Poisoned Lock");
				if let Err(err) = print_presence_sdj(&grp, &groupes, &membres, &comptes, &config, out_dir, &logger) {
					logger(Desc::Error(format!("Erreur lors de l'impression du SDG {} - {} - {}: {}", grp.saison.unwrap_or("none"), grp.site.unwrap_or("none"), grp.semaine.unwrap_or("none"), err)));
				}

				// increment progress
				*progress_hook.lock().expect("Poisoned Lock") += 1;
			}
		}
		Ok(())
	});

	Ok(crate::ui::UpdateAction::Push(Box::new(screen.with_thread(thread_handle))).one())
}