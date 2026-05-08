use std::sync::Arc;

use crate::ui::AppState;

pub fn afficher_donnees(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let screen = crate::ui::screens::ViewTable::from_regs(
		&state.groupes.read().expect("Poisoned Lock"),
		&state.membres.read().expect("Poisoned Lock"),
		&state.comptes.read().expect("Poisoned Lock"),
	);

	Ok(crate::ui::UpdateAction::Push(Box::new(screen)).one())
}