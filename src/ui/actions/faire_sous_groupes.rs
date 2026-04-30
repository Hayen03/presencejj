use std::sync::Arc;

use crate::ui::AppState;

pub fn faire_sous_groupes(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	Ok(crate::ui::UpdateAction::Continue)
}