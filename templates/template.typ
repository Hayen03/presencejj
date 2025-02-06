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