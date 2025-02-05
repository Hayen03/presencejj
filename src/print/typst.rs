use chrono::{Datelike, Local};
use tempfile::{tempfile, NamedTempFile};

use crate::{data::cam::CAM, groupes::{comptes::Compte, membres::Membre}};

use super::PrintError;
use core::str;
use std::{fmt::Display, fs::File, io::Write, process::Command};

pub fn po<T: Display>(obj: Option<T>, brackets: bool) -> String {
	if brackets {
		obj.map_or("none".into(), |s| format!("[{}]", s))
	} else {
		obj.map_or("none".into(), |s| format!("{}", s))
	}
}

pub fn print_fiche_med(membre: &Membre, compte: &Compte, out_dir: String) -> Result<(), PrintError> {
	// calcul le nom du fichier de sortie
	let out_file = format!("{}/fichemed_{}_{}.pdf", out_dir, membre.nom, membre.prenom);

	// ouvre le fichier temporaire
	let mut file = match NamedTempFile::new_in("./templates") {
		Ok(f) => f,
		Err(_) => return Err(PrintError::TempFileError),
	};
	
	println!("{}", file.path().to_str().unwrap_or("ERROR"));

	let _ = write!(file, 
"#import \"template.typ\": *
#let enfant = new_enfant(
	nom: [{nom}],
	prenom: [{prenom}],
	cam: new_cam(nam: \"{nam}\", exp_mois: {exp_mois:02}, exp_year: {exp_year:04}),
	genre: {genre},
	allergies: ({allergies}),
	maladies: ({maladies}),
	prob_comportement: {prob_comportement},
	compte: new_compte(mandataire: [{mandataire}], tel: {tel}, adresse: {adresse}),
	prise_med: {prise_med},
	medicaments: new_medicaments(anti_inflamatoire: {med_anti_infl}, sirop_toux: {med_sirop}, ibuprofene: {med_ibu}, antiemetique: {med_antieme}, antibiotique: {med_antibio}, acetaminophene: {med_acet},),
	contact_1: {contact_1},
	contact_2: {contact_2},
	quitte: ({quitte}),
	mdp: {mdp},
	piscine: new_piscine(auth_partage: {auth_partage}, vfi: {vfi}, tete_sous_eau: {tse}),
	auth_photo: {auth_photo},
	commentaire: {comment},
	naissance: {naissance},
	age: {age},
)
#show: it => fiche_med( it,
	enfant: enfant,
)
",
		nom=membre.nom,
		prenom=membre.prenom,
		nam=po(membre.fiche_sante.cam.as_ref().map(CAM::numero), true),
		exp_mois=po(membre.fiche_sante.cam.as_ref().map(CAM::exp_mois), true),
		exp_year=po(membre.fiche_sante.cam.as_ref().map(CAM::exp_an), true),
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
		prise_med=match &membre.fiche_sante.probleme_comportement {
			None => "none".into(),
			Some(bj) => format!("bool_just(val: {val}, just: {just})", val=bj.reponse, just=po(bj.justification.as_ref(), true)),
		},
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
		mdp=po(membre.quitte.mdp.as_ref(), true),
		auth_partage=po(membre.piscine.partage, false),
		vfi=po(membre.piscine.vfi, false),
		tse=po(membre.piscine.tete_sous_eau, false),
		auth_photo=po(membre.auth_photo, false),
		comment=po(membre.commentaire.as_ref(), true),
		naissance=format!("{an:04}/{mois:02}/{jour:02}", an=membre.naissance.year(), mois=membre.naissance.month0()+1, jour=membre.naissance.day0()+1),
		age=po(Local::now().date_naive().years_since(membre.naissance), true),
	);
	
	
	let output = Command::new("typst")
			.arg("compile")
			.arg("--package-path")
			.arg("./templates")
			.arg(file.path().to_str().unwrap())
			.arg(out_file)
			.output()
			.expect("failed to execute process");

	println!("{}", unsafe {str::from_utf8_unchecked(&output.stderr)});

	let _ = file.close();
	Ok(())
}