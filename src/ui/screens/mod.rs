mod progress;
mod error_screen;
mod progress_log_screen;
mod info_screen;
mod input_screen;
mod menu;
mod task_screen;
mod text_screen;
mod groupe_table;
mod membre_table;
mod compte_table;
mod view_table;
mod page_membre;
mod page_compte;
mod page_groupe;

use std::sync::{Arc, Mutex, RwLock};

pub use progress::ProgressBar;
pub use error_screen::*;
pub use progress_log_screen::*;
pub use info_screen::*;
pub use input_screen::*;
pub use menu::*;
pub use task_screen::*;
pub use text_screen::*;
pub use groupe_table::*;
pub use membre_table::*;
pub use compte_table::*;
pub use view_table::*;
pub use page_membre::*;
pub use page_compte::*;
pub use page_groupe::*;
use ratatui::{style::{Color, Stylize}, text::{Line, Span, Text}, widgets::{Paragraph, Wrap}};
use lazy_static::lazy_static;

use crate::{cdj::{comptes::CompteID, groupes::GroupeID, membres::{Contact, Interet, MembreID}}, data::{BoolJustifie, Genre, Taille, adresse::Adresse, cam::CAM, email::Email, tel::Tel}, prelude::{Date, OuiNon}, ui::{UpdateAction, actions::UpdateActions}};

lazy_static!{
	pub static ref ENTER_INSTRUCTIONS: Line<'static> = Line::from(vec![
		" Appuyez sur ".gray(),
		"Entrée".light_blue().bold(),
		" pour continuer ".gray(),
	]).centered();
	pub static ref ESC_INSTRUCTIONS: Line<'static> = Line::from(vec![
		" Appuyez sur ".gray(),
		"ESC".light_blue().bold(),
		" pour annuler ".gray(),
	]).centered();
	pub static ref ENTER_ESC_INSTRUCTIONS: Line<'static> = Line::from(vec![
		" Appuyez sur ".gray(),
		"Entrée".light_blue().bold(),
		" pour valider, ".gray(),
		"Esc".light_blue().bold(),
		" pour annuler ".gray(),
	]).centered();
}


#[derive(Debug, Clone, Default)]
pub enum Desc {
	#[default]
	None,
	Info(String),
	Warning(String),
	Error(String),
}
impl Desc {
	pub fn as_str(&self) -> &str {
		match self {
			Desc::None => "",
			Desc::Info(s) | Desc::Warning(s) | Desc::Error(s) => s.as_str(),
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct Logger<'a> {
	text: Text<'a>,
	dirty: bool,
}
impl Logger<'_> {
	pub fn log(&mut self, desc: Desc) {
		let line = match desc {
			Desc::None => Line::default(),
			Desc::Info(s) => Line::from(s).green(),
			Desc::Warning(s) => Line::from(s).yellow(),
			Desc::Error(s) => Line::from(s).red(),
		};
		self.text.push_line(line);
		self.dirty = true;
	}
	pub fn widget(&'_ self) -> Paragraph<'_> {
		Paragraph::new(self.text.clone()).wrap(Wrap { trim: false })
	}
	/*
	pub fn height(&self, width: u16) -> u16 {
		self.lns.iter().filter_map(|desc| {
			match desc {
				Desc::None => None,
				Desc::Info(s) => Some(textwrap::wrap(s, width as usize).len() as u16),
				Desc::Warning(s) => Some(textwrap::wrap(s, width as usize).len() as u16),
				Desc::Error(s) => Some(textwrap::wrap(s, width as usize).len() as u16),
			}
		}).sum()
	}
	*/
	pub fn clean(&mut self) {
		self.dirty = false;
	}
	pub fn is_dirty(&self) -> bool {
		self.dirty
	}
}

#[derive(Debug, Clone, Copy, Default)]
enum ScrollMode {
	#[default]
	Auto,
	Manual(usize),
}

#[derive(Debug)]
pub enum PageError {
	NonmatchingIDs{msg: String},
	MissingData{msg: String},
}
impl std::fmt::Display for PageError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PageError::NonmatchingIDs { msg } => write!(f, "Non matching IDs:{}", msg),
			PageError::MissingData { msg } => write!(f, "Missing data: {}", msg),
		}
	}
}
impl std::error::Error for PageError {}

#[derive(Debug, Clone, Copy)]
pub enum AssociatedPage {
	Membre{mid: MembreID},
	Compte{cid: CompteID},
	Groupe{gid: GroupeID, sg: Option<u32>},
}

#[derive(Debug, Clone)]
pub enum FieldType {
	Str(Option<String>),
	Bool(Option<bool>),
	Int(Option<i32>),
	Email(Option<Email>),
	Adresse(Adresse),
	Tel(Option<Tel>),
	BoolJustify(Option<BoolJustifie>),
	Cam(Option<CAM>),
	Date(Option<Date>),
	Genre(Option<Genre>),
	Taille(Option<Taille>),
	Interet(Option<Interet>),
	Contact(Option<Contact>),
}
impl From<&str> for FieldType {
	fn from(value: &str) -> Self {
		FieldType::Str(Some(value.to_string()))
	}
}
impl From<Option<&str>> for FieldType {
	fn from(value: Option<&str>) -> Self {
		FieldType::Str(value.map(|s| s.to_string()))
	}
}
impl From<bool> for FieldType {
	fn from(value: bool) -> Self {
		FieldType::Bool(Some(value))
	}
}
impl From<Option<bool>> for FieldType {
	fn from(value: Option<bool>) -> Self {
		FieldType::Bool(value)
	}
}
impl From<i32> for FieldType {
	fn from(value: i32) -> Self {
		FieldType::Int(Some(value))
	}
}
impl From<Option<i32>> for FieldType {
	fn from(value: Option<i32>) -> Self {
		FieldType::Int(value)
	}
}
impl From<Email> for FieldType {
	fn from(value: Email) -> Self {
		FieldType::Email(Some(value))
	}
}
impl From<Option<Email>> for FieldType {
	fn from(value: Option<Email>) -> Self {
		FieldType::Email(value)
	}
}
impl From<Adresse> for FieldType {
	fn from(value: Adresse) -> Self {
		FieldType::Adresse(value)
	}
}
impl From<Option<Adresse>> for FieldType {
	fn from(value: Option<Adresse>) -> Self {
		FieldType::Adresse(value.unwrap_or_default())
	}
}
impl From<Tel> for FieldType {
	fn from(value: Tel) -> Self {
		FieldType::Tel(Some(value))
	}
}
impl From<Option<Tel>> for FieldType {
	fn from(value: Option<Tel>) -> Self {
		FieldType::Tel(value)
	}
}
impl From<BoolJustifie> for FieldType {
	fn from(value: BoolJustifie) -> Self {
		FieldType::BoolJustify(Some(value))
	}
}
impl From<Option<BoolJustifie>> for FieldType {
	fn from(value: Option<BoolJustifie>) -> Self {
		FieldType::BoolJustify(value)
	}
}
impl From<CAM> for FieldType {
	fn from(value: CAM) -> Self {
		FieldType::Cam(Some(value))
	}
}
impl From<Option<CAM>> for FieldType {
	fn from(value: Option<CAM>) -> Self {
		FieldType::Cam(value)
	}
}
impl From<Date> for FieldType {
	fn from(value: Date) -> Self {
		FieldType::Date(Some(value))
	}
}
impl From<Option<Date>> for FieldType {
	fn from(value: Option<Date>) -> Self {
		FieldType::Date(value)
	}
}
impl From<Genre> for FieldType {
	fn from(value: Genre) -> Self {
		FieldType::Genre(Some(value))
	}
}
impl From<Option<Genre>> for FieldType {
	fn from(value: Option<Genre>) -> Self {
		FieldType::Genre(value)
	}
}
impl From<Taille> for FieldType {
	fn from(value: Taille) -> Self {
		FieldType::Taille(Some(value))
	}
}
impl From<Option<Taille>> for FieldType {
	fn from(value: Option<Taille>) -> Self {
		FieldType::Taille(value)
	}
}
impl From<OuiNon> for FieldType {
	fn from(value: OuiNon) -> Self {
		FieldType::Bool(Some(value.into()))
	}
}
impl From<Option<OuiNon>> for FieldType {
	fn from(value: Option<OuiNon>) -> Self {
		FieldType::Bool(value.map(|v| v.into()))
	}
}
impl From<Interet> for FieldType {
	fn from(value: Interet) -> Self {
		FieldType::Interet(Some(value))
	}
}
impl From<Option<Interet>> for FieldType {
	fn from(value: Option<Interet>) -> Self {
		FieldType::Interet(value)
	}
}
impl From<Contact> for FieldType {
	fn from(value: Contact) -> Self {
		FieldType::Contact(Some(value))
	}
}
impl From<Option<Contact>> for FieldType {
	fn from(value: Option<Contact>) -> Self {
		FieldType::Contact(value)
	}
}

pub struct Field {
	label: Option<String>,
	value: FieldType,
	after: Option<TextInputAfter>,
	multiline: bool,
	associated_page: Option<AssociatedPage>,
	editable: bool,
}
impl std::fmt::Debug for Field {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Field")
			.field("label", &self.label)
			.field("value", &self.value)
			.field("after", &self.after.is_some())
			.field("multiline", &self.multiline)
			.field("associated_page", &self.associated_page)
			.field("editable", &self.editable)
			.finish()
	}
}
impl<T> From<T> for Field where FieldType: From<T> {
	fn from(value: T) -> Self {
		Field { label: None, value: FieldType::from(value), after: None, multiline: false, associated_page: None, editable: true }
	}
}
impl Field {
	pub fn with_label(mut self, label: String) -> Self {
		self.label = Some(label);
		self
	}
	pub fn with_after(mut self, after: TextInputAfter) -> Self {
		self.after = Some(after);
		self
	}
	pub fn with_multiline(mut self, multiline: bool) -> Self {
		self.multiline = multiline;
		self
	}
	pub fn with_associated_page(mut self, page: AssociatedPage) -> Self {
		self.associated_page = Some(page);
		self
	}
	pub fn set_editable(mut self, editable: bool) -> Self {
		self.editable = editable;
		self
	}

	pub fn get_str(&self) -> Option<Option<&str>> {
		match &self.value {
			FieldType::Str(s) => Some(s.as_deref()),
			_ => None,
		}
	}
	pub fn get_bool(&self) -> Option<Option<bool>> {
		match &self.value {
			FieldType::Bool(b) => Some(*b),
			_ => None,
		}
	}
	pub fn get_int(&self) -> Option<Option<i32>> {
		match &self.value {
			FieldType::Int(i) => Some(*i),
			_ => None,
		}
	}
	pub fn get_email(&self) -> Option<Option<&Email>> {
		match &self.value {
			FieldType::Email(e) => Some(e.as_ref()),
			_ => None,
		}
	}
	pub fn get_adresse(&self) -> Option<&Adresse> {
		match &self.value {
			FieldType::Adresse(a) => Some(a),
			_ => None,
		}
	}
	pub fn get_tel(&self) -> Option<Option<&Tel>> {
		match &self.value {
			FieldType::Tel(t) => Some(t.as_ref()),
			_ => None,
		}
	}
	pub fn get_bool_justifie(&self) -> Option<Option<&BoolJustifie>> {
		match &self.value {
			FieldType::BoolJustify(bj) => Some(bj.as_ref()),
			_ => None,
		}
	}
	pub fn get_cam(&self) -> Option<Option<&CAM>> {
		match &self.value {
			FieldType::Cam(cam) => Some(cam.as_ref()),
			_ => None,
		}
	}
	pub fn get_date(&self) -> Option<Option<&Date>> {
		match &self.value {
			FieldType::Date(d) => Some(d.as_ref()),
			_ => None,
		}
	}
	pub fn get_genre(&self) -> Option<Option<&Genre>> {
		match &self.value {
			FieldType::Genre(g) => Some(g.as_ref()),
			_ => None,
		}
	}
	pub fn get_taille(&self) -> Option<Option<&Taille>> {
		match &self.value {
			FieldType::Taille(t) => Some(t.as_ref()),
			_ => None,
		}
	}
	pub fn get_interet(&self) -> Option<Option<&Interet>> {
		match &self.value {
			FieldType::Interet(i) => Some(i.as_ref()),
			_ => None,
		}
	}
	pub fn get_contact(&self) -> Option<Option<&Contact>> {
		match &self.value {
			FieldType::Contact(c) => Some(c.as_ref()),
			_ => None,
		}
	}

	pub fn is_some(&self) -> bool {
		match &self.value {
			FieldType::Str(s) => s.is_some(),
			FieldType::Bool(b) => b.is_some(),
			FieldType::Int(i) => i.is_some(),
			FieldType::Email(e) => e.is_some(),
			FieldType::Adresse(a) => true,
			FieldType::Tel(t) => t.is_some(),
			FieldType::BoolJustify(bj) => bj.is_some(),
			FieldType::Cam(cam) => cam.is_some(),
			FieldType::Date(d) => d.is_some(),
			FieldType::Genre(g) => g.is_some(),
			FieldType::Taille(t) => t.is_some(),
			FieldType::Interet(i) => i.is_some(),
			FieldType::Contact(c) => c.is_some(),
		}
	}

	pub fn set_value(&mut self, value: FieldType) {
		self.value = value;
	}

	pub fn value_to_string(&self) -> String {
		match &self.value {
			FieldType::Str(Some(s)) => s.clone(),
			FieldType::Bool(Some(b)) => OuiNon::from(*b).to_string(),
			FieldType::Int(Some(i)) => i.to_string(),
			FieldType::Email(Some(e)) => e.to_string(),
			FieldType::Adresse(a) => a.to_string(),
			FieldType::Tel(Some(t)) => t.to_string(),
			FieldType::BoolJustify(Some(bj)) => bj.to_string(),
			FieldType::Cam(Some(cam)) => cam.to_string(),
			FieldType::Date(Some(d)) => d.format("%Y-%m-%d").to_string(),
			FieldType::Genre(Some(g)) => g.to_string(),
			FieldType::Taille(Some(t)) => t.to_string(),
			FieldType::Interet(Some(i)) => i.to_string(),
			FieldType::Contact(Some(c)) => c.to_string(),
			_ => String::new(),
		}
	}
	pub fn to_line(&self, selected: bool) -> Line<'static> {
		let val = self.value_to_string();
		let val = if let FieldType::Bool(Some(b)) = &self.value {
			if *b {
				Span::from(val).green()
			} else {
				Span::from(val).red()
			}
		} else if selected {
			Span::from(val).white()
		} else {
			Span::from(val).gray()
		};
		if selected {
			Line::from(vec![
				Span::from(format!("{}: ", self.label.as_deref().unwrap_or(""))).white().bold(),
				val,
			]).bg(Color::DarkGray)
		} else {
			Line::from(vec![
				Span::from(format!("{}: ", self.label.as_deref().unwrap_or(""))).white().bold(),
				val,
			])
		}
	}
	pub fn to_line_and_count(&self, width: u16) -> (Line<'static>, usize) {
		let line = self.to_line(false);
		let line_count = Paragraph::new(line.clone()).wrap(Wrap { trim: false }).line_count(width);
		(line, line_count)
	}

	pub fn on_action(this: Arc<RwLock<Self>>, dirty_flag: Option<Arc<Mutex<bool>>>) -> UpdateActions {
		let editable = this.read().expect("Poisoned Lock").editable;
		let associated_page = this.read().expect("Poisoned Lock").associated_page;
		match (editable, associated_page) {
			(true, Some(page)) => { // give choice to edit or open page
				let menu = Menu::new(Box::new([
					MenuItem {id: "Editer", action: Box::new(move |state| {
						let mut actions = mk_action(this.clone(), dirty_flag.clone());
						actions.insert(0, UpdateAction::Pop);
						Ok(actions)
					}) },
					MenuItem {id: "Voir", action: Box::new(move |state| {
						Ok(vec![
							UpdateAction::Pop, // pop the menu
							match page {
								AssociatedPage::Membre { mid } => UpdateAction::OpenMembre(mid),
								AssociatedPage::Compte { cid } => UpdateAction::OpenCompte(cid),
								AssociatedPage::Groupe { gid, sg } => UpdateAction::OpenGroupe(gid, sg),
							},
						])
					})},
				]));
				UpdateAction::PushSub(Box::new(menu)).one()
			},
			(true, None) => mk_action(this, dirty_flag),
			(false, Some(page)) => { // just open page
				match page {
					AssociatedPage::Membre { mid } => UpdateAction::OpenMembre(mid).one(),
					AssociatedPage::Compte { cid } => UpdateAction::OpenCompte(cid).one(),
					AssociatedPage::Groupe { gid, sg } => UpdateAction::OpenGroupe(gid, sg).one(),
				}
			},
			(false, None) => UpdateAction::Continue.one(),
		}
	}
}
#[derive(Debug, Default)]
pub struct FieldBlock {
	title: Option<Line<'static>>,
	fields: Vec<Arc<RwLock<Field>>>,
}
impl FieldBlock {
	pub fn with_title(mut self, title: Line<'static>) -> Self {
		self.title = Some(title);
		self
	}
	pub fn add_field(&mut self, field: Field) -> Arc<RwLock<Field>> {
		let arc_field = Arc::new(RwLock::new(field));
		self.fields.push(arc_field.clone());
		arc_field
	}

	pub fn to_text(&self, selected: Option<Arc<RwLock<Field>>>) -> Text<'static> {
		let mut text = Text::default();
		if let Some(title) = &self.title {
			text.push_line(title.clone());
		}
		for field in &self.fields {
			let is_selected = selected.as_ref().is_some_and(|s| Arc::ptr_eq(s, field));
			text.push_line(field.read().expect("Poisoned Lock").to_line(is_selected));
		}
		text
	}
	pub fn idx_of(&self, field: &Arc<RwLock<Field>>) -> Option<usize> {
		self.fields.iter().position(|f| Arc::ptr_eq(f, field))
	}

	fn add_lines_to_cluster(&self, text: &mut Text<'static>, width: u16, selected: Option<Arc<RwLock<Field>>>, counter: &mut Option<usize>) -> Option<usize> {
		if let Some(title) = &self.title {
			add_line_to_text(title.clone(), text, width, counter);
		}
		let mut ret = None;
		for field in &self.fields {
			let is_selected = selected.as_ref().is_some_and(|s| Arc::ptr_eq(s, field));
			if is_selected {
				ret = *counter;
			}
			add_line_to_text(field.read().expect("Poisoned Lock").to_line(is_selected), text, width, counter);
		}
		ret
	}
}
#[derive(Debug, Default)]
pub struct FieldBlockCluster {
	title: Option<Line<'static>>,
	blocks: Vec<FieldBlock>,
}
impl FieldBlockCluster {
	pub fn new(blocks: Vec<FieldBlock>) -> Self {
		Self { title: None, blocks }
	}
	pub fn with_title(mut self, title: Line<'static>) -> Self {
		self.title = Some(title);
		self
	}
	pub fn add_block(&mut self, block: FieldBlock) -> &mut Self {
		self.blocks.push(block);
		self
	}

	pub fn get_text_and_scroll(&self, selected: Option<Arc<RwLock<Field>>>, height: u16, width: u16, current_scroll: u16) -> (Text<'static>, u16) {
		let mut text = Text::default();
		let mut at = if selected.is_none() {None} else {Some(0)};
		let mut selected_line_idx = None;
		if let Some(title) = &self.title {
			add_line_to_text(title.clone(), &mut text, width, &mut at);
			
		}
		let l = self.blocks.len();
		for (i, block) in self.blocks.iter().enumerate() {
			if i > 0 && i < l-1 {
				add_line_to_text(Line::default(), &mut text, width, &mut at);
			}
			selected_line_idx = block.add_lines_to_cluster(&mut text, width, selected.clone(), &mut at);
		}
		
		// determine scroll
		let total_lines = Paragraph::new(text.clone()).wrap(Wrap { trim: false }).line_count(width);
		let max_scroll = total_lines.saturating_sub(height as usize);
		let scroll = selected_line_idx.map_or(current_scroll, |idx| {
			let cs = current_scroll;
			let idx = idx as u16;
			if idx < cs {
				idx
			} else if idx >= cs + height {
				idx.saturating_sub(height).saturating_add(1)
			} else {
				cs
			}
		}).min(max_scroll as u16);

		(text, scroll)
	}

	
}

fn add_line_to_text(line: Line<'static>, text: &mut Text<'static>, width: u16, counter: &mut Option<usize>) {
	let height = Paragraph::new(line.clone()).wrap(Wrap { trim: false }).line_count(width);
	if let Some(c) = counter {
		*c += height;
	}
	text.push_line(line);
}

fn mk_menu_action<T>(id: T, field_hook: Arc<RwLock<Field>>, dirty_flag: Option<Arc<Mutex<bool>>>) -> Box<crate::ui::actions::Action> 
where T: Into<FieldType> + Clone + 'static {
	Box::new(move |state| {
		{
			let mut lock = field_hook.write().expect("Poisoned Lock");
			lock.value = id.clone().into();
		}
		if let Some(dirty_flag) = &dirty_flag {
			*dirty_flag.lock().expect("Poisoned Lock") = true;
		}
		Ok(UpdateAction::Pop.one())
	})
}

fn mk_action(this: Arc<RwLock<Field>>, dirty_flag: Option<Arc<Mutex<bool>>>) -> Vec<UpdateAction> {
	let field_hook = this.clone();
	let title = this.read().expect("Poisoned Lock").label.clone();
	let lock = this.read().expect("Poisoned Lock");
	match &lock.value {
		FieldType::Str(s) => {
			let mut input_screen = LineInputScreen::default()
				.with_value(s.clone().unwrap_or_default())
				.with_after(Box::new(move |input, state| {
					if let Some(input) = input {
						let val = input.trim().to_string();
						let val = if val.is_empty() {
							None
						} else {
							Some(val)
						};
						let mut lock = field_hook.write().expect("Poisoned Lock");
						lock.value = FieldType::Str(val);
						if let Some(dirty_flag) = &dirty_flag {
							*dirty_flag.lock().expect("Poisoned Lock") = true;
						}
					}
					Ok(UpdateAction::Pop.one())
				}));
			if let Some(title) = title {
				input_screen = input_screen.with_title(title);
			};
			UpdateAction::PushSub(Box::new(input_screen)).one()
		},
		FieldType::Bool(b) => {
			let mut screen = Menu::new(Box::new([
				MenuItem {id: OuiNon::Oui, action: mk_menu_action(OuiNon::Oui, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: OuiNon::Non, action: mk_menu_action(OuiNon::Non, field_hook.clone(), dirty_flag.clone())},
			]));
			if let Some(title) = title {
				screen = screen.with_title(title);
			};
			UpdateAction::PushSub(Box::new(screen)).one()
		},
		FieldType::Int(i) => {
			let mut input_screen = LineInputScreen::default()
				.with_value(i.map(|i| i.to_string()).unwrap_or_default())
				.with_validation(Arc::new(|s| s.trim().parse::<u32>().is_ok()))
				.with_after(Box::new(move |input, state| {
					if let Some(input) = input {
						let mut lock = field_hook.write().expect("Poisoned Lock");
						if input.trim().is_empty() {
							lock.value = FieldType::Int(None);
							if let Some(dirty_flag) = &dirty_flag {
								*dirty_flag.lock().expect("Poisoned Lock") = true;
							}
						} else if let Ok(val) = input.trim().parse::<i32>() {
							lock.value = FieldType::Int(Some(val));
							if let Some(dirty_flag) = &dirty_flag {
								*dirty_flag.lock().expect("Poisoned Lock") = true;
							}
						} else {
							return Ok(UpdateAction::Bell.one());
						}
					}
					Ok(UpdateAction::Pop.one())
				}));
			if let Some(title) = title {
				input_screen = input_screen.with_title(title);
			};
			UpdateAction::PushSub(Box::new(input_screen)).one()
		},
		FieldType::Email(e) => {
			let mut input_screen = LineInputScreen::default()
				.with_value(e.clone().map(|e| e.to_string()).unwrap_or_default())
				.with_validation(Arc::new(|s| s.trim().parse::<Email>().is_ok()))
				.with_after(Box::new(move |input, state| {
					if let Some(input) = input {
						let input = input.trim();
						let mut lock = field_hook.write().expect("Poisoned Lock");
						if input.is_empty() {
							lock.value = FieldType::Email(None);
							if let Some(dirty_flag) = &dirty_flag {
								*dirty_flag.lock().expect("Poisoned Lock") = true;
							}
						} else if let Ok(val) = input.trim().parse::<Email>() {
							lock.value = FieldType::Email(Some(val));
							if let Some(dirty_flag) = &dirty_flag {
								*dirty_flag.lock().expect("Poisoned Lock") = true;
							}
						} else {
							return Ok(UpdateAction::Bell.one());
						}
					}
					Ok(UpdateAction::Pop.one())
				}));
			if let Some(title) = title {
				input_screen = input_screen.with_title(title);
			};
			UpdateAction::PushSub(Box::new(input_screen)).one()
		},
		FieldType::Adresse(a) => UpdateAction::Continue.one(),
		FieldType::Tel(t) => {
			let mut input_screen = LineInputScreen::default()
				.with_value(t.map(|t| t.to_string()).unwrap_or_default())
				.with_validation(Arc::new(|s| s.trim().parse::<Tel>().is_ok()))
				.with_message(Text::from("123-456-7890"))
				.with_after(Box::new(move |input, state| {
					if let Some(input) = input {
						let input = input.trim();
						let mut lock = field_hook.write().expect("Poisoned Lock");
						if input.is_empty() {
							lock.value = FieldType::Tel(None);
							if let Some(dirty_flag) = &dirty_flag {
								*dirty_flag.lock().expect("Poisoned Lock") = true;
							}
						} else if let Ok(val) = input.parse::<Tel>() {
							lock.value = FieldType::Tel(Some(val));
							if let Some(dirty_flag) = &dirty_flag {
								*dirty_flag.lock().expect("Poisoned Lock") = true;
							}
						} else {
							return Ok(UpdateAction::Bell.one());
						}
					}
					Ok(UpdateAction::Pop.one())
				}));
			if let Some(title) = title {
				input_screen = input_screen.with_title(title);
			};
			UpdateAction::PushSub(Box::new(input_screen)).one()
		},
		FieldType::BoolJustify(bj) => UpdateAction::Continue.one(),
		FieldType::Cam(cam) => UpdateAction::Continue.one(),
		FieldType::Date(d) => {
			let mut input_screen = LineInputScreen::default()
				.with_value(d.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default())
				.with_validation(Arc::new(|s| s.trim().parse::<Date>().is_ok()))
				.with_message(Text::from("AAAA-mm-jj"))
				.with_after(Box::new(move |input, state| {
					if let Some(input) = input {
						let input = input.trim();
						let mut lock = field_hook.write().expect("Poisoned Lock");
						if input.is_empty() {
							lock.value = FieldType::Date(None);
							if let Some(dirty_flag) = &dirty_flag {
								*dirty_flag.lock().expect("Poisoned Lock") = true;
							}
						} else if let Ok(val) = input.trim().parse::<Date>() {
							lock.value = FieldType::Date(Some(val));
							if let Some(dirty_flag) = &dirty_flag {
								*dirty_flag.lock().expect("Poisoned Lock") = true;
							}
						} else {
							return Ok(UpdateAction::Bell.one());
						}
					}
					Ok(UpdateAction::Pop.one())
				}));
			if let Some(title) = title {
				input_screen = input_screen.with_title(title);
			};
			UpdateAction::PushSub(Box::new(input_screen)).one()
		},
		FieldType::Genre(g) => {

			let mut input_screen = Menu::new(Box::new([
				MenuItem {id: Genre::Homme, action: mk_menu_action(Genre::Homme, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: Genre::Femme, action: mk_menu_action(Genre::Femme, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: Genre::Autre, action: mk_menu_action(Genre::Autre, field_hook.clone(), dirty_flag.clone())},
			]));
			if let Some(title) = title {
				input_screen = input_screen.with_title(title);
			};
			UpdateAction::PushSub(Box::new(input_screen)).one()
		},
		FieldType::Taille(t) => {
			
			let mut input_screen = Menu::new(Box::new([
				MenuItem {id: Taille::XS, action: mk_menu_action(Taille::XS, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: Taille::S, action: mk_menu_action(Taille::S, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: Taille::M, action: mk_menu_action(Taille::M, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: Taille::L, action: mk_menu_action(Taille::L, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: Taille::XL, action: mk_menu_action(Taille::XL, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: Taille::XXL, action: mk_menu_action(Taille::XXL, field_hook.clone(), dirty_flag.clone())},
			]));
			if let Some(title) = title {
				input_screen = input_screen.with_title(title);
			};
			UpdateAction::PushSub(Box::new(input_screen)).one()
		},
		FieldType::Interet(i) => {
			let mut input_screen = Menu::new(Box::new([
				MenuItem {id: Interet::Art, action: mk_menu_action(Interet::Art, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: Interet::Nature, action: mk_menu_action(Interet::Nature, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: Interet::Science, action: mk_menu_action(Interet::Science, field_hook.clone(), dirty_flag.clone())},
				MenuItem {id: Interet::Sport, action: mk_menu_action(Interet::Sport, field_hook.clone(), dirty_flag.clone())},
			]));
			if let Some(title) = title {
				input_screen = input_screen.with_title(title);
			};
			UpdateAction::PushSub(Box::new(input_screen)).one()
		},
		FieldType::Contact(c) => {
			UpdateAction::Continue.one()
		},
	}
}