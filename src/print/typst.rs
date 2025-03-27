use chrono::{Datelike, Local};

use crate::{config::Config, data::cam::CAM, groupes::{comptes::{Compte, CompteReg, NULL_COMPTE}, groupes::{Groupe, GroupeReg, SousGroupe}, membres::{Interet, Membre, MembreID, MembreReg}}};

use super::PrintError;
use core::str;
use std::{collections::HashSet, fmt::Display, fs::OpenOptions, io::Write, ops::BitAnd, process::Command};

enum Delimiter {
	Quotes,
	Brackets,
	Dollars,
	Parentheses,
	Braces,
	None,
}

pub fn po<T: Display>(obj: Option<T>, delimiter: Delimiter) -> String {
	match delimiter {
		Delimiter::Quotes => obj.map_or("none".into(), |s| format!("\"{}\"", s.to_string().replace("#", r"\#").replace("@", r"\@").replace("\"", "\\\""))),
		Delimiter::Brackets => obj.map_or("none".into(), |s| format!("[{}]", s.to_string().replace("#", r"\#").replace("@", r"\@").replace("$", r"\$"))),
		Delimiter::Dollars => obj.map_or("none".into(), |s| format!("${}$", s.to_string().replace("#", r"\#").replace("@", r"\@").replace("$", r"\$"))),
		Delimiter::Parentheses => obj.map_or("none".into(), |s| format!("({})", s.to_string().replace("#", r"\#").replace("@", r"\@"))),
		Delimiter::Braces => obj.map_or("none".into(), |s| format!("{{{}}}", s.to_string().replace("#", r"\#").replace("@", r"\@"))),
		Delimiter::None => obj.map_or("none".into(), |s| format!("{}", s.to_string().replace("#", r"\#").replace("@", r"\@"))),
	}
}

pub fn print_fiche_med(membre: &Membre, compte: &Compte, config: &Config, site: &str, update: bool) -> Result<(), PrintError> {
	// calcul le nom du fichier de sortie
	let dir = format!("{}/{}/fiche_med", config.out_dir, site);
	let out_file = format!("{}/fichemed_{}_{}.pdf", dir, membre.nom, membre.prenom);

	// make sure the directory exists
	let _ = std::fs::create_dir_all(dir);

	// ouvre le fichier temporaire
	let tmp_file_dir = format!("{}/templates", config.working_dir);
	let _ = std::fs::create_dir_all(&tmp_file_dir);
	let tmp_file_path = format!("{}/tmp.typ", tmp_file_dir);
	let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(&tmp_file_path).expect("Could not open temporary file");

	let _ = write!(file, 
"#import \"template.typ\": *
#let enfant = {mbr}
#show: it => fiche_med( it,
	enfant: enfant,
)
",
		mbr=mk_membre(membre, compte),
	);
	
	drop(file);
	
	let mut cmd = Command::new("typst");
	cmd
	.current_dir(&config.working_dir)
		.arg("compile")
		.arg(&tmp_file_path)
		.arg(&out_file);
	let output = cmd
			.output()
			.expect("failed to execute process");
	let err = unsafe {str::from_utf8_unchecked(&output.stderr)}.trim();
	if err.len() > 0 {
		println!("{}", err);
	}

	std::fs::remove_file(tmp_file_path).unwrap();
	Ok(())
}

pub fn print_presence_anim(groupe: &Groupe, sous_groupe: Option<&SousGroupe>, membres: &MembreReg, comptes: &CompteReg, config: &Config) -> Result<(), PrintError> {
	// calcul le nom du fichier de sortie
	let dir = format!("{out}/{saison}/{site}/anim/sem{semaine}", 
		out=config.out_dir, 
		site=groupe.site.as_ref().map(String::as_str).unwrap_or("none"), 
		saison=groupe.saison.as_ref().map(String::as_str).unwrap_or("none"),
		semaine=groupe.semaine.as_ref().map(String::as_str).unwrap_or("none"),
	).replace(" ", "-");
	let out_filename = format!("presence_anim_{activite}_{site}_{categorie}_{discriminant}{num}_{profil}_sem{semaine}.pdf", 
		site=groupe.site.as_ref().map(String::as_str).unwrap_or("none"),
		categorie=groupe.category.as_ref().map(String::as_str).unwrap_or("none"),
		discriminant=groupe.discriminant.as_ref().map(String::as_str).unwrap_or("none"),
		semaine=groupe.semaine.as_ref().map(String::as_str).unwrap_or("none"),
		activite=groupe.activite.as_ref().map(String::as_str).unwrap_or("none"),
		num=sous_groupe.map(|sg| sg.disc).as_ref().map(u32::to_string).unwrap_or("none".into()),
		profil=sous_groupe.map(|sg| sg.profil.as_ref()).unwrap_or(None).map(Interet::as_str).unwrap_or("none"),
	).replace(" ", "-");
	let out_file = format!("{dir}/{filename}", dir=dir, filename=out_filename);

	// make sure the directory exists
	let _ = std::fs::create_dir_all(dir);

	// ouvre le fichier temporaire
	let tmp_file_dir = format!("{}/templates", config.working_dir);
	let _ = std::fs::create_dir_all(&tmp_file_dir);
	let tmp_file_path = format!("{}/tmp.typ", tmp_file_dir);
	let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(&tmp_file_path).expect("Could not open temporary file");

	let _ = write!(file, 
"#import \"template.typ\": *
#let grp = {grp}
#let participants = (
",
		grp=mk_groupe(groupe, sous_groupe),
	);
	let participants = if let Some(sg) = sous_groupe {sg.participants.iter()} else {groupe.participants.iter()};
	let mut ps = participants.map(|id| membres.get(*id).expect("Participant non existant")).collect::<Vec<_>>();
	ps.sort_by(|m1, m2| m1.cmp_nom(m2));
	for membre in ps.iter() {
		//let membre = membres.get(*participant).expect("Participant non existant");
		let compte = membre.compte.map(|c| comptes.get(c).expect("Compte non existant")).unwrap_or(&NULL_COMPTE);
		let _ = write!(file, 
"{mbr},
", mbr=mk_membre(membre, compte));
	}
	let _ = write!(file, 
")
#show: it => presence_anim(it, groupe: grp, participants: participants)
");
	
	drop(file);
	
	let mut cmd = Command::new("typst");
	cmd
	.current_dir(&config.working_dir)
		.arg("compile")
		.arg(&tmp_file_path)
		.arg(&out_file);
	let output = cmd
			.output()
			.expect("failed to execute process");
	let err = unsafe {str::from_utf8_unchecked(&output.stderr)}.trim();
	if err.len() > 0 {
		println!("{}", err);
	}

	std::fs::remove_file(tmp_file_path).unwrap();
	println!("Wrote {}", out_file);
	Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Default, Hash, Ord)]
pub struct PresenceSDJInfo<'a> {
	pub site: Option<&'a str>,
	pub semaine: Option<&'a str>,
	pub saison: Option<&'a str>,
}
fn filter_grp(grp: &Groupe, info: &PresenceSDJInfo) -> bool {
	grp.saison.as_ref().map(String::as_str) == info.saison &&
	grp.site.as_ref().map(String::as_str) == info.site &&
	grp.semaine.as_ref().map(String::as_str) == info.semaine
}
fn get_mbr_sg(mid: MembreID, grp: &Groupe) -> Option<&SousGroupe> {
	for sg in grp.sous_groupe.iter() {
		if sg.participants.contains(&mid) {
			return Some(sg);
		}
	}
	return None;
}
pub fn print_presence_sdj(info: &PresenceSDJInfo, groupes: &GroupeReg, membres: &MembreReg, comptes: &CompteReg, config: &Config) -> Result<(), PrintError> {
	let out_dir = format!("{out}/{saison}/{site}/sdj", 
		out=config.out_dir, 
		site=info.site.unwrap_or("none"), 
		saison=info.saison.unwrap_or("none")
	).replace(" ", "-");
	let out_filename = format!("presence_sdj_{saison}_{site}_sem{semaine}.pdf", 
		site=info.site.unwrap_or("none"), 
		saison=info.saison.unwrap_or("none"), 
		semaine=info.semaine.unwrap_or("none")
	).replace(" ", "-");
	let out_file = format!("{dir}/{file}", dir=out_dir, file=out_filename);
	let _ = std::fs::create_dir_all(out_dir);

	// ouvre le fichier temporaire
	let tmp_file_dir = format!("{}/templates", config.working_dir);
	let _ = std::fs::create_dir_all(&tmp_file_dir);
	let tmp_file_path = format!("{}/tmp.typ", tmp_file_dir);
	let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(&tmp_file_path).expect("Could not open temporary file");

	let _ = write!(file,
"#import \"template.typ\": *
#let site = {site}
#let saison = {saison}
#let semaine = {semaine}
#let groupes = (
",
		site = po(info.site, Delimiter::Brackets),
		saison = po(info.saison, Delimiter::Brackets),
		semaine = po(info.semaine, Delimiter::Brackets),
	);
	
	let mut participants = HashSet::new();
	for grp in groupes.groupes().filter(|g| filter_grp(g, info) ) {
		for p in grp.participants.iter() {
			participants.insert(*p);
			let (profil, anim) = match get_mbr_sg(*p, grp) {
				None => (None, None),
				Some(sg) => (sg.profil.as_ref(), sg.animateur.as_ref()),
			};
			let cat = grp.category.as_ref();
			let disc = grp.discriminant.as_ref();
			let _ = write!(file, 
"\"{mid}\": new_groupe(categorie: {categorie}, discriminant: {discriminant}, animateur: {animateur}, profil: {profil}),\n",
				mid=p,
				categorie=po(cat, Delimiter::Brackets),
				discriminant=po(disc, Delimiter::Brackets),
				animateur=po(anim, Delimiter::Brackets),
				profil=po(profil, Delimiter::Brackets),
			);
		}
	}
	let _ = write!(file, 
")
#let participants = (
"
	);
	let mut participants = participants.into_iter().map(|mid| membres.get(mid).expect("Membre non existant")).collect::<Vec<_>>();
	participants.sort_by(|arg0: &&Membre, other: &&Membre| Membre::cmp_nom(*arg0, *other));
	for membre in participants.iter() {
		let compte = membre.compte.map(|c| comptes.get(c).expect("Compte non existant")).unwrap_or(&NULL_COMPTE);
		let _ = write!(file,"{},\n", mk_membre(membre, compte));
	}
	let _ = write!(file, 
")
#show: it => presence_sdj(
	site: site,
	saison: saison,
	semaine: semaine,
	groupes: groupes,
	participants: participants,
)");

	drop(file);

	let mut cmd = Command::new("typst");
	cmd
	.current_dir(&config.working_dir)
		.arg("compile")
		.arg(&tmp_file_path)
		.arg(&out_file);
	let output = cmd
			.output()
			.expect("failed to execute process");
	let err = unsafe {str::from_utf8_unchecked(&output.stderr)}.trim();
	if err.len() > 0 {
		println!("{}", err);
	}

	//std::fs::remove_file(tmp_file_path).unwrap();
	println!("Wrote {}", out_file);
	Ok(())
}

fn mk_membre(membre: &Membre, compte: &Compte) -> String {
	format!("new_enfant(
		id: \"{mid}\",
		nom: [{nom}],
		prenom: [{prenom}],
		cam: new_cam(nam: {nam}, exp_mois: {exp_mois}, exp_year: {exp_year}),
		genre: {genre},
		allergies: ({allergies}),
		maladies: ({maladies}),
		prob_comportement: {prob_comportement},
		compte: new_compte(mandataire: [{mandataire}], tel: {tel}, adresse: {adresse}, email: \"{email}\"),
		prise_med: {prise_med},
		auth_soins: {auth_soins},
		medicaments: new_medicaments(anti_inflamatoire: {med_anti_infl}, sirop_toux: {med_sirop}, ibuprofene: {med_ibu}, antiemetique: {med_antieme}, antibiotique: {med_antibio}, acetaminophene: {med_acet},),
		contact_1: {contact_1},
		contact_2: {contact_2},
		quitte: ({quitte}),
		mdp: {mdp},
		piscine: new_piscine(auth_partage: {auth_partage}, vfi: {vfi}, tete_sous_eau: {tse}),
		auth_photo: {auth_photo},
		commentaire: {comment},
		naissance: [{naissance}],
		age: {age},
	)",
			mid=membre.id,
			nom=membre.nom,
			prenom=membre.prenom,
			nam=po(membre.fiche_sante.cam.as_ref().map(CAM::numero), Delimiter::Brackets),
			exp_mois=po(membre.fiche_sante.cam.as_ref().map(|s| format!("{:02}", s.exp_mois())), Delimiter::Brackets),
			exp_year=po(membre.fiche_sante.cam.as_ref().map(|s| format!("{:04}", s.exp_an())), Delimiter::Brackets),
			genre=po(membre.genre, Delimiter::Brackets),
			allergies = match membre.fiche_sante.allergies.len() {
				0 => String::new(),
				1 => format!("[{}],", membre.fiche_sante.allergies[0]),
				_ => membre.fiche_sante.allergies.iter().map(|s| format!("[{}]", s)).collect::<Vec<String>>().join(", ")
			},
			maladies = match membre.fiche_sante.maladies.len() {
				0 => String::new(),
				1 => format!("[{}],", membre.fiche_sante.maladies[0]),
				_ => membre.fiche_sante.maladies.iter().map(|s| format!("[{}]", s)).collect::<Vec<String>>().join(", ")
			},
			prob_comportement = match &membre.fiche_sante.probleme_comportement {
				None => "none".into(),
				Some(bj) => format!("bool_just(val: {val}, just: {just})", val=bj.reponse, just=po(bj.justification.as_ref(), Delimiter::Brackets)),
			},
			mandataire=compte.mandataire,
			tel=po(compte.tel, Delimiter::Brackets),
			adresse=po(compte.adresse.as_ref().map(|adr| adr.full()), Delimiter::Brackets),
			email=po(compte.email.as_ref(), Delimiter::None),
			prise_med=match &membre.fiche_sante.prise_med {
				None => "none".into(),
				Some(bj) => format!("bool_just(val: {val}, just: {just})", val=bj.reponse, just=po(bj.justification.as_ref(), Delimiter::Brackets)),
			},
			auth_soins=po(membre.fiche_sante.auth_soins, Delimiter::None),
			med_anti_infl=po(membre.fiche_sante.auth_medicaments.anti_inflamatoire, Delimiter::None),
			med_sirop=po(membre.fiche_sante.auth_medicaments.sirop_toux, Delimiter::None),
			med_ibu=po(membre.fiche_sante.auth_medicaments.ibuprofene, Delimiter::None),
			med_antieme=po(membre.fiche_sante.auth_medicaments.anti_emetique, Delimiter::None),
			med_antibio=po(membre.fiche_sante.auth_medicaments.anti_biotique, Delimiter::None),
			med_acet=po(membre.fiche_sante.auth_medicaments.acetaminophene, Delimiter::None),
			contact_1=match &membre.contacts[0] {
				None => "none".into(),
				Some(c) => format!("new_contact(nom: [{nom}], tel: {tel}, lien: {lien})", nom=c.nom, tel=po(c.tel, Delimiter::Brackets), lien=po(c.lien.as_ref(), Delimiter::Brackets),),
			},
			contact_2=match &membre.contacts[1] {
				None => "none".into(),
				Some(c) => format!("new_contact(nom: [{nom}], tel: {tel}, lien: {lien})", nom=c.nom, tel=po(c.tel, Delimiter::Brackets), lien=po(c.lien.as_ref(), Delimiter::Brackets)),
			},
			quitte = match membre.quitte.avec.len() {
				0 => String::new(),
				1 => format!("[{}],", membre.quitte.avec[0]),
				_ => membre.quitte.avec.iter().map(|s| format!("[{}]", s)).collect::<Vec<String>>().join(", ")
			},
			mdp=po(membre.quitte.mdp.as_ref(), Delimiter::Brackets),
			auth_partage=po(membre.piscine.partage, Delimiter::None),
			vfi=po(membre.piscine.vfi, Delimiter::None),
			tse=po(membre.piscine.tete_sous_eau, Delimiter::None),
			auth_photo=po(membre.auth_photo, Delimiter::None),
			comment=po(membre.commentaire.as_ref(), Delimiter::Brackets),
			naissance=format!("{an:04}/{mois:02}/{jour:02}", an=membre.naissance.year(), mois=membre.naissance.month0()+1, jour=membre.naissance.day0()+1),
			age=po(Local::now().date_naive().years_since(membre.naissance), Delimiter::Brackets),
		)
}

fn mk_groupe(groupe: &Groupe, sous_groupe: Option<&SousGroupe>) -> String {
	format!("new_groupe(saison: {saison}, site: {site}, categorie: {categorie}, discriminant: {discriminant}, animateur: {animateur}, semaine: {semaine}, activite: {activite}, profil: {profil}, groupe_num: {groupe_num})",
	saison=po(groupe.saison.as_ref(), Delimiter::Brackets),
	site=po(groupe.site.as_ref(), Delimiter::Brackets),
	categorie=po(groupe.category.as_ref().map(String::as_str), Delimiter::Brackets),
	discriminant=po(groupe.discriminant.as_ref(), Delimiter::Brackets),
	animateur=po(sous_groupe.map(|sg| sg.animateur.as_ref()).unwrap_or(None).map(String::as_str), Delimiter::Brackets),
	semaine=po(groupe.semaine.as_ref(), Delimiter::Brackets),
	activite=po(groupe.activite.as_ref(), Delimiter::Brackets),
	groupe_num=po(sous_groupe.map(|sg| sg.disc).as_ref().map(u32::to_string), Delimiter::Brackets),
	profil=po(sous_groupe.map(|sg| sg.profil.as_ref()).unwrap_or(None).map(Interet::as_str), Delimiter::Brackets),
	)
}