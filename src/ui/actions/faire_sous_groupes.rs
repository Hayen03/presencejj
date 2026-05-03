use std::sync::Arc;

use ratatui::text::Text;

use crate::{groupes::groupes::{Groupe, GroupeID, NULL_GROUPE}, prelude::print_option, ui::{AppState, TextInputError, screens::{Desc, ProgressLogScreen}}};

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
}