#let exists(var) = {
  var != none and var != "" and var != [] and var != () and var != (:) and not (type(var) == str and var.trim() == "")
}

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
	= #enfant.nom, #enfant.prenom
]