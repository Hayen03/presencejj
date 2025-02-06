use chrono::{Datelike, Local};

use crate::{config::Config, data::cam::CAM, groupes::{comptes::{Compte, CompteReg}, groupes::Groupe, membres::{Membre, MembreReg}}};

use super::PrintError;
use core::str;
use std::{fmt::Display, fs::OpenOptions, io::Write, process::Command};

pub fn po<T: Display>(obj: Option<T>, brackets: bool) -> String {
	if brackets {
		obj.map_or("none".into(), |s| format!("[{}]", s))
	} else {
		obj.map_or("none".into(), |s| format!("{}", s))
	}
}

pub fn print_fiche_med(membre: &Membre, compte: &Compte, config: &Config, site: &str, update: bool) -> Result<(), PrintError> {
	// calcul le nom du fichier de sortie
	let dir = format!("{}/{}", config.out_dir, site);
	let out_file = format!("{}/fiche_med/fichemed_{}_{}.pdf", dir, membre.nom, membre.prenom);

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
	.current_dir("/Users/leojetzer/Documents/presencejj")
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

pub fn print_presence_anim(groupe: &Groupe, membres: &MembreReg, comptes: &CompteReg, out_dir: String) -> Result<(), PrintError> {
	Ok(())
}

fn mk_membre(membre: &Membre, compte: &Compte) -> String {
	format!("new_enfant(
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
		mdp: \"{mdp}\",
		piscine: new_piscine(auth_partage: {auth_partage}, vfi: {vfi}, tete_sous_eau: {tse}),
		auth_photo: {auth_photo},
		commentaire: {comment},
		naissance: [{naissance}],
		age: {age},
	)
	#show: it => fiche_med( it,
		enfant: enfant,
	)
	",
			nom=membre.nom,
			prenom=membre.prenom,
			nam=po(membre.fiche_sante.cam.as_ref().map(CAM::numero), true),
			exp_mois=po(membre.fiche_sante.cam.as_ref().map(|s| format!("{:02}", s.exp_mois())), true),
			exp_year=po(membre.fiche_sante.cam.as_ref().map(|s| format!("{:04}", s.exp_an())), true),
			genre=po(membre.genre, true),
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
				Some(bj) => format!("bool_just(val: {val}, just: {just})", val=bj.reponse, just=po(bj.justification.as_ref(), true)),
			},
			mandataire=compte.mandataire,
			tel=po(compte.tel, true),
			adresse=po(compte.adresse.as_ref().map(|adr| adr.full()), true),
			email=po(compte.email.as_ref(), false),
			prise_med=match &membre.fiche_sante.prise_med {
				None => "none".into(),
				Some(bj) => format!("bool_just(val: {val}, just: {just})", val=bj.reponse, just=po(bj.justification.as_ref(), true)),
			},
			auth_soins=po(membre.fiche_sante.auth_soins, false),
			med_anti_infl=po(membre.fiche_sante.auth_medicaments.anti_inflamatoire, false),
			med_sirop=po(membre.fiche_sante.auth_medicaments.sirop_toux, false),
			med_ibu=po(membre.fiche_sante.auth_medicaments.ibuprofene, false),
			med_antieme=po(membre.fiche_sante.auth_medicaments.anti_emetique, false),
			med_antibio=po(membre.fiche_sante.auth_medicaments.anti_biotique, false),
			med_acet=po(membre.fiche_sante.auth_medicaments.acetaminophene, false),
			contact_1=match &membre.contacts[0] {
				None => "none".into(),
				Some(c) => format!("new_contact(nom: [{nom}], tel: {tel}, lien: {lien})", nom=c.nom, tel=po(c.tel, true), lien=po(c.lien.as_ref(), true),),
			},
			contact_2=match &membre.contacts[1] {
				None => "none".into(),
				Some(c) => format!("new_contact(nom: [{nom}], tel: {tel}, lien: {lien})", nom=c.nom, tel=po(c.tel, true), lien=po(c.lien.as_ref(), true)),
			},
			quitte = match membre.quitte.avec.len() {
				0 => String::new(),
				1 => format!("[{}],", membre.quitte.avec[0]),
				_ => membre.quitte.avec.iter().map(|s| format!("[{}]", s)).collect::<Vec<String>>().join(", ")
			},
			mdp=po(membre.quitte.mdp.as_ref(), false),
			auth_partage=po(membre.piscine.partage, false),
			vfi=po(membre.piscine.vfi, false),
			tse=po(membre.piscine.tete_sous_eau, false),
			auth_photo=po(membre.auth_photo, false),
			comment=po(membre.commentaire.as_ref(), true),
			naissance=format!("{an:04}/{mois:02}/{jour:02}", an=membre.naissance.year(), mois=membre.naissance.month0()+1, jour=membre.naissance.day0()+1),
			age=po(Local::now().date_naive().years_since(membre.naissance), true),
		)
}