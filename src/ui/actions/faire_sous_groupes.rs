use std::sync::Arc;

use ratatui::text::Text;

use crate::{cdj::groupes::{Groupe, GroupeID, NULL_GROUPE}, ui::{AppState, UIError, screens::{Desc, ProgressLogScreen}}};

pub fn faire_sous_groupes(state: Arc<AppState>) -> crate::ui::actions::ActionResult {

	let plans = {
		let groupes = state.groupes.read().expect("Poisoned Lock");
		groupes.groupes().filter_map(|grp| {
			if grp.id == NULL_GROUPE.id {
				None
			} else {
				let nb_sg = guess_nb_sous_groupes(grp);
				Some(SousGroupePlan {
					id: grp.id,
					desc: grp.short_desc(),
					nb_sg,
					participants: grp.participants.len(),
				})
			}
		}).collect::<Vec<SousGroupePlan>>()
	};
	let nb_groupes = plans.len();
	let screen = ProgressLogScreen::new("Création des sous-groupes".into(), nb_groupes as u32);
	let cancel_hook = screen.get_cancel_hook();
	let progress_hook = screen.get_progress_hook();
	let log_hook = screen.get_logger();
	let thread_handle = std::thread::spawn(move || {
		// poll the user for the missing nb_sg
		//let mut nb_sgs = HashMap::new();
		for plan in plans {
			// early stopping if cancel requested
			if *cancel_hook.lock().expect("Poisoned Lock") {
				log_hook.lock().expect("Poisoned Lock").log(Desc::Warning("Création des sous-groupes annulée".into()));
				return Err(UIError::CancelAction { desc: "La tâche a été annulée.".into() });
			}
			let nb = if let Some(nb_sg) = plan.nb_sg {
				Some(nb_sg)
			} else {
				log_hook.lock().expect("Poisoned Lock").log(Desc::Info(format!("Le nombre de sous-groupes pour le groupe '{}' est inconnu, demande à l'utilisateur...", plan.desc)));
				let poll = crate::ui::Poll {
					title: "Nombre de sous-groupes manquant".into(),
					prompt: Text::from(format!("Entrez le nombre de sous groupes pour le groupe {} (nombre de participant: {})", plan.desc, plan.participants)),
					validation: Some(Arc::new(|s| s.parse::<usize>().is_ok())),
					show_error: true,
				}.poll(state.clone());
				let nb = {
					match poll {
						Err(e) => {
							log_hook.lock().expect("Poisoned Lock").log(Desc::Error(format!("Erreur lors de la réception du nombre de sous-groupes pour le groupe '{}': {}", plan.desc, e)));
							None
						},
						Ok(None) => {
							log_hook.lock().expect("Poisoned Lock").log(Desc::Warning(format!("L'utilisateur a annulé la saisie du nombre de sous-groupes pour le groupe '{}'", plan.desc)));
							None
						},
						Ok(Some(s)) => {
							match s.parse::<usize>() {
								Ok(n) => Some(n),
								Err(e) => {
									log_hook.lock().expect("Poisoned Lock").log(Desc::Error(format!("L'utilisateur a entré une valeur invalide pour le nombre de sous-groupes du groupe '{}': {}", plan.desc, e)));
									None
								},
							}
						},
					}
				};
				if let Some(nb) = nb {
					log_hook.lock().expect("Poisoned Lock").log(Desc::Info(format!("Nombre de sous-groupes pour le groupe '{}' défini à {}", plan.desc, nb)));
					Some(nb)
				} else {
					log_hook.lock().expect("Poisoned Lock").log(Desc::Warning(format!("Nombre de sous-groupes pour le groupe '{}' non défini, ce groupe sera ignoré", plan.desc)));
					None
				}
			};

			// create the subgroups
			if let Some(nb) = nb {
				if nb == 0 {
					log_hook.lock().expect("Poisoned Lock").log(Desc::Warning(format!("Aucun sous-groupe à créer pour le groupe '{}'", plan.desc)));
				} else {
					let mut groupes = state.groupes.write().expect("Poisoned Lock");
					let membres = state.membres.read().expect("Poisoned Lock");
					match groupes.get_mut(plan.id).expect("Groupe Introuvable").mk_sous_groupes(nb, &membres) {
						Ok(_v) => {
							log_hook.lock().expect("Poisoned Lock").log(Desc::Info(format!("{} sous-groupes créés pour le groupe '{}'", nb, plan.desc)));
						},
						Err(_e) => {
							log_hook.lock().expect("Poisoned Lock").log(Desc::Error(format!("Erreur lors de la création des sous-groupes pour le groupe '{}'", plan.desc)));
						},
					}
				}
			}

			// increment progress
			*progress_hook.lock().expect("Poisoned Lock") += 1;
		}
		Ok(())
	});
	Ok(crate::ui::UpdateAction::Push(Box::new(screen.with_thread(thread_handle))).one())

}

fn guess_nb_sous_groupes(grp: &Groupe) -> Option<usize> {
    let cat = grp.category.as_ref().map(|s| s.trim().to_lowercase());
    match (cat.as_deref(), grp.estime_cap()) {
        (_, 0) => Some(0),
        (Some("crocus"), i) => { // crocus -> 10 par groupes
            Some((i as f32/10.0).ceil() as usize)
        },
		
        (Some("balaous"), i) => { // balaous -> 12 par groupes
            Some((i as f32/12.0).ceil() as usize)
        },
        (Some("basaltes"), i) => { // basaltes -> 15 par groupes
            Some((i as f32/15.0).ceil() as usize)
        },
        (Some("12-15 ans"), i) => {
            Some((i as f32/15.0).ceil() as usize)
        },
		
        (_c, _) => { // inconnu, on doit demander
            None
        },
    }
}

struct SousGroupePlan {
	pub id: GroupeID,
	pub desc: String,
	pub nb_sg: Option<usize>,
	pub participants: usize,
}