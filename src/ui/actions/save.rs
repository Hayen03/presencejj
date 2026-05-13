use std::sync::Arc;

use lazy_static::lazy_static;
use ratatui::{style::Stylize, text::{Line, Text}};

use crate::ui::{AppState, FilePoll, UIError, screens::{TaskScreen, TextScreen}, serial::SaveData};

lazy_static!{
	pub static ref SAVE_SCREEN_TITLE: Line<'static> = Line::from(" Sauvegarde ").centered().white().bold();
	pub static ref SAVE_WORK_TEXT: Text<'static> = Text::from("Sauvegarde en cours, veuillez patienter...").yellow();
	pub static ref SAVE_DONE_TEXT: Text<'static> = Text::from("Sauvegarde terminée!").green();
	pub static ref SAVE_ERROR_LINE: Line<'static> = Line::from("Une erreur est survenue lors de la sauvegarde.").red();
}

pub fn sauvegarder(state: Arc<AppState>) -> crate::ui::actions::ActionResult {

	let initial_screen = TextScreen::new(SAVE_SCREEN_TITLE.clone(), SAVE_WORK_TEXT.clone());
	let after = move |val: Option<Result<(), UIError>>| -> Option<Box<dyn crate::ui::Screen>> {
		match val {
			None => None,
			Some(Ok(())) => Some(Box::new(TextScreen::new(SAVE_SCREEN_TITLE.clone(), SAVE_DONE_TEXT.clone()))),
			Some(Err(e)) => {
				let text = Text::from(vec![
					SAVE_ERROR_LINE.clone(),
					Line::from(e.to_string()),
				]);
				Some(Box::new(TextScreen::new(SAVE_SCREEN_TITLE.clone(), text)))
			},
		}
	};
	let work_thread = std::thread::spawn(move || {
		let out = FilePoll::save("Sélectionnez le fichier de sortie".into())
			.with_filter("pres", &["pres"])
			.poll(state.clone());
		let out = if let Some(out) = out {
			out
		} else {
			return Err(UIError::CancelAction { desc: String::from("Aucun fichier sélectionné") });
		};

		let comptes = state.comptes.read().expect("Poisoned Lock");
		let membres = state.membres.read().expect("Poisoned Lock");
		let groupes = state.groupes.read().expect("Poisoned Lock");
		let save_state = SaveData {comptes: &comptes, membres: &membres, groupes: &groupes };
		let bytes = postcard::to_allocvec(&save_state);
		match bytes {
			Ok(bytes) => {
				// save to file
				if let Err(e) = std::fs::write(out, bytes) {
					Err(UIError::IO { src: e })
				} else {
					Ok(())
				}
			},
			Err(e) => {
				Err(UIError::Others { src: Box::new(e) })
			},
		}
	});
	let screen = TaskScreen::new(Box::new(initial_screen), work_thread, Box::new(after));

	Ok(crate::ui::UpdateAction::Push(Box::new(screen)).one())
}
