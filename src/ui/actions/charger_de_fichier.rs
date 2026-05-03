use std::sync::Arc;

use ratatui::text::Text;

use crate::ui::AppState;

pub fn charger_de_fichier(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let text = Text::from("Cette fonctionnalité n'est pas encore implémentée, mais elle le sera bientôt !");
	let title = "Attention!".into();
	let size = {
		let lock = state.theme.read().expect("Poisoned Lock");
		(lock.info_box_max_width, lock.info_box_max_height)
	};
	let screen = crate::ui::screens::InfoScreen::new(title, text)
		.with_size(size.0, size.1);

	Ok(crate::ui::UpdateAction::PushSub(Box::new(screen)).one())
}