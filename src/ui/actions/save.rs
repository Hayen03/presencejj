use std::sync::Arc;

use crate::ui::AppState;

pub fn sauvegarder(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	Ok(crate::ui::UpdateAction::Continue)
}