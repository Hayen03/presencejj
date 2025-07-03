use std::str::FromStr;

use console::{style, Term};
use office::{DataType, Excel, Range};

use crate::{data::{adresse::Adresse, cam::CAM, email::Email, tel::Tel, BoolJustifie, Genre, Taille}, groupes::{comptes::{Compte, CompteID, CompteReg}, fiche_sante::{ALL_ALIMENTAIRE, ALL_ANIMAUX, ALL_INSECTES, ALL_PENICILINE, MAL_ASTHME, MAL_DIABETE, MAL_EMOPHILIE, MAL_EPILEPSIE}, groupes::{Groupe, GroupeID, GroupeReg}, membres::{Contact, Interet, Membre, MembreID, MembreReg}}, prelude::{print_option, Date, O}};
use crate::config::Config;

use super::{ExtractError, BOOL_W_COMMENT_DATA_RE, DATE_NAISSANCE_RE, FALSE_DATA_RE, GROUPE_PROG_RE, GROUPE_RE, TRUE_DATA_RE};

pub fn fill_regs(comptes: &mut CompteReg, membres: &mut MembreReg, groupes: &mut GroupeReg, config: &Config, filepath: &str, out_term: &Term, err_term: &Term) -> Result<(), ExtractError>{
    let mut wb = match Excel::open(filepath) {
        Ok(wb) => wb,
        Err(_) => return Err(ExtractError::CouldNotReadFile),
    };
    let _ = out_term.write_line(&format!("Lecture de \"{}\"", style(filepath).green()));
    let sheets = wb.sheet_names().unwrap();
    let mut dc = None;
    for sheet in sheets {
        let rng = wb.worksheet_range(&sheet).unwrap();
        let g = extract_group_info(&rng);
        //println!("{} = {}", g.id, g.desc());

        // 0. S'assurer qu'il n'y a pas eu d'erreur
        match g {
            Ok(mut grp) => {
                let _ = out_term.write_line(&format!("LECTURE {desc}", desc=grp.desc()));

                // 1. Voir si le groupe existe déjà. Chq. groupe devrait avoir une description unique
                let existing_grp = groupes.groupes().filter(|g| g.equiv(&grp)).map(|g| g.id).collect::<Vec<GroupeID>>();
                let gid = if existing_grp.is_empty() {
                    // 1.1 Si non, rajouter le groupe
                    let id = groupes.get_new_id_from_seed(grp.id.0);
                    grp.id = id;
                    let _ = groupes.add(grp);
                    id
                } else {
                    // 1.2 Si oui, prendre le premier groupe (devrait être le seul)
                    let _ = out_term.write_line("Groupe déjà existant!!");
                    existing_grp[0]
                };
                let grp = groupes.get_mut(gid).unwrap();

                // Construire la configuration des colones si ce n'est pas déjà fait
                if dc.is_none() {
                    dc = Some(DataColConfig::new(&rng, config.excel.data_ln));
                    //println!("{:?}", dc.as_ref().unwrap());
                }
                let dcc = dc.as_ref().unwrap();

                // 2. Boucler sur le reste des lignes pour rajouter les membres
                let rows = rng.rows().skip(config.excel.ln_skip);
                for ln in rows {
                    // 2.1 Trouver le compte
                    match extract_compte_info(ln, dcc) {
                        Err(e) => {
                            let _ = err_term.write_line(&format!("Erreur en lisant le compte '{}': {}", print_option(&into_string(&ln[0])), e));
                        },
                        Ok(mut c) => {
                            let cid = {
                                let existing_compte = comptes.comptes().filter(|cc| cc.equiv(&c)).map(|c| c.id).collect::<Vec<CompteID>>();
                                if !existing_compte.is_empty() {existing_compte[0]}
                                else {
                                    let id = comptes.get_new_id_from_seed(c.id.0);
                                    c.id = id;
                                    //let _ = out_term.write_line(&format!("{} = {} {} #{}", c.id, c.mandataire, print_option(&c.email), print_option(&c.tel)));
                                    //let _ = out_term.write_line(&format!("\t{}", print_option(&c.adresse)));
                                    let _ = comptes.add(c);
                                    id
                                }
                            };
                            let compte = comptes.get_mut(cid).unwrap();

                            match extract_membre_info(ln, dcc) {
                                Err(e) => {
                                    let _ = err_term.write_line(&format!("Erreur en lisant le membre '{}': {}", print_option(&into_string(&ln[0])), e));
                                },
                                Ok(mut mbr) => {
                                    mbr.compte = Some(cid);
                                    let mid = {
                                        let existing_membre = membres.membres().filter(|m| m.equiv(&mbr)).map(|m| m.id).collect::<Vec<MembreID>>();
                                        if !existing_membre.is_empty() {existing_membre[0]}
                                        else {
                                            let id = membres.get_new_id_from_seed(mbr.id.0);
                                            mbr.id = id;
                                            fill_membre_info(ln, dcc, &mut mbr, err_term);
                                            //println!("{:?}", mbr);
                                            let _ = membres.add(mbr);
                                            id
                                        }
                                    };
                                    let membre = membres.get_mut(mid).unwrap();

                                    // ajouter au compte
                                    let _ = compte.add_membre(membre);

                                    // ajouter au groupe
                                    grp.add_participant(mid);
                                }
                            }
                        },
                    }
                }
            },
            Err(e) => {
                let _ = err_term.write_line(&format!("Erreur en lisant la page '{}': {}", sheet, e));
            },
        }
    }

    Ok(())
}

fn extract_group_info(ws: &Range) -> Result<Groupe, ExtractError> {
    let mut g = Groupe::default();
    let grp_desc = into_string(ws.get_value(0, 0));
    if grp_desc.is_none() {return Err(ExtractError::InvalidFormat);}
    let grp_desc = grp_desc.unwrap();
    if let Some(cap) = GROUPE_RE.captures(&grp_desc) {
        g.activite = Some(cap.name("activite").unwrap().as_str().into());
        g.site = Some(cap.name("site").unwrap().as_str().into());
        g.semaine = Some(cap.name("semaine").unwrap().as_str().into());
        match (cap.name("grage_min"), cap.name("grage_max"), cap.name("crocus"), cap.name("balaous"), cap.name("basaltes")) {
            (Some(mn), Some(mx), _, _, _) => { // tranche d'âge
                let grage_min = mn.as_str().parse().unwrap();
                let grage_max = mx.as_str().parse().unwrap();
                g.age_min = Some(grage_min);
                g.age_max = Some(grage_max);
                g.category = Some(Groupe::guess_category(g.age_min, g.age_max));
            },
            (_, _, Some(_), _, _) => { // Crocus
                g.age_min = Some(5);
                g.age_max = Some(6);
                g.category = Some("Crocus".into());
            },
            (_, _, _, Some(_), _) => { // Balaous
                g.age_min = Some(7);
                g.age_max = Some(8);
                g.category = Some("Balaous".into());
            },
            (_, _, _, _, Some(_)) => { // Basaltes
                g.age_min = Some(9);
                g.age_max = Some(12);
                g.category = Some("Basaltes".into());
            },
            _ => { // autre
                g.category = Some(cap.name("category").unwrap().as_str().trim().into());
            }
        }
    }
    let (h, _) = ws.get_size();
    if h < 6 {
        return Err(ExtractError::InvalidFormat);
    }
    let grp_prog = into_string(ws.get_value(2, 0));
    if let Some(grp_prog) = grp_prog {
        if let Some(cap) = GROUPE_PROG_RE.captures(&grp_prog) {
            g.saison = Some(cap.name("prog").unwrap().as_str().into());
        }
    }
    g.discriminant = into_string(ws.get_value(1, 0));
    //g.animateur = into_string(ws.get_value(3, 0));
    g.id = GroupeID(g.get_id_seed());
    Ok(g)
}

fn extract_membre_info(ln: &[DataType], dcc: &DataColConfig) -> Result<Membre, ExtractError> {
    let mut mbr = Membre::default();
    let col_nom = match dcc.nom {
        None => return Err(ExtractError::MissingInformations("Nom")),
        Some(n) => n,
    };
    let col_prenom = match dcc.prenom {
        None => return Err(ExtractError::MissingInformations("Prénom")),
        Some(n) => n,
    };
    let col_naissance = match dcc.naissance {
        None => return Err(ExtractError::MissingInformations("Naissance")),
        Some(n) => n,
    };
    mbr.nom = match into_string(&ln[col_nom]) {
        None => return Err(ExtractError::MissingInformations("Nom")),
        Some(n) => n,
    };
    mbr.prenom = match into_string(&ln[col_prenom]) {
        None => return Err(ExtractError::MissingInformations("Prénom")),
        Some(n) => n,
    };
    mbr.naissance = match into_string(&ln[col_naissance]) {
        None => return Err(ExtractError::MissingInformations("Naissance")),
        Some(n) => {
            if let Some(cap) = DATE_NAISSANCE_RE.captures(&n) {
                let an = cap.name("an").unwrap().as_str().parse().unwrap();
                let mois = cap.name("mois").unwrap().as_str().parse().unwrap();
                let jour = cap.name("jour").unwrap().as_str().parse().unwrap();
                match Date::from_ymd_opt(an, mois, jour) {
                    None => return Err(ExtractError::MissingInformations("Naissance")),
                    Some(d) => d,
                }
            }
            else {
                return Err(ExtractError::MissingInformations("Naissance"));
            }
        },
    };
    mbr.id = MembreID(mbr.get_id_seed());
    Ok(mbr)
}
fn fill_membre_info(ln: &[DataType], dcc: &DataColConfig, membre: &mut Membre, err_term: &Term) {
    // allergies
    if let Some(col) = dcc.all_alim {
        if let Some((b, c)) = into_bool_with_comment(&ln[col]) {
            if b {
                membre.fiche_sante.allergies.push(ALL_ALIMENTAIRE.into());
            }
            if let Some(s) = c {
                for a in s.split(&[',', ';']) {
                    membre.fiche_sante.allergies.push(a.trim().replace('\n', " "));
                }
            }
        }
    }
    if let Some(col) = dcc.all_anim {
        if let Some(b) = into_bool(&ln[col]) {
            if b {
                membre.fiche_sante.allergies.push(ALL_ANIMAUX.into());
            }
        }
    }
    if let Some(col) = dcc.all_insecte {
        if let Some(b) = into_bool(&ln[col]) {
            if b {
                membre.fiche_sante.allergies.push(ALL_INSECTES.into());
            }
        }
    }
    if let Some(col) = dcc.all_peni {
        if let Some(b) = into_bool(&ln[col]) {
            if b {
                membre.fiche_sante.allergies.push(ALL_PENICILINE.into());
            }
        }
    }
    if let Some(col) = dcc.all_autre {
        if let Some((_, Some(s))) = into_bool_with_comment(&ln[col]) {
            for a in s.split(&[',', ';']) {
                membre.fiche_sante.allergies.push(a.trim().replace('\n', " "));
            }
        }
    }

    // maladies
    if let Some(col) = dcc.mal_asthme {
        if let Some(b) = into_bool(&ln[col]) {
            if b {
                membre.fiche_sante.maladies.push(MAL_ASTHME.into());
            }
        }
    }
    if let Some(col) = dcc.mal_diabete {
        if let Some(b) = into_bool(&ln[col]) {
            if b {
                membre.fiche_sante.maladies.push(MAL_DIABETE.into());
            }
        }
    }
    if let Some(col) = dcc.mal_emo {
        if let Some(b) = into_bool(&ln[col]) {
            if b {
                membre.fiche_sante.maladies.push(MAL_EMOPHILIE.into());
            }
        }
    }
    if let Some(col) = dcc.mal_epi {
        if let Some(b) = into_bool(&ln[col]) {
            if b {
                membre.fiche_sante.maladies.push(MAL_EPILEPSIE.into());
            }
        }
    }
    if let Some(col) = dcc.mal_autre {
        if let Some((_, Some(s))) = into_bool_with_comment(&ln[col]) {
            for a in s.split(&[',', ';']) {
                membre.fiche_sante.maladies.push(a.trim().replace('\n', " "));
            }
        }
    }

    // Medicament
    if let Some(col) = dcc.med_acetaminophene {
        if let Some(b) = into_bool(&ln[col]) {
            membre.fiche_sante.auth_medicaments.acetaminophene = Some(b);
        }
    }
    if let Some(col) = dcc.med_antibio {
        if let Some(b) = into_bool(&ln[col]) {
            membre.fiche_sante.auth_medicaments.anti_biotique = Some(b);
        }
    }
    if let Some(col) = dcc.med_antieme {
        if let Some(b) = into_bool(&ln[col]) {
            membre.fiche_sante.auth_medicaments.anti_emetique = Some(b);
        }
    }
    if let Some(col) = dcc.med_antiinfl {
        if let Some(b) = into_bool(&ln[col]) {
            membre.fiche_sante.auth_medicaments.anti_inflamatoire = Some(b);
        }
    }
    if let Some(col) = dcc.med_ibu {
        if let Some(b) = into_bool(&ln[col]) {
            membre.fiche_sante.auth_medicaments.ibuprofene = Some(b);
        }
    }
    if let Some(col) = dcc.med_sirop_toux {
        if let Some(b) = into_bool(&ln[col]) {
            membre.fiche_sante.auth_medicaments.sirop_toux = Some(b);
        }
    }

    // Authorisation de soins
    if let Some(col) = dcc.auth_soins {
        if let Some(b) = into_bool(&ln[col]) {
            membre.fiche_sante.auth_soins = Some(b);
        }
    }

    // problèmes de comportement
    if let Some(col) = dcc.prob_comportement {
        //println!("Trouvé la colone");
        if let Some((b, c)) = into_bool_with_comment(&ln[col]) {
            //println!("Lit le bool justifie");
            let bj = BoolJustifie {
                reponse: b,
                justification: c,
            };
            membre.fiche_sante.probleme_comportement = Some(bj);
        }
    }

    // prise de médicament
    if let Some(col) = dcc.prise_med {
        if let Some((b, c)) = into_bool_with_comment(&ln[col]) {
            let bj = BoolJustifie {
                reponse: b,
                justification: c,
            };
            membre.fiche_sante.prise_med = Some(bj);
        }
    }

    // carte d'assurance maladies
    if let Some(col) = dcc.cam {
        if let Some(s) = into_string(&ln[col]) {
            match CAM::from_str(&s) {
                Ok(cam) => membre.fiche_sante.cam = Some(cam),
                Err(_e) => {let _ = err_term.write_line(&format!("Erreur en lisant le CAM: {} ({})", s, _e.to_string()));},
            }
        }
    }

    // genre
    if let Some(col) = dcc.genre {
        if let Some(s) = into_string(&ln[col]) {
            match Genre::from_str(&s) {
                Ok(genre) => membre.genre = Some(genre),
                Err(_) => {let _ = err_term.write_line(&format!("Erreur en lisant le genre: {}", s));},
            }
        }
    }

    // interets
    if let Some(col) = dcc.interet_1 {
        if let Some(s) = into_string(&ln[col]) {
            match Interet::from_str(&s) {
                Ok(interet) => membre.interets[0] = Some(interet),
                Err(_) => { let _ = err_term.write_line("Erreur en lisant le 1er interet"); }
            }
        }
    }
    if let Some(col) = dcc.interet_2 {
        if let Some(s) = into_string(&ln[col]) {
            match Interet::from_str(&s) {
                Ok(interet) => membre.interets[1] = Some(interet),
                Err(_) => { let _ = err_term.write_line("Erreur en lisant le 1er interet"); }
            }
        }
    }
    if let Some(col) = dcc.interet_3 {
        if let Some(s) = into_string(&ln[col]) {
            match Interet::from_str(&s) {
                Ok(interet) => membre.interets[2] = Some(interet),
                Err(_) => { let _ = err_term.write_line("Erreur en lisant le 1er interet"); }
            }
        }
    }
    if let Some(col) = dcc.interet_4 {
        if let Some(s) = into_string(&ln[col]) {
            match Interet::from_str(&s) {
                Ok(interet) => membre.interets[3] = Some(interet),
                Err(_) => { let _ = err_term.write_line("Erreur en lisant le 1er interet"); }
            }
        }
    }

    // contacts
    if let Some(col) = dcc.contact_1_nom {
        if let Some(nom) = into_string(&ln[col]) {
            let mut contact = Contact {nom, tel: None, lien: None};
            if let Some(col) = dcc.contact_1_tel {
                if let Some(s) = into_string(&ln[col]) {
                    if let Ok(tel) = Tel::from_str(s.trim()) {
                        contact.tel = Some(tel);
                    }
                }
            }
            if let Some(col) = dcc.contact_1_lien {
                if let Some(s) = into_string(&ln[col]) {
                    contact.lien = Some(s.trim().into())
                }
            }
            membre.contacts[0] = Some(contact);
        }
    }
    if let Some(col) = dcc.contact_2_nom {
        if let Some(nom) = into_string(&ln[col]) {
            let mut contact = Contact {nom, tel: None, lien: None};
            if let Some(col) = dcc.contact_2_tel {
                if let Some(s) = into_string(&ln[col]) {
                    if let Ok(tel) = Tel::from_str(s.trim()) {
                        contact.tel = Some(tel);
                    }
                }
            }
            if let Some(col) = dcc.contact_2_lien {
                if let Some(s) = into_string(&ln[col]) {
                    contact.lien = Some(s.trim().into())
                }
            }
            membre.contacts[1] = Some(contact);
        }
    }

    // accompagnement
    if let Some(col) = dcc.accompagnement {
        if let Some(b) = into_bool(&ln[col]) {
            membre.accompagnement = Some(b);
        }
    }

    // quitte avec
    if let Some(col) = dcc.quit_parent {
        if let Some(b) = into_bool(&ln[col]) {
            if b {
                membre.quitte.avec.push("Parent".into());
            }
        }
    }
    if let Some(col) = dcc.quit_seul {
        if let Some(b) = into_bool(&ln[col]) {
            if b {
                membre.quitte.avec.push("Seul".into());
            }
        }
    }
    if let Some(col) = dcc.quit_acceuil {
        if let Some((b, c)) = into_bool_with_comment(&ln[col]) {
            if b {
                match c {
                    None => membre.quitte.avec.push("Service d'accueil".into()),
                    Some(c) => membre.quitte.avec.push(format!("Service d'accueil: {}", c)),
                }
            }
        }
    }
    if let Some(col) = dcc.quit_autre {
        if let Some((b, c)) = into_bool_with_comment(&ln[col]) {
            if b {
                match c {
                    None => membre.quitte.avec.push("Autre".into()),
                    Some(c) => membre.quitte.avec.push(format!("Autre: {}", c)),
                }
            }
        }
    }
    if let Some(col) = dcc.mdp {
        if let Some(s) = into_string(&ln[col]) {
            membre.quitte.mdp = Some(s);
        }
    }

    // piscine
    if let Some(col) = dcc.vfi {
        if let Some(b) = into_bool(&ln[col]) {
            membre.piscine.vfi = Some(b);
        }
    }
    if let Some(col) = dcc.auth_partage_sauveteur {
        if let Some(b) = into_bool(&ln[col]) {
            membre.piscine.partage = Some(b);
        }
    }
    if let Some(col) = dcc.tete_sous_eau {
        if let Some(b) = into_bool(&ln[col]) {
            membre.piscine.tete_sous_eau = Some(b);
        }
    }

    // taille
    if let Some(col) = dcc.taille {
        if let Some(s) = into_string(&ln[col]) {
            match Taille::from_str(&s) {
                Ok(t) => membre.taille = Some(t),
                Err(_e) => { let _ = err_term.write_line("Erreur en lisant la taille"); }
            }
        }
    }

    // authorisation photo
    if let Some(col) = dcc.auth_photo {
        if let Some(b) = into_bool(&ln[col]) {
            membre.auth_photo = Some(b);
        }
    }

    // commentaires
    if let Some(col) = dcc.commentaire {
        membre.commentaire = into_string(&ln[col]);
    }
}

fn extract_compte_info(ln: &[DataType], dcc: &DataColConfig) -> Result<Compte, ExtractError> {
    let mut cmpt = Compte::default();
    let col_mandataire = match dcc.mandataire {
        None => return Err(ExtractError::MissingInformations("Mandataire")),
        Some(n) => n,
    };
    cmpt.mandataire = match into_string(&ln[col_mandataire]) {
        None => return Err(ExtractError::InvalidFormat),
        Some(m) => m,
    };
    if let Some(col_email) = dcc.courriel {
        let email = into_string(&ln[col_email]).map(|s| Email::from_str(&s).ok());
        cmpt.email = email.unwrap_or_default();
    }
    if let Some(col_tel) = dcc.tel {
        let t = into_string(&ln[col_tel]).map(|s| Tel::from_str(&s).ok());
        cmpt.tel = t.unwrap_or_default();
        if cmpt.tel.is_none() {
            println!("N'a pu lire le numéro de téléphone pour le compte '{}'", cmpt.mandataire);
        }
    }
    if let Some(col_adr) = dcc.adresse {
        let adr = into_string(&ln[col_adr]).map(|s| Adresse::from_full(&s).ok());
        cmpt.adresse = adr.unwrap_or_default();
    }
    cmpt.id = CompteID(cmpt.get_id_seed());
    Ok(cmpt)
}

pub fn into_string(data: &DataType) -> O<String> {
    let ret = match data {
        DataType::Int(i) => Some(i.to_string()),
        DataType::Float(f) => Some(f.to_string()),
        DataType::String(s) => {
            let ss = s.trim();
            if ss.is_empty() {
                None
            } else {
                Some(String::from(ss))
            }
        }
        DataType::Bool(b) => Some(b.to_string()),
        DataType::Error(_) => None,
        DataType::Empty => None,
    };
    ret
}
pub fn into_int(data: &DataType) -> O<i64> {
    let ret = match data {
        DataType::Int(i) => Some(*i),
        DataType::Float(f) => Some(*f as i64),
        DataType::String(s) => match s.parse() {
            Ok(n) => Some(n),
            Err(_) => None,
        },
        DataType::Bool(b) => Some(i64::from(*b)),
        DataType::Error(_) => None,
        DataType::Empty => None,
    };
    ret
}
pub fn into_bool(data: &DataType) -> O<bool> {
    match data {
        DataType::Int(i) => Some(*i == 0),
        DataType::Float(f) => Some(*f == 0.0),
        DataType::String(s) => {
            if TRUE_DATA_RE.is_match(s) {
                Some(true)
            } else if FALSE_DATA_RE.is_match(s) {
                Some(false)
            } else {
                None
            }
        }
        DataType::Bool(b) => Some(*b),
        DataType::Error(_) => None,
        DataType::Empty => None,
    }
}
pub fn into_bool_with_comment(data: &DataType) -> O<(bool, O<String>)> {
    if let DataType::String(s) = data {
        if let Some(cap) = BOOL_W_COMMENT_DATA_RE.captures(s) {
            let bool_str = cap.name("bool").unwrap().as_str();
            let comment = cap.name("comment").map(|m| String::from(m.as_str().trim()));
            let comment = comment.filter(|c| !c.is_empty());
            let bool_val = TRUE_DATA_RE.is_match(bool_str);
            Some((bool_val, comment))
        } else {
            None
        }
    } else {
        None
    }
}

#[derive(Debug)]
struct DataColConfig {
    nom: O<usize>,
    prenom: O<usize>,
    genre: O<usize>,
    naissance: O<usize>,
    mandataire: O<usize>,
    courriel: O<usize>,
    tel: O<usize>,
    adresse: O<usize>,
    accompagnement: O<usize>,
    cam: O<usize>,
    auth_soins: O<usize>,
    prob_comportement: O<usize>,
    prise_med: O<usize>,
    med_acetaminophene: O<usize>,
    med_antibio: O<usize>,
    med_antiinfl: O<usize>,
    med_ibu: O<usize>,
    med_sirop_toux: O<usize>,
    med_antieme: O<usize>,
    mal_asthme: O<usize>,
    mal_diabete: O<usize>,
    mal_emo: O<usize>,
    mal_epi: O<usize>,
    mal_autre: O<usize>,
    all_alim: O<usize>,
    all_anim: O<usize>,
    all_insecte: O<usize>,
    all_peni: O<usize>,
    all_autre: O<usize>,
    contact_1_nom: O<usize>,
    contact_1_tel: O<usize>,
    contact_1_lien: O<usize>,
    contact_2_nom: O<usize>,
    contact_2_tel: O<usize>,
    contact_2_lien: O<usize>,
    quit_parent: O<usize>,
    quit_seul: O<usize>,
    quit_acceuil: O<usize>,
    quit_autre: O<usize>,
    mdp: O<usize>,
    auth_partage_sauveteur: O<usize>,
    vfi: O<usize>,
    tete_sous_eau: O<usize>,
    taille: O<usize>,
    interet_1: O<usize>,
    interet_2: O<usize>,
    interet_3: O<usize>,
    interet_4: O<usize>,
    auth_photo: O<usize>,
    commentaire: O<usize>,
}
impl DataColConfig {
    fn new(rng: &Range, ln: usize) -> Self {
        // créer la ligne que l'on pourra fouiller
        let mut i = 0;
        let (_, tgt) = rng.get_size();
        let mut cols = Vec::new();
        while i < tgt {
            cols.push(rng.get_value(ln, i));
            i += 1;
        }
        Self {
            nom: DataColConfig::search(&cols, "Nom"),
            prenom: DataColConfig::search(&cols, "Prénom"),
            genre: DataColConfig::search(&cols, "Genre"),
            naissance: DataColConfig::search(&cols, "Date de naissance"),
            mandataire: DataColConfig::search(&cols, "Mandataire du compte"),
            courriel: DataColConfig::search(&cols, "Courriel"),
            tel: match DataColConfig::search(&cols, "Principal") {
                Some(tel) => Some(tel),
                None => DataColConfig::search(&cols, "Numéro de téléphone principal"),
            },
            adresse: match DataColConfig::search(&cols, "Adresse princ.") {
                Some(adr) => Some(adr),
                None => DataColConfig::search(&cols, "Adresse principale")
            },
            accompagnement: DataColConfig::search(&cols, "Accompagnement"),
            cam: DataColConfig::search(&cols, "assurance maladie"),
            auth_soins: DataColConfig::search(&cols, "Autorisation de soigner"),
            prob_comportement: DataColConfig::search(&cols, "Problèmes de comportement?"),
            prise_med: DataColConfig::search(&cols, "Prise de Médicament"),
            med_acetaminophene: DataColConfig::search(&cols, "Med Acétaminophène"),
            med_antibio: DataColConfig::search(&cols, "Med Antibiotique"),
            med_antiinfl: DataColConfig::search(&cols, "Med Anti-Inflamatoires"),
            med_ibu: DataColConfig::search(&cols, "Med Ibuprophène"),
            med_sirop_toux: DataColConfig::search(&cols, "Med Sirop Toux"),
            med_antieme: DataColConfig::search(&cols, "Med Antiémétique"),
            mal_asthme: DataColConfig::search(&cols, "MC Asthme"),
            mal_diabete: DataColConfig::search(&cols, "MC Diabète"),
            mal_emo: DataColConfig::search(&cols, "MC Émophilie"),
            mal_epi: DataColConfig::search(&cols, "MC Épilépsie"),
            mal_autre: DataColConfig::search(&cols, "MC Autre"),
            all_alim: DataColConfig::search(&cols, "Allergie Alimentaire"),
            all_anim: DataColConfig::search(&cols, "Allergie Animaux"),
            all_insecte: DataColConfig::search(&cols, "Allergie Insecte"),
            all_peni: DataColConfig::search(&cols, "Allergie Pénicilline"),
            all_autre: DataColConfig::search(&cols, "Allergie Autre"),
            contact_1_nom: DataColConfig::search(&cols, "contact d'urgence - Nom"),
            contact_1_tel: DataColConfig::search(&cols, "contact d'urgence - Téléphone"),
            contact_1_lien: DataColConfig::search(&cols, "contact d'urgence - Lien de parenté"),
            contact_2_nom: DataColConfig::search(&cols, "Contact Urgence 2 - Nom"),
            contact_2_tel: DataColConfig::search(&cols, "Contact Urgence 2 - Téléphone"),
            contact_2_lien: DataColConfig::search(&cols, "Contact Urgence 2 - Lien de parenté"),
            quit_parent: DataColConfig::search(&cols, "Quitte Parent"),
            quit_seul: DataColConfig::search(&cols, "Quitte Seul"),
            quit_acceuil: DataColConfig::search(&cols, "Quitte Service d'accueil"),
            quit_autre: DataColConfig::search(&cols, "Quitte Autre"),
            mdp: DataColConfig::search(&cols, "Mdp"),
            auth_partage_sauveteur: DataColConfig::search(&cols, "Autorisation Partage aux Sauveteurs"),
            vfi: DataColConfig::search(&cols, "VFI"),
            tete_sous_eau: DataColConfig::search(&cols, "Tête Sous l'Eau"),
            taille: DataColConfig::search(&cols, "Taille"),
            interet_1: DataColConfig::search(&cols, "Interet_1"),
            interet_2: DataColConfig::search(&cols, "Interet_2"),
            interet_3: DataColConfig::search(&cols, "Interet_3"),
            interet_4: DataColConfig::search(&cols, "Interet_4"),
            auth_photo: DataColConfig::search(&cols, "Autorisation Photo"),
            commentaire: DataColConfig::search(&cols, "Commentaires"),
        }
    }
    fn search(cols: &[&DataType], trgt: &str) -> O<usize> {
        for (n, elem) in cols.iter().enumerate() {
            if let DataType::String(s) = elem {
                if s.as_str().trim() == trgt {
                    return Some(n);
                }
            }
        }
        None
    }
}