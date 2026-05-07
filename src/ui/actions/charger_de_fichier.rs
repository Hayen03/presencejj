use std::sync::Arc;

use lazy_static::lazy_static;
use ratatui::{style::Stylize, text::{Line, Text}};

use crate::ui::{AppState, Screen, UIError, screens::{TaskScreen, TextScreen}, serial::LoadData};

lazy_static!{
	pub static ref LOAD_SCREEN_TITLE: Line<'static> = Line::from(" Chargement ").centered().white().bold();
	pub static ref LOAD_WORK_TEXT: Text<'static> = Text::from("Chargement en cours, veuillez patienter...").yellow();
	pub static ref LOAD_ERROR_LINE: Line<'static> = Line::from("Une erreur est survenue lors du chargement.").red();
	pub static ref LOAD_DONE_TEXT: Text<'static> = Text::from("Chargement terminé!").green();
}

pub fn charger_de_fichier(state: Arc<AppState>) -> crate::ui::actions::ActionResult {
	let initial_screen = TextScreen::new(LOAD_SCREEN_TITLE.clone(), LOAD_WORK_TEXT.clone());
	let after = move |val: Option<Result<(), UIError>>| -> Option<Box<dyn Screen>> {
		match val {
			None => None,
			Some(Ok(())) => Some(Box::new(TextScreen::new(LOAD_SCREEN_TITLE.clone(), LOAD_DONE_TEXT.clone()))),
			Some(Err(e)) => {
				let text = Text::from(vec![
					LOAD_ERROR_LINE.clone(),
					Line::from(e.to_string()),
				]);
				Some(Box::new(TextScreen::new(LOAD_SCREEN_TITLE.clone(), text)))
			},
		}
	};
	let work_thread = std::thread::spawn(move || {
		let file = state.get_in_pres("Choisissez le fichier");
		if let Some(file) = file {
			let bytes = std::fs::read(file);
			match bytes {
				Ok(bytes) => {
					let load_state = postcard::from_bytes::<LoadData>(&bytes);
					match load_state {
						Ok(load_state) => {
							// put the data in state
							{
								// overwrite or update? for now, just overwrite
								*state.comptes.write().expect("Poisoned Lock") = load_state.comptes;
								*state.membres.write().expect("Poisoned Lock") = load_state.membres;
								*state.groupes.write().expect("Poisoned Lock") = load_state.groupes;
							}
							Ok(())
						},
						Err(e) => {
							Err(UIError::Others { src: Box::new(e) })
						},
					}
				},
				Err(e) => {
					Err(UIError::IO { src: e })
				},
			}
		} else {
			Err(UIError::UnexpectedState { desc: "No input file selected".to_string() })
		}
	});
	let screen = TaskScreen::new(Box::new(initial_screen), work_thread, Box::new(after));

	Ok(crate::ui::UpdateAction::Push(Box::new(screen)).one())
}