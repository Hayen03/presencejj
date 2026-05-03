use std::sync::Arc;

use crate::ui::{AppState, actions::ActionResult};

pub fn quit(state: Arc<AppState>) -> ActionResult {
	Ok(crate::ui::UpdateAction::Quit.one())
}