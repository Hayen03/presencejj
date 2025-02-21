#let exists(var) = {
  var != none and var != "" and var != [] and var != () and var != (:) and not (type(var) == str and var.trim() == "")
}
#let hr(fill: black) = {
  set block(above: 5pt, below: 5pt)
  line(length: 100%, stroke: fill)
}
#let print_bool(val) = {
	if not exists(val) []
	else if val [OUI]
	else [NON]
}
#let ansline() = line(start:(2%, 0.8em), end: (98%, 0.8em), stroke: 0.4pt)
#let als(amount) = {
  for _ in range(amount) {
    ansline()
  }
}
#let ila() = box(line(start:(2%, 0.8em), end: (98%, 0.8em), stroke: 0.4pt), width: 1fr)

#let new_naissance(an: none, mois: none, jour: none) = (
	an: an,
	mois: mois,
	jour: jour,
)
#let new_cam(nam: none, exp_mois: none, exp_year: none,) = (
	nam: nam,
	exp_mois: exp_mois,
	exp_year: exp_year,
)
#let bool_just(val: none, just: none) = (
	val: val,
	just: just,
)
#let new_compte(mandataire: none, tel: none, email: none, adresse: none,) = (
	mandataire: mandataire,
	tel: tel,
	email: email,
	adresse: adresse,
)
#let new_medicaments(anti_inflamatoire: none, sirop_toux: none, ibuprofene: none, antiemetique: none, antibiotique: none, acetaminophene: none,) = (
	anti_inflamatoire: anti_inflamatoire,
	sirop_toux: sirop_toux,
	ibuprofene: ibuprofene,
	antiemetique: antiemetique,
	antibiotique: antibiotique,
	acetaminophene: acetaminophene,
)
#let new_piscine(auth_partage: none, vfi: none, tete_sous_eau: none) = (
	auth_partage: auth_partage,
	vfi: vfi,
	tete_sous_eau: tete_sous_eau,
)
#let new_contact(nom: none, tel: none, lien: none) = (
	nom: nom,
	tel: tel,
	lien: lien,
)

#let new_enfant(
	id: none,
	nom: none,
	prenom: none,
	naissance: none,
	age: none,
	cam: new_cam(),
	genre: none,
	allergies: (),
	maladies: (),
	prob_comportement: bool_just,
	auth_soins: none,
	compte: new_compte(),
	prise_med: none,
	medicaments: new_medicaments(),
	contact_1: none,
	contact_2: none,
	quitte: (),
	mdp: none,
	piscine: new_piscine(),
	auth_photo: none,
	commentaire: none,
) = (
	id: id,
	nom: nom,
	prenom: prenom,
	naissance: naissance,
	age: age,
	cam: cam,
	genre: genre,
	allergies: allergies,
	maladies: maladies,
	prob_comportement: prob_comportement,
	auth_soins: auth_soins,
	compte: compte,
	prise_med: prise_med,
	medicaments: medicaments,
	contact_1: contact_1,
	contact_2: contact_2,
	quitte: quitte,
	mdp: mdp,
	piscine: piscine,
	auth_photo: auth_photo,
	commentaire: commentaire,
)

#let new_groupe(saison: none, site: none, categorie: none, discriminant: none, animateur: none, semaine: none, activite: none, profil: none, groupe_num: none) = (
	saison: saison,
	site: site,
	categorie: categorie,
	discriminant: discriminant,
	animateur: animateur,
	semaine: semaine,
	activite: activite,
	profil: profil,
	groupe_num: groupe_num,
)

#let fiche_med(doc, 
	enfant: new_enfant(),
) = [

	#show heading.where(depth: 1): set text(size: 28pt)
	#show heading.where(depth: 2): set text(size: 18pt)
	#show heading.where(depth: 3): set text(size: 14pt)

	#grid(columns: (1fr, auto))[
		#align(left + bottom)[= Fiche Santé]
	][
		#image("doc_skia.png", width: 2.5in)
	]
	#hr()

	== #enfant.nom, #enfant.prenom

	#grid(columns: (1fr, 1fr))[
		/ Date de naissance: #enfant.naissance (#enfant.age ans)
		/ Genre: #enfant.genre
		/ Assurance Maladie: #enfant.cam.nam #enfant.cam.exp_mois;/#enfant.cam.exp_year
	][
		/ Mandataire: #enfant.compte.mandataire
		/ Telephone: #enfant.compte.tel
		/ Adresse: #enfant.compte.adresse
		/ Courriel: #enfant.compte.email
	]

	#v(1fr)

	#grid(columns: (1fr, 1fr))[
		/ Authorisation de soigner: #if exists(enfant.auth_soins) {print_bool(enfant.auth_soins)}
		#let tc = if exists(enfant.prob_comportement) {print_bool(enfant.prob_comportement.val)}
		/ Trouble de comportement: #if exists(enfant.prob_comportement) [#tc#if exists(tc) and exists(enfant.prob_comportement.just) [, ]#if exists(enfant.prob_comportement.just) {enfant.prob_comportement.just}]
		#let pm = if exists(enfant.prise_med) {print_bool(enfant.prise_med.val)}
		/ Prise de médicament: #if exists(enfant.prise_med) [#pm#if exists(pm) and exists(enfant.prise_med.just) [, ]#if exists(enfant.prise_med.just) {enfant.prise_med.just}]
	][
		/ Allergies: #enfant.allergies.join(", ")
		/ Maladies: #enfant.maladies.join(", ")
	]

	#v(1fr)

	=== Médicament authorisés
	#grid(columns: (1fr, 1fr, 1fr), row-gutter: 0.5em)[
		/ Acetaminophene: #print_bool(enfant.medicaments.acetaminophene)
	][
		/ Antibiotique: #print_bool(enfant.medicaments.antibiotique)
	][
		/ Anti-emetique: #print_bool(enfant.medicaments.antiemetique)
	][
		/ Anti-inflamatoire: #print_bool(enfant.medicaments.anti_inflamatoire)
	][
		/ Ibuprofene: #print_bool(enfant.medicaments.ibuprofene)
	][
		/ Sirop pour la toux: #print_bool(enfant.medicaments.sirop_toux)
	]

	#v(1fr)

	#grid(columns: (1fr, 1fr))[
		=== Contact d'urgence 1
		/ Nom: #if exists(enfant.contact_1) {enfant.contact_1.nom}
		/ Telephone: #if exists(enfant.contact_1) {enfant.contact_1.tel}
		/ Lien: #if exists(enfant.contact_1) {enfant.contact_1.lien}
	][
		=== Contact d'urgence 2
		/ Nom: #if exists(enfant.contact_2) {enfant.contact_2.nom}
		/ Telephone: #if exists(enfant.contact_2) {enfant.contact_2.tel}
		/ Lien: #if exists(enfant.contact_2) {enfant.contact_2.lien}
	]

	#v(1fr)

	#grid(columns: (1fr, 1fr))[
		=== Piscine
		/ Authorisation de partage: #print_bool(enfant.piscine.auth_partage)
		/ VFI obligatoire: #print_bool(enfant.piscine.vfi)
		/ Peut mettre sa tête sous l'eau: #print_bool(enfant.piscine.tete_sous_eau)
	][
		=== Autre
		/ Quitte avec: #enfant.quitte.join(", ")
		/ Mot de passe: #enfant.mdp
		/ Authorisation photo: #print_bool(enfant.auth_photo)
		/ Commentaire: #enfant.commentaire
	]

	#v(1fr)

	#align(bottom, grid(columns: (1fr, 1fr))[
		/ Signature: #ila()
	][])
]

#let check_cell_size = 0.8cm
#let mk_presence_anim_row(num: none, enfant: new_enfant()) = (
	table.cell(rowspan:2, breakable: false, align(center+horizon)[#num]),
	[#enfant.nom, #enfant.prenom],
	[#enfant.naissance (#enfant.age ans)],
	table.cell(rowspan: 2)[], table.cell(rowspan: 2)[], table.cell(rowspan: 2)[], 
	table.cell(rowspan: 2)[], table.cell(rowspan: 2)[], table.cell(rowspan: 2)[], 
	table.cell(rowspan: 2)[], table.cell(rowspan: 2)[], table.cell(rowspan: 2)[], 
	table.cell(rowspan: 2)[], table.cell(rowspan: 2)[], table.cell(rowspan: 2)[], 
	table.cell(rowspan: 2)[], table.cell(rowspan: 2)[], table.cell(rowspan: 2)[],
	table.cell(colspan: 2)[
		#let bloc = ()
		#if exists(enfant.allergies) {bloc.push[*Allergies:* #enfant.allergies.join(", ")]}
		#if exists(enfant.compte.mandataire) {bloc.push[*Mandataire:* #enfant.compte.mandataire#if exists(enfant.compte.tel) [, #enfant.compte.tel]]}
		#bloc.filter(it => exists(it)).join("; ")
	],
)
#let color_anim_table(col, row) = {
	if row == 0 {
		return white
	}
	let n = calc.rem(row - 1, 4)
	if n < 2 {
		rgb("E0E0E0")
	} else {
		white
	}
}
#let presence_anim(doc, groupe: new_groupe(), participants: ()) = [
	#set page(paper: "us-letter", flipped: true, margin: 1cm)

	#show heading.where(depth: 1): set text(size: 24pt)
	#show heading.where(depth: 2): set text(size: 18pt)
	#show heading.where(depth: 3): set text(size: 14pt)

	#set table(fill: color_anim_table)

	#grid(columns: (1fr, auto))[
		#let ln = (
			//if exists(groupe.saison) [#groupe.saison],
			if exists(groupe.activite) [#groupe.activite],
			if exists(groupe.site) [#groupe.site],
			if exists(groupe.categorie) [#groupe.categorie],
			if exists(groupe.semaine) [sem. #groupe.semaine],
		).filter(it => exists(it))
		= #ln.join(" | ") 
		#let ln = (
			if exists(groupe.discriminant) [#groupe.discriminant],
			if exists(groupe.groupe_num) [#groupe.groupe_num],
			if exists(groupe.profil) [profil #groupe.profil],
			if exists(groupe.animateur) [(#groupe.animateur)],
		).filter(it => exists(it))
		#if ln.len() > 0 [== #ln.join(" ")]
		== Liste de Présence Animateur
	][
		#align(center+horizon, image("doc_skia.png", width: 2.5in))
	]

	#let cells = ()
	// remplir les cellules des enfants
	#for (num, enf) in participants.enumerate() {
		cells = cells + mk_presence_anim_row(num: num, enfant: enf)
	}

	#table(columns: (auto, 1fr, auto, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size, check_cell_size),
	table.header(repeat: true, [*\#*], [*Nom, Prénom*], [], table.cell(colspan: 3, align(center, [*Lundi*])), table.cell(colspan: 3, align(center, [*Mardi*])), table.cell(colspan: 3, align(center, [*Mercredi*])), table.cell(colspan: 3, align(center, [*Jeudi*])), table.cell(colspan: 3, align(center, [*Vendredi*]))),
	..cells,
	)
]

#let sign_cell_size = 3*check_cell_size
#let mk_sdj_row(enfant: new_enfant(), groupe: new_groupe()) = (
	if exists(enfant.mdp) {table.cell(rowspan: 3, breakable: false, align(center+horizon)[#enfant.mdp])} else {table.cell(rowspan: 3, breakable: false)[]},
	table.cell(rowspan: 2)[#enfant.nom, #enfant.prenom],
	table.cell(rowspan: 2)[#enfant.naissance (#enfant.age ans); *Quitte Avec:* #enfant.quitte.join(", ")],
	[], [], [], [], [], [], [], [], [], [],
	table.cell(colspan: 7)[
		#let grp = (
			if exists(groupe.categorie) [#groupe.categorie],
			if exists(groupe.discriminant) [#groupe.discriminant],
			if exists(groupe.profil) [profil #groupe.profil],
			if exists(groupe.animateur) [(#groupe.animateur)],
		).filter(it => exists(it))
		#let bloc = (
			if exists(enfant.allergies) [*Allergies: * #enfant.allergies.join(", ")],
			if exists(enfant.compte.mandataire) [*Mandataire: * #enfant.compte.mandataire],
			if exists(grp) [*Groupe:* #grp.join(" ")],
		)
		#bloc.filter(it => exists(it)).join("; ")
	],
)
#let color_sdg_table(col, row) = {
	if row == 0 {
		return white
	}
	let n = calc.rem(row - 1, 6)
	if n < 3 {
		rgb("E0E0E0")
	} else {
		white
	}
}
#let presence_sdj(
	site: none,
	saison: none,
	semaine: none,
	groupes: (:),
	participants: (),
) = [
		#set page(paper: "us-letter", flipped: true, margin: 1cm)

	#show heading.where(depth: 1): set text(size: 24pt)
	#show heading.where(depth: 2): set text(size: 18pt)
	#show heading.where(depth: 3): set text(size: 14pt)

	#set table(fill: color_sdg_table)

	#grid(columns: (1fr, auto))[
		#let ln = (
			if exists(site) [#site],
			if exists(semaine) [sem. #semaine],
		).filter(it => exists(it))
		= #ln.join(" | ") 
		== Liste de Présence SDJ
	][
		#align(center+horizon, image("doc_skia.png", width: 2.5in))
	]

	#let cells = ()
	#for membre in participants {
		cells = cells + mk_sdj_row(enfant: membre, groupe: groupes.at(membre.id))
	}

	#table(columns: (2cm, 4cm, 1fr, sign_cell_size, sign_cell_size, sign_cell_size, sign_cell_size, sign_cell_size), rows: auto,
	table.header(repeat: true, align(center)[*MDP*], [*Nom, Prénom*], [], align(center)[*Lundi*], align(center)[*Mardi*], align(center)[*Mercredi*], align(center)[*Jeudi*], align(center)[*Vendredi*],
	),
	..cells
	)
]

/*
(
	if exists(enfant.mdp) {table.cell(rowspan: 3, breakable: false, align(center+horizon)[#enfant.mdp])} else [],
	table.cell(rowspan: 2)[#enfant.nom, #enfant.prenom],
	table.cell(rowspan: 2)[#enfant.naissance (#enfant.age ans); *Quitte Avec:* #enfant.quitte.join(", ")],
	[], [], [], [], [], [], [], [], [], [],
	table.cell(colspan: 7)[
		#let grp = (
			if exists(groupe.categorie) [#groupe.categorie],
			if exists(groupe.discriminant) [#groupe.discriminant],
			if exists(groupe.profil) [profil #groupe.profil],
			if exists(groupe.animateur) [(#groupe.animateur)],
		).filter(it => exists(it))
		#let bloc = (
			if exists(enfant.allergies) [*Allergies: * #enfant.allergies.join(", ")],
			if exists(enfant.compte.mandataire) [*Mandataire: * #enfant.compte.mandataire],
			if exists(grp) [*Groupe:* #grp.join(" ")],
		)
		#bloc.filter(it => exists(it)).join("; ")
	],
)
*/