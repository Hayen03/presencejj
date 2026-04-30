use std::{fmt::Debug, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{Frame, Terminal, buffer::Buffer, layout::Rect, prelude::CrosstermBackend, style::{Color, Style, Stylize}, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, StatefulWidgetRef, Widget, WidgetRef}};

use crate::ui::{AppState, Screen, Theme, UIError, UpdateAction, event::{self, Event}, screens::ErrorScreen, tui::Tui};
use crate::ui::actions;

pub struct MenuItem<Ids> where Ids: ToString + Debug {
	id: Ids,
	action: Box<actions::Action>,
}
impl<Ids> Debug for MenuItem<Ids> where Ids: ToString + Debug {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MenuItem")
			.field("id", &self.id.to_string())
			.finish()
	}
}

#[derive(Debug)]
struct Menu<'a, Ids> where Ids: ToString + Debug {
	items: Box<[MenuItem<Ids>]>,
	selected: usize,
	title: String,
	widget: ratatui::widgets::List<'a>,
	_block: Block<'a>,
}
impl<'a, Ids> Menu<'a, Ids> where Ids: ToString + Debug {
	pub fn new(items: Box<[MenuItem<Ids>]>) -> Self {
		let block = Block::bordered()
			.title(" Menu ")
			.border_set(border::THICK);
		let widget = ratatui::widgets::List::new(items.iter().map(|item| Line::from(item.id.to_string())))
			.style(Style::new().white())
			.highlight_style(Style::new().yellow().on_dark_gray())
			.block(block.clone());
		Menu { items, selected: 0, title: String::new(), widget, _block: block }
	}
	pub fn with_title(mut self, title: String) -> Self {
		self.title = title;
		self
	}
	pub fn get_selected(&self) -> Option<&MenuItem<Ids>> {
		if self.items.is_empty() {
			None
		} else {
			Some(&self.items[self.selected])
		}
	}
	pub fn select(&mut self, index: usize) -> bool {
		if index < self.items.len() {
			self.selected = index;
			true
		} else {
			false
		}
	}
	pub fn next(&mut self) {
		if !self.items.is_empty() {
			self.selected = (self.selected + 1) % self.items.len();
		}
	}
	pub fn previous(&mut self) {
		if !self.items.is_empty() {
			self.selected = (self.selected + self.items.len() - 1) % self.items.len();
		}
	}
	pub fn block(mut self, block: Block<'a>) -> Self {
		self._block = block;
		self.widget = self.widget.block(self._block.clone());
		self
	}

	fn handle_key(&mut self, event: KeyEvent, state: Arc<AppState>) -> Result<UpdateAction, UIError> {
		match event.code {
			KeyCode::Up => {
				self.previous();
				Ok(UpdateAction::Continue)
			},
			KeyCode::Down => {
				self.next();
				Ok(UpdateAction::Continue)
			},
			KeyCode::Esc => Ok(UpdateAction::Quit),
			KeyCode::Enter => {
				if let Some(item) = self.get_selected() {
					let result = (item.action)(state)?;
					Ok(result)
				} else {
					Ok(UpdateAction::Continue)
				}
			},
			_ => Ok(UpdateAction::Continue),
		}
	}
}
impl<'a, Ids> WidgetRef for Menu<'a, Ids> where Ids: ToString + Debug {
	fn render_ref(&self, area: Rect, buf: &mut Buffer)
		where
			Self: Sized {
		let mut state = ratatui::widgets::ListState::default().with_selected(Some(self.selected));
		ratatui::widgets::StatefulWidget::render(&self.widget, area, buf, &mut state);
	}
}
impl<'a, Ids> Screen for Menu<'a, Ids> where Ids: ToString + Debug {
	fn handle_event(&mut self, event: event::Event, state: Arc<AppState>) -> Result<UpdateAction, UIError> {
		match event  {
			event::Event::Key(ke) => self.handle_key(ke, state),
			_ => Ok(UpdateAction::Continue),
		}
	}
	fn render_focus(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, focus: bool) {
		let mut state = ratatui::widgets::ListState::default().with_selected(Some(self.selected));
		let widget = if focus {
			self.widget.clone()
		} else {
			self.widget.clone()
				.style(Style::new().gray())
				.highlight_style(Style::new().gray().on_dark_gray())
				.block(self._block.clone()
					.border_style(Style::new().gray()))
		};
		ratatui::widgets::StatefulWidget::render(&widget, area, buf, &mut state);
	}
}

pub struct App {
	should_quit: bool,
	theme: &'static Theme,
	main_menu: Menu<'static, actions::MainActions>,
	stack: Vec<Box<dyn Screen>>, // Screen that appears in the zone to the left of the menu
	sub_screen_stack: Vec<Box<dyn Screen>>, // Screen that appears on top of everything else, used for popups and such
	state: Arc<AppState>,
}
impl Default for App {
	fn default() -> Self {
		let main_menu: Menu<actions::MainActions> = Menu::new(Box::new([
			MenuItem { id: actions::MainActions::ChargerDeFichier, action: Box::new(actions::charger_de_fichier) },
			MenuItem { id: actions::MainActions::ChargerDePresence, action: Box::new(actions::charger_de_presence) },
			MenuItem { id: actions::MainActions::ChargerDeProg, action: Box::new(actions::charger_de_prog) },

			MenuItem { id: actions::MainActions::AfficherDonnees, action: Box::new(actions::afficher_donnees) },
			MenuItem { id: actions::MainActions::FaireSousGroupes, action: Box::new(actions::faire_sous_groupes) },
			MenuItem { id: actions::MainActions::EstimerChandails, action: Box::new(actions::estimer_chandail) },
			
			MenuItem { id: actions::MainActions::ImprimerListesPresence, action: Box::new(actions::imprimer_liste_presence) },
			MenuItem { id: actions::MainActions::ImprimerFichesSante, action: Box::new(actions::imprimer_fiche_sante) },
			MenuItem { id: actions::MainActions::ImprimerStats, action: Box::new(actions::imprimer_stats) },

			MenuItem { id: actions::MainActions::Sauvegarder, action: Box::new(actions::sauvegarder) },
			MenuItem { id: actions::MainActions::Quitter, action: Box::new(actions::quit) },
		])).with_title("Menu Principal".into());
		App {
			should_quit: false,
			theme: &Theme::DARK,
			main_menu,
			stack: vec![],
			sub_screen_stack: vec![],
			state: Arc::new(AppState::default()),
		}
	}
}
impl App {
	pub fn run(mut self, terminal: &mut Tui) -> Result<(), UIError> {
		loop {
			terminal.draw(&self)?;
			match self.update(terminal.events.next()?) {
				Ok(cont) => {
					if !cont || self.should_quit {
						return Ok(());
					}
				},
				Err(err) => {
					return Err(err);
				}
			}
		}
	}

	fn get_focused_screen_mut(&mut self) -> &mut dyn Screen {
		match (self.stack.last_mut(), self.sub_screen_stack.last_mut()) {
			(_, Some(sub_screen)) => sub_screen.as_mut(),
			(Some(screen), None) => screen.as_mut(),
			(None, None) => &mut self.main_menu,
		}
	}
	fn update_current_screen(&mut self, event: Event) -> Result<UpdateAction, UIError> {
		let current_screen = match (self.stack.last_mut(), self.sub_screen_stack.last_mut()) {
			(_, Some(sub_screen)) => sub_screen.as_mut(),
			(Some(screen), None) => screen.as_mut(),
			(None, None) => &mut self.main_menu,
		};
		current_screen.handle_event(event, self.state.clone())
	}
	fn get_focused_screen(&self) -> &dyn Screen {
		match (self.stack.last(), self.sub_screen_stack.last()) {
			(_, Some(sub_screen)) => sub_screen.as_ref(),
			(Some(screen), None) => screen.as_ref(),
			(None, None) => &self.main_menu,
		}
	}

/**
* update fn for the main loop. Returns None if the app should continue, or Some(Result<(), UIError>) if the app is done and should return the result.
*/
	fn update(&mut self, event: Event) -> Result<bool, UIError> {
		let event_result = self.update_current_screen(event)?;
		match event_result {
			UpdateAction::Continue => Ok(true),
			UpdateAction::Quit => Ok(false),
			UpdateAction::Pop => {
				if self.sub_screen_stack.pop().is_none() {
					return Ok(self.stack.pop().is_some()); // if we pop while nothing to pop, it means we quit the main menu, so we return false to quit the app
				}
				Ok(true)
			},
			UpdateAction::Push(screen) => {
				self.stack.push(screen);
				Ok(true)
			},
			UpdateAction::PushSub(screen) => {
				self.sub_screen_stack.push(screen);
				Ok(true)
			},
			UpdateAction::Replace(screen) => {
				self.stack.pop();
				self.stack.push(screen);
				Ok(true)
			},
			UpdateAction::ReplaceSub(screen) => {
				self.sub_screen_stack.pop();
				self.sub_screen_stack.push(screen);
				Ok(true)
			},
			UpdateAction::ErrorPopUp(err) => {
				self.sub_screen_stack.push(Box::new(ErrorScreen::from_error(err)));
				Ok(true)
			},
			UpdateAction::ErrorReplace(err) => {
				self.sub_screen_stack.pop();
				self.sub_screen_stack.push(Box::new(ErrorScreen::from_error(err)));
				Ok(true)
			},
		}
	}

	pub fn quit(&mut self) {
		self.should_quit = true;
	}
}

impl Widget for &App {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let title = Line::from(" Présence JJ ");
		let block = Block::bordered()
			.title(title.centered())
			.border_set(border::THICK)
			.bg(Color::Black);
		let inner = block.inner(area);
		block.render(area, buf);
		if inner.width < self.theme.app_min_width {
			// get the focused screen and render it in the whole area, without rendering the menu
			let focus_on = self.get_focused_screen();
			focus_on.render_focus(inner, buf, true);
		} else {
			let menu_area = Rect{
				x: inner.x, 
				y: inner.y,
				width: self.theme.main_menu_width,
				height: inner.height,
			};
			let screen_area = Rect {
				x: inner.x + self.theme.main_menu_width,
				y: inner.y,
				width: inner.width - self.theme.main_menu_width,
				height: inner.height,
			};
			// find what screen has focus. we'll need to compare pointers
			let focus_on = self.get_focused_screen() as *const dyn Screen;
			// render the menu and the screen, if there is one
			self.main_menu.render_focus(menu_area, buf, std::ptr::addr_eq(focus_on, &self.main_menu as *const dyn Screen));
			if let Some(screen) = self.stack.last() {
				screen.render_focus(screen_area, buf, std::ptr::addr_eq(focus_on, screen.as_ref() as *const dyn Screen));
			} else {
				// render an empty box
				Block::bordered()
					.border_set(border::THICK)
					.render(screen_area, buf);
			}
			// render the sub screen on top of everything else, if there is one
			for sub_screen in self.sub_screen_stack.iter() {
				// the subscreen is given the full inner area, but it can choose to render itself in a smaller area if it wants to
				sub_screen.render_focus(inner, buf, std::ptr::addr_eq(focus_on, sub_screen.as_ref() as *const dyn Screen));
			}
		}
		let screen_area = Rect {
			x: area.x + 1,
			y: area.y + 1,
			width: area.width.saturating_sub(2),
			height: area.height.saturating_sub(2),
		};
		if let Some(screen) = self.stack.last() {
			screen.render_ref(screen_area, buf);
		}
	}
}