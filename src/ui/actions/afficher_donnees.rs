use std::sync::Arc;

use crate::ui::AppState;



pub fn afficher_donnees(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	Ok(crate::ui::UpdateAction::Continue)
}