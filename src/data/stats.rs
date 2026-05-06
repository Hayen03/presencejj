use std::{collections::{BTreeSet, HashMap}, fmt::Display, fs::File, hash::Hash, sync::{Arc, Mutex}};

use lazy_static::lazy_static;
//use rand::rand_core::le;
use rust_xlsxwriter::{chart::{Chart, ChartDataLabel, ChartType}, workbook::Workbook, worksheet::Worksheet, Color, Format, FormatAlign, FormatBorder, Formula, Table, TableColumn, XlsxError};

use crate::{cdj::{comptes::CompteReg, groupes::{Groupe, GroupeID, GroupeReg}, membres::{MembreID, MembreReg}}, data::adresse::CodePostal, ui::screens::Desc};

static XLSX_SITE_BLOCK_SPACE: u32 = 3; // espace entre les blocs de site dans le fichier excel
lazy_static!{
	static ref XLSX_SITE_FORMAT: Format = Format::new()
		.set_bold()
		.set_align(FormatAlign::Left)
		.set_font_size(24)
		.set_background_color(Color::RGB(0x4DA6FF));
	static ref XLSX_CAT_FORMAT: Format = Format::new()
	.set_bold()
	.set_align(FormatAlign::Center)
	.set_font_size(18)
	.set_background_color(Color::RGB(0x3399FF));
	static ref XLSX_COL_FORMAT: Format = Format::new()
		.set_bold()
		.set_align(FormatAlign::Center)
		.set_border_bottom(FormatBorder::Medium);
	static ref XLSX_SEP_FORMAT: Format = Format::new()
		.set_border_right(FormatBorder::Thin);
	static ref XLSX_COL_SEP_FORMAT: Format = Format::new()
		.set_bold()
		.set_align(FormatAlign::Center)
		.set_border_bottom(FormatBorder::Medium)
		.set_border_right(FormatBorder::Thin);
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub struct GroupeStats {
	pub capacite: usize,
	pub inscriptions: usize,
	pub annulations: usize, 
	pub liste_attente: usize,
}
fn get_group_stats(groupe: &Groupe, do_annulation: &dyn Fn(GroupeID, &str) -> Option<usize>, do_attente: &dyn Fn(GroupeID, &str) -> Option<usize>, get_missing_capacite: &dyn Fn(GroupeID, &str) -> usize) -> GroupeStats {
	let desc = groupe.desc();
	GroupeStats{
		capacite: groupe.capacite.unwrap_or_else(|| get_missing_capacite(groupe.id, &desc)),
		inscriptions: groupe.participants.len(),
		annulations: do_annulation(groupe.id, &desc).unwrap_or(0),
		liste_attente: do_attente(groupe.id, &desc).unwrap_or(0),
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash, Default)]
pub struct SemStats {
	pub capacite: usize,
	pub inscriptions: usize,
	pub annulations: usize,
	pub liste_attente: usize,
}

#[derive(Debug, Clone, Default)]
pub struct AgeStats {
	pub semaines: HashMap<String, SemStats>,
}
impl AgeStats {
	pub fn total(&self) -> SemStats {
		let mut total = SemStats::default();
		for sem_stats in self.semaines.values() {
			total.capacite += sem_stats.capacite;
			total.inscriptions += sem_stats.inscriptions;
			total.annulations += sem_stats.annulations;
			total.liste_attente += sem_stats.liste_attente;
		}
		total
	}
}

#[derive(Debug, Clone, Default)]
pub struct SiteStats {
	pub ages: HashMap<String, AgeStats>,
	pub villes: VilleStats,
}
impl SiteStats {
	pub fn total(&self) -> AgeStats {
		let mut stats = AgeStats::default();
		for age in self.ages.values() {
			for (sem, sem_stats) in &age.semaines {
				let s = stats.semaines.entry(sem.clone()).or_insert(SemStats::default());
				s.capacite += sem_stats.capacite;
				s.inscriptions += sem_stats.inscriptions;
				s.annulations += sem_stats.annulations;
				s.liste_attente += sem_stats.liste_attente;
			}
		}
		stats
	}
}

#[derive(Debug, Clone, Default)]
pub struct Stats {
	pub sites: HashMap<String, SiteStats>,
	pub villes: VilleStats,
}
impl Stats {
	pub fn total(&self) -> SiteStats {
		let mut stats = SiteStats::default();
		for site in self.sites.values() {
			for (age, age_stats) in &site.ages {
				let a = stats.ages.entry(age.clone()).or_default();
				for (sem, sem_stats) in &age_stats.semaines {
					let s = a.semaines.entry(sem.clone()).or_default();
					s.capacite += sem_stats.capacite;
					s.inscriptions += sem_stats.inscriptions;
					s.annulations += sem_stats.annulations;
					s.liste_attente += sem_stats.liste_attente;
				}
			}
		}
		stats
	}
}
pub struct FillStats<'a, 'b> {
	pub groupes: &'b mut dyn Iterator<Item=&'a Groupe>,
	pub membres: &'a MembreReg,
	pub comptes: &'a CompteReg,
	pub do_annulation: &'a dyn Fn(GroupeID, &str) -> Option<usize>,
	pub do_attente: &'a dyn Fn(GroupeID, &str) -> Option<usize>,
	pub get_missing_capacite: &'a dyn Fn(GroupeID, &str) -> usize,
	pub progress: Option<Arc<Mutex<u32>>>,
	pub cancel: Option<Arc<Mutex<bool>>>,
}
impl<'a, 'b> FillStats<'a, 'b> {
	pub fn fill(self) -> Option<(Stats, HashMap<GroupeID, GroupeStats>)> {
		// unpack the fields for easier access
		let FillStats { groupes, membres, comptes, do_annulation, do_attente, get_missing_capacite, progress, cancel } = self;

		let mut mids: BTreeSet<MembreID> = BTreeSet::new();
		let mut stats = Stats::default();
		let mut gstats = HashMap::new();
		for groupe in groupes {
			// early check for cancel
			if let Some(cancel_hook) = cancel.as_ref() {
				if *cancel_hook.lock().expect("Poisoned Lock") {
					return None;
				}
			}
			if !groupe.is_null() {
				let site_stats = stats.sites.entry(groupe.site.clone().unwrap_or_default()).or_default();
				let age_stats = site_stats.ages.entry(groupe.category.clone().unwrap_or_default()).or_default();
				let sem_stats = age_stats.semaines.entry(groupe.semaine.clone().unwrap_or_default()).or_default();
				
				let group_stats = get_group_stats(groupe, do_annulation, do_attente, get_missing_capacite);
				gstats.insert(groupe.id, group_stats);
				sem_stats.capacite += group_stats.capacite;
				sem_stats.inscriptions += group_stats.inscriptions;
				sem_stats.annulations += group_stats.annulations;
				sem_stats.liste_attente += group_stats.liste_attente;

				let mut partial_mids: BTreeSet<MembreID> = BTreeSet::new();
				// calcul des villes
				for mid in &groupe.participants {
					if let Ok(membre) = membres.get(*mid) {
						if let Some(cid) = membre.compte {
							if let Ok(compte) = comptes.get(cid) {
								if let Some(adr) = compte.adresse.as_ref() {
									if let Some(code) = adr.code_postal {
										let ville = Ville::get_from_code_postal(code);
										match ville {
											Some(ville) => {
												if partial_mids.insert(*mid) {
													let n = site_stats.villes.villes.entry(ville).or_default();
													*n += 1;
													site_stats.villes.total += 1;
												}
												if mids.insert(*mid) {
													let n = stats.villes.villes.entry(ville).or_default();
													*n += 1;
													stats.villes.total += 1;
												}
											},
											None => {
												if partial_mids.insert(*mid) {
													site_stats.villes.autres += 1;
													site_stats.villes.total += 1;
												}
												if mids.insert(*mid) {
													stats.villes.autres += 1;
													stats.villes.total += 1;
												}
											},
										}
									}
								} else { 
									if partial_mids.insert(*mid) {
										site_stats.villes.inconnues += 1;
										site_stats.villes.total += 1;
									}
									if mids.insert(*mid) {
										stats.villes.inconnues += 1;
										stats.villes.total += 1;
									}
								}
							} else { 
								if partial_mids.insert(*mid) {
									site_stats.villes.inconnues += 1;
									site_stats.villes.total += 1;
								}
								if mids.insert(*mid) {
									stats.villes.inconnues += 1;
									stats.villes.total += 1;
								}
							}
						} else { 
							if partial_mids.insert(*mid) {
								site_stats.villes.inconnues += 1;
								site_stats.villes.total += 1;
							}
							if mids.insert(*mid) {
								stats.villes.inconnues += 1;
								stats.villes.total += 1;
							}
						}
					} else { 
						if partial_mids.insert(*mid) {
							site_stats.villes.inconnues += 1;
							site_stats.villes.total += 1;
						}
						if mids.insert(*mid) {
							stats.villes.inconnues += 1;
							stats.villes.total += 1;
						}
					}
				}

				// incr progress
				if let Some(progress) = progress.as_ref() {
					let mut progress = progress.lock().expect("Poisoned Lock");
					*progress += 1;
				}
			}
		}
		Some((stats, gstats))
	}
}

#[derive(Debug, Clone, Default)]
pub struct UniqueStats {
	pub total: usize,
	pub ages: HashMap<String, usize>,
	pub sites: HashMap<String, usize>,
	pub sems: Vec<String>,
}
pub fn get_unique_stats<'a>(groupes: impl Iterator<Item=&'a Groupe>, progress: Option<Arc<Mutex<u32>>>, cancel: Option<Arc<Mutex<bool>>>) -> Option<UniqueStats> {
	let mut stats = UniqueStats::default();
	let mut all: BTreeSet<MembreID> = BTreeSet::new();
	let mut ages: HashMap<String, BTreeSet<MembreID>> = HashMap::new();
	let mut sites: HashMap<String, BTreeSet<MembreID>> = HashMap::new();
	let mut sems: BTreeSet<String> = BTreeSet::new();
	for groupe in groupes {
		// early check for cancel
		if let Some(cancel_hook) = cancel.as_ref() {
			if *cancel_hook.lock().expect("Poisoned Lock") {
				return None;
			}
		}
		if groupe.is_null() {
			continue;
		}
		let age_set = ages.entry(groupe.category.clone().unwrap_or_default()).or_default();
		let site_set = sites.entry(groupe.site.clone().unwrap_or_default()).or_default();
		sems.insert(groupe.semaine.clone().unwrap_or_default());
		for participant in &groupe.participants {
			all.insert(*participant);
			age_set.insert(*participant);
			site_set.insert(*participant);
		}
		// incr progress
		if let Some(progress) = progress.as_ref() {
			let mut progress = progress.lock().expect("Poisoned Lock");
			*progress += 1;
		}
	}
	stats.total = all.len();
	for (ages, set) in ages {
		stats.ages.insert(ages, set.len());
	}
	for (sites, set) in sites {
		stats.sites.insert(sites, set.len());
	}
	stats.sems = sems.into_iter().collect();
	stats.sems.sort_by(|a, b| {
		let ia = a.parse::<u32>();
		let ib = b.parse::<u32>();
		match (ia, ib) {
			(Ok(a), Ok(b)) => a.cmp(&b),
			_ => a.cmp(b)
		}
	});
	Some(stats)
}

#[derive(Debug)]
pub enum StatsError {
	FromXlsx(XlsxError),
	FromIO(std::io::Error),
	Cancelled,
}
impl From<XlsxError> for StatsError {
	fn from(value: XlsxError) -> Self {
		StatsError::FromXlsx(value)
	}
}
impl Display for StatsError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			StatsError::FromXlsx(e) => write!(f, "Erreur lors de la manipulation du fichier Excel: {}", e),
			Self::FromIO(e) => write!(f, "Erreur d'entrée/sortie: {}", e),
			Self::Cancelled => write!(f, "Action annulée par l'utilisateur"),
		}
	}
}
impl From<std::io::Error> for StatsError {
	fn from(value: std::io::Error) -> Self {
		StatsError::FromIO(value)
	}
}
impl std::error::Error for StatsError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			StatsError::FromXlsx(e) => Some(e),
			Self::FromIO(e) => Some(e),
			Self::Cancelled => None,
		}
	}
}

fn check_hook(progress: &Option<Arc<Mutex<u32>>>, cancel: &Option<Arc<Mutex<bool>>>) -> bool {
	if let Some(progress) = progress {
		*progress.lock().expect("Poisoned Lock") += 1;
	}
	if let Some(cancel) = cancel {
		*cancel.lock().expect("Poisoned Lock")
	} else {
		false
	}
}

type Gkey = (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>);
fn get_group_key(id: GroupeID, groupes: &GroupeReg) -> Gkey {
	match groupes.get(id) {
		Ok(groupe) => (
			groupe.saison.clone(),
			groupe.activite.clone(),
			groupe.site.clone(),
			groupe.category.clone(),
			groupe.semaine.clone(),
		),
		Err(_) => (None, None, None, None, None),
	}
}
pub struct StatsToExcel<'a> {
	pub stats: &'a Stats,
	pub gstats: &'a HashMap<GroupeID, GroupeStats>,
	pub groupes: &'a GroupeReg,
	pub ustats: &'a UniqueStats,
	pub out: &'a str,
	pub logger: &'a dyn Fn(Desc),
	pub progress: Option<Arc<Mutex<u32>>>,
	pub cancel: Option<Arc<Mutex<bool>>>,
}
impl StatsToExcel<'_> {
	pub fn print(self) -> Result<(), StatsError> {
		// unwrap the fields for easier access
		let StatsToExcel { stats, gstats, groupes, ustats, out, logger, progress, cancel } = self;

		let mut workbook = Workbook::new();

		// adding raw data worksheet
		let rawdata_sheet = workbook.add_worksheet();
		let _ = rawdata_sheet.set_name("Donnees")?;
		let gdata_table = Table::new()
			.set_columns(&[
				TableColumn::new().set_header("Saison"),
				TableColumn::new().set_header("Activite"),
				TableColumn::new().set_header("Site"),
				TableColumn::new().set_header("Categorie"),
				TableColumn::new().set_header("Semaine"),
				TableColumn::new().set_header("Capacite"),
				TableColumn::new().set_header("Inscriptions"),
				TableColumn::new().set_header("Annulations"),
				TableColumn::new().set_header("Attente"),
			])
			.set_name("groupes");
		let gstats = {
			let mut ngs = HashMap::new();
			for (id, gstat) in gstats {
				ngs.insert(get_group_key(*id, groupes), *gstat);
			}
			ngs
		};
		let mut gdata_keys_rows: Vec<[String; 5]> = Vec::new();
		let mut gdata_data_rows: Vec<[u32; 4]> = Vec::new();
		let mut sorted_keys: Vec<Gkey> = gstats.keys().cloned().collect();
		sorted_keys.sort();
		let glen = sorted_keys.len() as u32;
		for key in sorted_keys {
			let gstat = gstats.get(&key).unwrap();
			gdata_keys_rows.push([
				key.0.unwrap_or_default(),
				key.1.unwrap_or_default(),
				key.2.unwrap_or_default(),
				key.3.unwrap_or_default(),
				key.4.unwrap_or_default(),
			]);
			gdata_data_rows.push([
				gstat.capacite as u32,
				gstat.inscriptions as u32,
				gstat.annulations as u32,
				gstat.liste_attente as u32,
			]);
			if check_hook(&progress, &cancel) {
				return Err(StatsError::Cancelled);
			}
		}

		rawdata_sheet.write_row_matrix(1, 0, gdata_keys_rows)?;
		rawdata_sheet.write_row_matrix(1, 5, gdata_data_rows)?;
		if check_hook(&progress, &cancel) {
			return Err(StatsError::Cancelled);
		}

		rawdata_sheet.add_table(0, 0, glen, 8, &gdata_table)?;
		// tables d'enfant unique
		let udata_site_table = Table::new()
			.set_columns(&[
				TableColumn::new().set_header("Site"),
				TableColumn::new().set_header("Enfants uniques"),
			])
			.set_name("unique_sites");
		let udata_age_table = Table::new()
			.set_columns(&[
				TableColumn::new().set_header("Categorie"),
				TableColumn::new().set_header("Enfants uniques"),
			])
			.set_name("unique_categories");
		let mut udata_site_col: Vec<String> = ustats.sites.keys().cloned().collect();
		udata_site_col.sort();
		let mut udata_age_col: Vec<String> = ustats.ages.keys().cloned().collect();
		udata_age_col.sort();
		let mut udata_site_datacol: Vec<u32> = Vec::new();
		for site in &udata_site_col {
			udata_site_datacol.push(*ustats.sites.get(site).unwrap() as u32);
		}
		let mut udata_age_datacol: Vec<u32> = Vec::new();
		for age in &udata_age_col {
			udata_age_datacol.push(*ustats.ages.get(age).unwrap() as u32);
		}
		let udata_site_len = udata_site_col.len() as u32;
		let udata_age_len = udata_age_col.len() as u32;
		//println!("Sites: {:?}", udata_site_col);
		//println!("Ages: {:?}", udata_age_col);
		rawdata_sheet.write_column(1, 10, &udata_site_col)?;
		rawdata_sheet.write_column(1, 11, udata_site_datacol)?;
		rawdata_sheet.write(1+udata_site_len, 10, "Total")?;
		rawdata_sheet.write(1+udata_site_len, 11, ustats.total as u32)?;
		rawdata_sheet.write_column(1, 13, &udata_age_col)?;
		rawdata_sheet.write_column(1, 14, udata_age_datacol)?;
		rawdata_sheet.add_table(0, 10, udata_site_len+1, 11, &udata_site_table)?;
		rawdata_sheet.add_table(0, 13, udata_age_len, 14, &udata_age_table)?;
		if check_hook(&progress, &cancel) {
			return Err(StatsError::Cancelled);
		}

		// Tableau pour les stats de ville
		let ville_table = Table::new()
			.set_columns(&[
				TableColumn::new().set_header("Site"), 
				TableColumn::new().set_header("Longueuil"), 
				TableColumn::new().set_header("St-Hubert"), 
				TableColumn::new().set_header("Greenfield Park"), 
				TableColumn::new().set_header("Autres"), 
				TableColumn::new().set_header("Inconnues"), 
				TableColumn::new().set_header("Total"),
			])
			.set_name("villes");
		let at_row = 1;
		let at_col = 16;
		rawdata_sheet.write_column(at_row, at_col, &udata_site_col)?;
		rawdata_sheet.write(at_row + udata_site_len, at_col, "Total")?;
		for (i, site) in udata_site_col.iter().enumerate() {
			let mut site_entry = stats.sites.get(site).cloned().unwrap_or_default();
			let ville_stats = &mut site_entry.villes;
			let data_row = [
				*ville_stats.villes.entry(Ville::Longueuil).or_default(),
				*ville_stats.villes.entry(Ville::StHubert).or_default(),
				*ville_stats.villes.entry(Ville::GreenfieldPark).or_default(),
				ville_stats.autres,
				ville_stats.inconnues,
				ville_stats.total,
			];
			rawdata_sheet.write_row(at_row + i as u32, at_col + 1, data_row)?;
		}
		if check_hook(&progress, &cancel) {
			return Err(StatsError::Cancelled);
		}
		// la ligne de total
		{
			let mut total_stats = stats.villes.clone();
			let data_row = [
				*total_stats.villes.entry(Ville::Longueuil).or_default(),
				*total_stats.villes.entry(Ville::StHubert).or_default(),
				*total_stats.villes.entry(Ville::GreenfieldPark).or_default(),
				total_stats.autres,
				total_stats.inconnues,
				total_stats.total,
			];
			rawdata_sheet.write_row(at_row + udata_site_len, at_col + 1, data_row)?;
		}
		rawdata_sheet.add_table(at_row-1, at_col, at_row+udata_site_len, at_col + 6, &ville_table)?;
		rawdata_sheet.autofit();
		if check_hook(&progress, &cancel) {
			return Err(StatsError::Cancelled);
		}

		// Faire la feuille de stats
		let stats_sheet = workbook.add_worksheet();
		stats_sheet.set_name("Stats")?;
		let percent_format = Format::new().set_num_format("0.00%");
		// on fait des gros blocs par site de camp
		let mut at_site = 0; // debut du bloc de site
		let cols: [String; 6] = [
					"Capacite".into(), "Inscriptions".into(), "Annulations".into(), "Attente".into(), "% Occupation".into(), "% Annulation".into(),
				];
		for site in udata_site_col {
			SiteBlock {
				percent_format: &percent_format,
				stats,
				ustats,
				cols: &cols,
				at_site,
				site: &site,
				total: TotalStatus::new(false, false, false),
			}.write(stats_sheet)?;
			at_site += 4 + ustats.sems.len() as u32 + XLSX_SITE_BLOCK_SPACE; // une ligne par semaine + 4 lignes de titre (et une de total) + 3 lignes de séparation
			if check_hook(&progress, &cancel) {
				return Err(StatsError::Cancelled);
			}
		}
		// gros bloc de total
		{
			SiteBlock {
				percent_format: &percent_format,
				stats,
				ustats,
				cols: &cols,
				at_site,
				site: "Total",
				total: TotalStatus::new(true, false, false),
			}.write(stats_sheet)?;
		}
		stats_sheet.autofit();
		if check_hook(&progress, &cancel) {
			return Err(StatsError::Cancelled);
		}

		// Charts
		let chart_sheet = workbook.add_worksheet();
		chart_sheet.set_name("Charts")?;
		chart_occupation_par_categorie(chart_sheet, stats, ustats, 0, 0)?;
		chart_occupation_annulations_par_semaine(chart_sheet, stats, ustats, 0, 8)?;
		chart_enfants_uniques_par_site(chart_sheet, stats, ustats, 16, 0)?;
		chart_occupation_par_site(chart_sheet, stats, ustats, 16, 8)?;
		chart_proportion_ville(chart_sheet, stats, ustats, 0, 16)?;
		if check_hook(&progress, &cancel) {
			return Err(StatsError::Cancelled);
		}

		let file = File::create(out)?;
		workbook.save_to_writer(file)?;
		Ok(())
	}
}

pub fn to_excel_column(mut n: u16) -> String {
    if n == 0 {
        return String::new();
    }

    let mut result = String::new();
    while n > 0 {
        n -= 1; // Adjust because Excel columns are 1-indexed, but we want 0-based logic.
        let c = ((n % 26) as u8 + b'A') as char;
        result.insert(0, c);
        n /= 26;
    }
    result
}

#[derive(Debug, Clone, Default, Copy)]
pub struct TotalStatus {
	pub sem: bool,
	pub age: bool,
	pub site: bool,
}
impl TotalStatus {
	pub fn new(site: bool, age: bool, sem: bool) -> Self {
		Self { sem, age, site }
	}
	pub fn tup(&self) -> (bool, bool, bool) {
		(self.site, self.age, self.sem)
	}
}

#[derive(Debug, Clone, Default)]
pub struct SemCols {
	pub cap_col: u16,
	pub cap_col_str: String,
	pub insc_col: u16,
	pub insc_col_str: String,
	pub ann_col: u16,
	pub ann_col_str: String,
	pub att_col: u16,
	pub att_col_str: String,
	pub site_row_xlsx: u32,
	pub cat_row_xlsx: u32,
	pub cat_col_str: String,
	pub pinsc_col: u16,
	pub pann_col: u16,
	pub sem_start_row: u32,
}
impl SemCols {
	pub fn new(at_cat: u16, at_site: u32) -> Self {
		let cap_col = at_cat; // + 0
		let cap_col_str = to_excel_column(cap_col + 1);
		let insc_col = at_cat + 1;
		let insc_col_str = to_excel_column(insc_col + 1);
		let ann_col = at_cat + 2;
		let ann_col_str = to_excel_column(ann_col + 1);
		let att_col = at_cat + 3;
		let att_col_str = to_excel_column(att_col + 1);
		let site_row_xlsx = at_site+1;
		let cat_row_xlsx = at_site+1+1;
		let cat_col_str = to_excel_column(at_cat + 1);
		let pinsc_col = at_cat + 4;
		let pann_col = at_cat + 5;
		let sem_start_row = at_site + 3+1;
		Self {
			cap_col, cap_col_str, insc_col, insc_col_str, ann_col, ann_col_str, att_col, att_col_str, site_row_xlsx, cat_row_xlsx, cat_col_str, pinsc_col, pann_col, sem_start_row,
		}
	}
}

pub fn write_xlsx_row(stats_sheet: &mut Worksheet, sem_stats: &SemStats, percent_format: &Format, sem_row: u32, semcols: &SemCols, total: TotalStatus) -> Result<(), XlsxError> {
	match total.tup() {
		(total_site, total_age, false) => {
			let sem_row_xlsx = sem_row+1;
			let mut conds: Vec<String> = vec![];
			if !total_site {
				conds.push(format!("(groupes[Site]=$A${site})", site=semcols.site_row_xlsx));
			}
			if !total_age {
				conds.push(format!("(groupes[Categorie]=${cat_col}${cat_row})", cat_col=semcols.cat_col_str, cat_row=semcols.cat_row_xlsx));
			}
			conds.push(format!("(groupes[Semaine]=$A{sem_row})", sem_row=sem_row_xlsx));
			let cond_str = conds.join("*");
			// base data
			stats_sheet.write_formula(sem_row, semcols.cap_col, Formula::new(format!("SUM(FILTER(groupes[Capacite], {conds}, 0))", conds=cond_str)).set_result(sem_stats.capacite.to_string()))?;
			stats_sheet.write_formula(sem_row, semcols.insc_col, Formula::new(format!("SUM(FILTER(groupes[Inscriptions], {conds}, 0))", conds=cond_str)).set_result(sem_stats.inscriptions.to_string()))?;
			stats_sheet.write_formula(sem_row, semcols.ann_col, Formula::new(format!("SUM(FILTER(groupes[Annulations], {conds}, 0))", conds=cond_str)).set_result(sem_stats.annulations.to_string()))?;
			stats_sheet.write_formula(sem_row, semcols.att_col, Formula::new(format!("SUM(FILTER(groupes[Attente], {conds}, 0))", conds=cond_str)).set_result(sem_stats.liste_attente.to_string()))?;
			// percent data
			stats_sheet.write_formula_with_format(sem_row, semcols.pinsc_col, Formula::new(format!("IF({cap_col}{row}=0, 0, {insc_col}{row}/{cap_col}{row})", row=sem_row_xlsx, cap_col=semcols.cap_col_str, insc_col=semcols.insc_col_str)).set_result((if sem_stats.capacite == 0 {0.0} else {sem_stats.inscriptions as f64/sem_stats.capacite as f64}).to_string()), percent_format)?;
			stats_sheet.write_formula_with_format(sem_row, semcols.pann_col, Formula::new(format!("IF(({insc_col}{row}+{ann_col}{row})=0, 0, {ann_col}{row}/({insc_col}{row}+{ann_col}{row}))", row=sem_row_xlsx, insc_col=semcols.insc_col_str, ann_col=semcols.ann_col_str)).set_result((if (sem_stats.inscriptions + sem_stats.annulations) == 0 {0.0} else {sem_stats.annulations as f64/(sem_stats.inscriptions + sem_stats.annulations) as f64}).to_string()), percent_format)?;
		},
		(_, _, true) => {
			let total_row_xlsx = sem_row + 1;
			let sem_end_row = sem_row;
			stats_sheet.write_formula(sem_row, semcols.cap_col, Formula::new(format!("SUM({col}{start}:{col}{end})", col=semcols.cap_col_str, start=semcols.sem_start_row, end=sem_end_row)).set_result(sem_stats.capacite.to_string()))?;
			stats_sheet.write_formula(sem_row, semcols.insc_col, Formula::new(format!("SUM({col}{start}:{col}{end})", col=semcols.insc_col_str, start=semcols.sem_start_row, end=sem_end_row)).set_result(sem_stats.inscriptions.to_string()))?;
			stats_sheet.write_formula(sem_row, semcols.ann_col, Formula::new(format!("SUM({col}{start}:{col}{end})", col=semcols.ann_col_str, start=semcols.sem_start_row, end=sem_end_row)).set_result(sem_stats.annulations.to_string()))?;
			stats_sheet.write_formula(sem_row, semcols.att_col, Formula::new(format!("SUM({col}{start}:{col}{end})", col=semcols.att_col_str, start=semcols.sem_start_row, end=sem_end_row)).set_result(sem_stats.liste_attente.to_string()))?;
			// percent data
			stats_sheet.write_formula_with_format(sem_row, semcols.pinsc_col, Formula::new(format!("IF({cap_col}{row}=0, 0, {insc_col}{row}/{cap_col}{row})", row=total_row_xlsx, cap_col=semcols.cap_col_str, insc_col=semcols.insc_col_str)).set_result((if sem_stats.capacite == 0 {0.0} else {sem_stats.inscriptions as f64/sem_stats.capacite as f64}).to_string()), percent_format)?;
			stats_sheet.write_formula_with_format(sem_row, semcols.pann_col, Formula::new(format!("IF(({insc_col}{row}+{ann_col}{row})=0, 0, {ann_col}{row}/({insc_col}{row}+{ann_col}{row}))", row=total_row_xlsx, insc_col=semcols.insc_col_str, ann_col=semcols.ann_col_str)).set_result((if (sem_stats.inscriptions + sem_stats.annulations) == 0 {0.0} else {sem_stats.annulations as f64/(sem_stats.inscriptions + sem_stats.annulations) as f64}).to_string()), percent_format)?;
		},
	}
	Ok(())
}

pub struct CatBlock<'a> {
	pub percent_format: &'a Format,
	pub stats: &'a Stats,
	pub ustats: &'a UniqueStats,
	pub site: &'a str,
	pub cat: &'a str,
	pub cols: &'a [String],
	pub at_cat: u16,
	pub at_site: u32,
	pub total: TotalStatus,
}
impl CatBlock<'_> {
	pub fn write(self, worksheet: &mut Worksheet) -> Result<(), XlsxError> {
		let (total_site, total_age, _) = self.total.tup();
		{
			worksheet.merge_range(self.at_site+1, self.at_cat, self.at_site+1, self.at_cat + self.cols.len() as u16 - 1, self.cat, &XLSX_CAT_FORMAT)?;
			//stats_sheet.write(at_site + 1, at_cat, cat)?;
			worksheet.write_row(self.at_site + 2, self.at_cat, self.cols)?;

			let semcols = SemCols::new(self.at_cat, self.at_site);

			for (j, sem) in self.ustats.sems.iter().enumerate() {
				let sem_row = self.at_site + 3 + j as u32;
				let sem_stats = self.stats.sites.get(self.site).and_then(|s| s.ages.get(self.cat).and_then(|a| a.semaines.get(sem))).copied().unwrap_or_default();
				write_xlsx_row(worksheet, &sem_stats, self.percent_format, sem_row, &semcols, TotalStatus::new(total_site, total_age, false))?;
			}
			// ligne de total
			{
				let total_row = self.at_site + 3 + self.ustats.sems.len() as u32;
				let total_stats = self.stats.sites.get(self.site).and_then(|s| s.ages.get(self.cat)).map(|a| a.total()).unwrap_or_default();
				write_xlsx_row(worksheet, &total_stats, self.percent_format, total_row, &semcols, TotalStatus::new(total_site, total_age, true))?;
			}
		}
		// un peu de formatage
		worksheet.set_range_format(self.at_site+2, self.at_cat, self.at_site+2, self.at_cat+4, &XLSX_COL_FORMAT)?;
		Ok(())
	}
}

pub struct SiteBlock<'a> {
	pub percent_format: &'a Format,
	pub stats: &'a Stats,
	pub ustats: &'a UniqueStats,
	pub site: &'a str,
	pub cols: &'a [String],
	pub at_site: u32,
	pub total: TotalStatus,
}
impl SiteBlock<'_> {
	pub fn write(self, worksheet: &mut Worksheet) -> Result<(), XlsxError> {
		let (total_site, _, _) = self.total.tup();
		let mut age_col = self.ustats.ages.keys().cloned().collect::<Vec<_>>();
		age_col.sort();
		let full_range = 6*(self.ustats.ages.len() + 1) as u16;
		worksheet.merge_range(self.at_site, 0, self.at_site, full_range, self.site, &XLSX_SITE_FORMAT)?;
		//stats_sheet.write(at_site, 0, site)?;
		worksheet.write(self.at_site + 2, 0, "Semaine")?;
		worksheet.write_column(self.at_site + 3, 0, &self.ustats.sems)?;
		worksheet.write(self.at_site + 3 + self.ustats.sems.len() as u32, 0, "Total")?;
		for (i, cat) in age_col.iter().enumerate() {
			let at_cat = 1 + (i*self.cols.len()) as u16;
			CatBlock {
				percent_format: self.percent_format,
				stats: self.stats,
				ustats: self.ustats,
				site: self.site,
				cat,
				cols: self.cols,
				at_cat,
				at_site: self.at_site,
				total: TotalStatus::new(total_site, true, false),
			}.write(worksheet)?;

			// un peu de formatage
			worksheet.set_range_format(self.at_site+3, at_cat+5, self.at_site+3+self.ustats.sems.len() as u32, at_cat+5, &XLSX_SEP_FORMAT)?;
			worksheet.set_cell_format(self.at_site+2, at_cat+5, &XLSX_COL_SEP_FORMAT)?;
		}
		// petit bloc de total
		{
			let i = age_col.len();
			let at_cat = 1 + (i*self.cols.len()) as u16;
			CatBlock {
				percent_format: self.percent_format,
				stats: self.stats,
				ustats: self.ustats,
				site: self.site,
				cat: "Total",
				cols: self.cols,
				at_cat,
				at_site: self.at_site,
				total: TotalStatus::new(total_site, true, false),
			}.write(worksheet)?;
		}
		Ok(())
	}
}

fn chart_occupation_par_categorie(sheet: &mut Worksheet, stats: &Stats, ustats: &UniqueStats, row: u32, col: u16) -> Result<(), XlsxError> {
	// le défi c'est de trouvé le range de données pour chaque cat
	let mut chart = Chart::new(ChartType::Line);
	let mut cats = ustats.ages.keys().cloned().collect::<Vec<String>>();
	cats.sort();
	let at_site_total = (ustats.sems.len() as u32 + 4 + XLSX_SITE_BLOCK_SPACE) * ustats.sites.len() as u32;
	for (i, _cat) in cats.iter().enumerate() {
		let at_cat = 1 + (i * 6) as u16;
		let range_start = at_site_total +3;
		let range_col = at_cat + 4; // la colonne des pourcentages d'occupation
		let range_end = range_start + ustats.sems.len() as u32 - 1;
		chart.add_series()
			.set_values(("Stats", range_start, range_col, range_end, range_col))
			.set_categories(("Stats", range_start, 0, range_end, 0))
			.set_name(("Stats", at_site_total + 1, at_cat));
	}
	chart.set_name("Taux d'occupation par groupe d'âge");
	chart.title().set_name("Taux d'occupation par groupe d'âge");
	chart.x_axis().set_name("Semaine");
	chart.y_axis().set_name("Taux d'occupation");

	sheet.insert_chart(row, col, &chart)?;
	Ok(())
}

fn chart_occupation_annulations_par_semaine(sheet: &mut Worksheet, stats: &Stats, ustats: &UniqueStats, row: u32, col: u16) -> Result<(), XlsxError> {
	let mut chart = Chart::new(ChartType::Column);
	let at_site_total = (ustats.sems.len() as u32 + 4 + XLSX_SITE_BLOCK_SPACE) * ustats.sites.len() as u32;
	let at_cat_total = ustats.ages.len() as u16 * 6 + 1; // 6 colonnes par catégorie + 1 pour les semaines
	let row_start = at_site_total + 3;
	let row_end = row_start + ustats.sems.len() as u32 - 1;
	chart.add_series()
		.set_values(("Stats", row_start, at_cat_total + 4, row_end, at_cat_total + 4)) // pourcentage d'occupation
		.set_categories(("Stats", row_start, 0, row_end, 0))
		.set_name("Taux d'occupation");
	chart.add_series()
		.set_values(("Stats", row_start, at_cat_total + 5, row_end, at_cat_total + 5)) // pourcentage d'annulation
		.set_categories(("Stats", row_start, 0, row_end, 0))
		.set_name("Taux d'annulation");
	chart.set_name("Taux d'occupation et d'annulation par semaine");
	chart.title().set_name("Taux d'occupation et d'annulation par semaine");
	chart.x_axis().set_name("Semaine");

	sheet.insert_chart(row, col, &chart)?;
	Ok(())
}

fn chart_enfants_uniques_par_site(sheet: &mut Worksheet, stats: &Stats, ustats: &UniqueStats, row: u32, col: u16) -> Result<(), XlsxError> {
	let mut chart = Chart::new(ChartType::Column);
	chart.set_name("Enfants uniques par site");
	chart.title().set_name("Enfants uniques par site");
	chart.x_axis().set_name("Site");
	chart.y_axis().set_name("Nombre d'enfants uniques");
	chart.add_series()
		.set_values(("Donnees", 1, 11, 1+ustats.sites.len() as u32, 11))
		.set_categories(("Donnees", 1, 10, 1+ustats.sites.len() as u32, 10))
		.set_data_label(ChartDataLabel::new().show_value());

	sheet.insert_chart(row, col, &chart)?;
	Ok(())
}

fn chart_occupation_par_site(sheet: &mut Worksheet, stats: &Stats, ustats: &UniqueStats, row: u32, col: u16) -> Result<(), XlsxError> {
	let mut chart = Chart::new(ChartType::Line);
	chart.set_name("Taux d'occupation par site");
	chart.title().set_name("Taux d'occupation par site");
	chart.x_axis().set_name("Semaine");
	chart.y_axis().set_name("Taux d'occupation");

	let at_cat_total = ustats.ages.len() as u16 * 6 + 1; // 6 colonnes par catégorie + 1 pour les semaines
	let mut sites = ustats.sites.keys().cloned().collect::<Vec<String>>();
	sites.sort();
	for (i, site) in sites.iter().enumerate() {
		let at_site = (i as u32)*(4 + ustats.sems.len() as u32 + XLSX_SITE_BLOCK_SPACE);
		let row_start = at_site + 3;
		let row_end = row_start + ustats.sems.len() as u32 - 1;
		let range_col = at_cat_total + 4; // la colonne des pourcentages d'occupation
		chart.add_series()
			.set_values(("Stats", row_start, range_col, row_end, range_col))
			.set_categories(("Stats", row_start, 0, row_end, 0))
			.set_name(("Stats", at_site, 0)); // le nom du site est dans la colonne 0
	}

	sheet.insert_chart(row, col, &chart)?;
	Ok(())
}

fn chart_proportion_ville(sheet: &mut Worksheet, stats: &Stats, ustats: &UniqueStats, row: u32, col: u16) -> Result<(), XlsxError> {
	let mut chart = Chart::new(ChartType::Pie);
	chart.set_name("Proportion des villes");
	chart.title().set_name("Proportion des villes");
	chart.x_axis().set_name("Ville");

	chart.add_series()
		.set_values(("Donnees", ustats.sites.len() as u32 + 1, 17, ustats.sites.len() as u32 + 1, 21))
		.set_categories(("Donnees", 0, 17, 0, 21));
		
	sheet.insert_chart(row, col, &chart)?;
	Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ville {
	Longueuil,
	StHubert,
	GreenfieldPark,
}
impl Ville {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Longueuil => "Longueuil",
			Self::StHubert => "Saint-Hubert",
			Self::GreenfieldPark => "Greenfield Park",
		}
	}
	pub fn code_postaux(&self) -> &'static [&'static str] {
		match self {
			Self::Longueuil => &["J4G", "J4H", "J4J", "J4K", "J4L", "J4M", "J4N", "J4P",],
			Self::StHubert => &["J3Y", "J3Z", "J4T",],
			Self::GreenfieldPark => &["J4R", "J4V",],
		}
	}
	pub fn get_from_code_postal(code: CodePostal) -> Option<Self> {
		let code_part = code.as_str().get(0..3)?;
		[Self::Longueuil, Self::StHubert, Self::GreenfieldPark].into_iter().find(|&ville| ville.code_postaux().contains(&code_part))
	}
}
impl Display for Ville {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
#[derive(Debug, Clone, Default)]
pub struct VilleStats {
	pub villes: HashMap<Ville, u32>,
	pub autres: u32,
	pub inconnues: u32,
	pub total: u32,
}