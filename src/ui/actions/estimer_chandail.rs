use std::sync::Arc;

use ratatui::text::Text;

use crate::ui::{AppState, UIError, UpdateAction, actions::UpdateActions, screens::LineInputScreen};

pub fn estimer_chandail(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let text = Text::from("Cette fonctionnalité n'est pas encore implémentée, mais elle le sera bientôt !");
	let title = "Attention!".into();
	let size = {
		let lock = state.theme.read().expect("Poisoned Lock");
		(lock.info_box_max_width, lock.info_box_max_height)
	};
	let screen = crate::ui::screens::InfoScreen::new(title, text, size);

	let validation = Box::new(|s: &str| s.parse::<u32>().is_ok());
	let after = |s: Option<&str>, state: Arc<AppState>| -> Result<UpdateActions, UIError> {
		let s = match s {
			Some(s) => s,
			None => return Ok(UpdateAction::ErrorReplace(Box::new(UIError::msg("Aucun nombre entré"))).one())
		};
		let n = match s.parse::<u32>() {
			Ok(n) => n,
			Err(_) => return Ok(UpdateAction::ErrorPopUp(Box::new(UIError::msg("Nombre Invalide"))).one())
		};
		let msg = format!("Le nombre entré est {n}");
		let (width, height) = {
			let lock = state.theme.read().expect("Poisoned Lock");
			(lock.info_box_max_width, lock.info_box_max_height)
		};
		let new_screen = crate::ui::screens::InfoScreen::new("Résultat".into(), Text::from(msg), (width, height));
		Ok(crate::ui::UpdateAction::ReplaceSub(Box::new(new_screen)).one())
	};
	let input_screen = LineInputScreen::default()
		.with_after(Box::new(after))
		.with_validation(validation)
		.with_message(Text::from("Entrez un nombre"))
		.with_title("Test".into())
		.with_placeholder("nombre entier".into());

	Ok(crate::ui::UpdateAction::PushSub(Box::new(input_screen)).one())
}

