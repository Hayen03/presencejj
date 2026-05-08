use lazy_static::lazy_static;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, WidgetRef};

use crate::cdj::groupes::GroupeReg;
use crate::cdj::{comptes::CompteReg, membres::MembreReg};
use crate::ui::{Screen, UpdateAction};
use crate::ui::screens::{groupe_table::GroupeTable, membre_table::MembreTable, compte_table::CompteTable};

lazy_static! {
	pub static ref VIEW_TABLE_TITLE: Line<'static> = Line::default(); // placeholder, not used
	pub static ref VIEW_TABLE_INSTRUCTIONS: Line<'static> = Line::from(vec![
		" Utilisez ".gray(),
		"Tab".light_blue().bold(),
		" pour changer de table, ".gray(),
		"Entrée".light_blue().bold(),
		" pour voir les détails, ".gray(),
		"Esc".light_blue().bold(),
		" pour revenir en arrière ".gray(),
	]).centered();
	pub static ref VIEW_TABLE_BLOCK: Block<'static> = Block::bordered()
		.title_top(VIEW_TABLE_TITLE.clone())
		.title_bottom(VIEW_TABLE_INSTRUCTIONS.clone())
		.border_style(Style::new().white())
		.border_set(border::THICK)
		.bg(Color::Black);
	pub static ref VIEW_TABLE_SELECTION_STYLE: Style = Style::new().yellow().bold().on_gray();
	pub static ref VIEW_TABLE_UNSELECTED_STYLE: Style = Style::new().white();
	pub static ref VIEW_TABLE_HEADER_BLOCK: Block<'static> = Block::default()
		.borders(Borders::BOTTOM)
		.border_set(border::PLAIN)
		.border_style(Style::new().gray());
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Selection {
	#[default]
	Groupes,
	Membres,
	Comptes,
}
impl Selection {
	fn as_str(&self) -> &'static str {
		match self {
			Selection::Groupes => "Groupes",
			Selection::Membres => "Membres",
			Selection::Comptes => "Comptes",
		}
	}
	#[allow(dead_code)]
	fn all() -> [Selection; 3] {
		[Selection::Groupes, Selection::Membres, Selection::Comptes]
	}
	fn next(self) -> Self {
		match self {
			Selection::Groupes => Selection::Membres,
			Selection::Membres => Selection::Comptes,
			Selection::Comptes => Selection::Groupes,
		}
	}
}

#[derive(Debug, Default)]
pub struct ViewTable {
	selection: Selection,
	pub groupes: GroupeTable,
	pub membres: MembreTable,
	pub comptes: CompteTable,
}
impl ViewTable {
	pub fn from_regs(groupes: &GroupeReg, membres: &MembreReg, comptes: &CompteReg) -> Self {
		let mut gt = GroupeTable::default();
		gt.update(groupes);
		gt.fit_widths();
		let mut mt = MembreTable::default();
		mt.update(membres);
		mt.fit_widths();
		let mut ct = CompteTable::default();
		ct.update(comptes);
		ct.fit_widths();
		ViewTable {
			selection: Selection::Groupes,
			groupes: gt,
			membres: mt,
			comptes: ct,
		}
	}
}
impl WidgetRef for ViewTable {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		Clear.render(area, buf);

		// render the block (border + title + instructions)
		VIEW_TABLE_BLOCK.clone().render(area, buf);
		let inner = VIEW_TABLE_BLOCK.inner(area);

		// create the header
		let header = Line::from(vec![
			stylize_selection(self.selection, Selection::Groupes),
			" | ".gray(),
			stylize_selection(self.selection, Selection::Membres),
			" | ".gray(),
			stylize_selection(self.selection, Selection::Comptes),
		]);
		let header_area = Rect {
			x: inner.x,
			y: inner.y,
			width: inner.width,
			height: 2,
		};
		Paragraph::new(header)
			.block(VIEW_TABLE_HEADER_BLOCK.clone())
			.centered()
			.render(header_area, buf);

		// render the selected table
		let table_area = Rect {
			x: inner.x,
			y: inner.y + 2, // header + separator
			width: inner.width,
			height: inner.height.saturating_sub(2),
		};
		match self.selection {
			Selection::Groupes => self.groupes.render_ref(table_area, buf),
			Selection::Membres => self.membres.render_ref(table_area, buf),
			Selection::Comptes => self.comptes.render_ref(table_area, buf),
		}
	}
}
impl Screen for ViewTable {
	fn handle_event(&mut self, event: crate::ui::event::Event, state: std::sync::Arc<crate::ui::AppState>) -> Result<crate::ui::actions::UpdateActions, crate::ui::UIError> {
		match event {
			crate::ui::event::Event::Key(key) => {
				use crossterm::event as cte;
				match key.code {
					cte::KeyCode::Tab => {
						self.selection = self.selection.next();
						Ok(UpdateAction::Continue.one())
					},
					cte::KeyCode::Esc => {
						Ok(UpdateAction::Pop.one())
					},
					_ => {
						// pass the event to the selected table
						match self.selection {
							Selection::Groupes => self.groupes.handle_event(event, state.clone()),
							Selection::Membres => self.membres.handle_event(event, state.clone()),
							Selection::Comptes => self.comptes.handle_event(event, state.clone()),
						}
					},
				}
			},
			_ => Ok(UpdateAction::Continue.one()),
		}
	}
}

fn stylize_selection(current: Selection, target: Selection) -> Span<'static> {
	if current == target {
		target.as_str().light_blue().bold().on_gray()
	} else {
		target.as_str().white()
	}
}