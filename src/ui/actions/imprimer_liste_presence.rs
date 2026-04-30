use std::sync::Arc;

use crate::ui::AppState;

pub fn imprimer_liste_presence(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	Ok(crate::ui::UpdateAction::Continue)
}