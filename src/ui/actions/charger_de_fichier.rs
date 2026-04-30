use std::sync::Arc;

use crate::ui::AppState;

pub fn charger_de_fichier(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	Ok(crate::ui::UpdateAction::Continue)
}