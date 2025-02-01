use std::str::FromStr;

use console::{style, Term};
use lazy_static::lazy_static;
use office::{DataType, Excel, Range};
use regex::Regex;

use crate::{data::{adresse::Adresse, email::Email, tel::Tel}, groupes::{comptes::{Compte, CompteID, CompteReg}, groupes::{Groupe, GroupeID, GroupeReg}, membres::{Membre, MembreID, MembreReg}}, prelude::{print_option, O}, Config};

use super::{ExtractError, BOOL_W_COMMENT_DATA_RE, FALSE_DATA_RE, GROUPE_PROG_RE, GROUPE_RE, TRUE_DATA_RE};

pub fn fill_regs(comptes: &mut CompteReg, membres: &mut MembreReg, groupes: &mut GroupeReg, config: &Config, filepath: &str, out_term: &Term, err_term: &Term) -> Result<(), ExtractError>{
    let mut wb = match Excel::open(filepath) {
        Ok(wb) => wb,
        Err(_) => return Err(ExtractError::CouldNotReadFile),
    };
    let _ = out_term.write_line(&format!("Lecture de \"{}\"", style(filepath).green()));
    let sheets = wb.sheet_names().unwrap();
    let mut dc = None;
    for sheet in sheets {
        let mut rng = wb.worksheet_range(&sheet).unwrap();
        let mut g = extract_group_info(&rng);
        //println!("{} = {}", g.id, g.desc());

        // 0. S'assurer qu'il n'y a pas eu d'erreur
        match g {
            Ok(mut grp) => {
                // 1. Voir si le groupe existe déjà. Chq. groupe devrait avoir une description unique
                let existing_grp = groupes.groupes().filter(|g| g.equiv(&grp)).map(|g| g.id).collect::<Vec<GroupeID>>();
                let gid = if existing_grp.len() == 0 {
                    // 1.1 Si non, rajouter le groupe
                    let id = groupes.get_new_id_from_seed(grp.id.0);
                    grp.id = id;
                    let _ = groupes.add(grp);
                    id
                } else {
                    // 1.2 Si oui, prendre le premier groupe (devrait être le seul)
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
                                if existing_compte.len() > 0 {existing_compte[0]}
                                else {
                                    let id = comptes.get_new_id_from_seed(c.id.0);
                                    c.id = id;
                                    let _ = out_term.write_line(&format!("{} = {} {} #{}", c.id, c.mandataire, print_option(&c.email), print_option(&c.tel)));
                                    let _ = out_term.write_line(&format!("\t{}", print_option(&c.adresse)));
                                    let _ = comptes.add(c);
                                    id
                                }
                            };
                            let mut compte = comptes.get_mut(cid).unwrap();

                            match extract_membre_info(ln, dcc) {
                                Err(e) => {
                                    let _ = err_term.write_line(&format!("Erreur en lisant le membre '{}': {}", print_option(&into_string(&ln[0])), e));
                                },
                                Ok(mut mbr) => {
                                    mbr.compte = Some(cid);
                                    let mid = {
                                        let existing_membre = membres.membres().filter(|m| m.equiv(&mbr)).map(|m| m.id).collect::<Vec<MembreID>>();
                                        if existing_membre.len() > 0 {existing_membre[0]}
                                        else {
                                            let id = membres.get_new_id_from_seed(mbr.id.0);
                                            mbr.id = id;
                                            fill_membre_info(ln, dcc, &mut mbr);
                                            let _ = membres.add(mbr);
                                            id
                                        }
                                    };
                                    let mut membre = membres.get_mut(mid).unwrap();

                                    // ajouter au compte
                                    let _ = compte.add_membre(&mut membre);

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
        let grage_min = cap.name("grage_min").unwrap().as_str();
        let grage_max = cap.name("grage_max").unwrap().as_str();
        g.category = Some(format!("{}-{} ans", grage_min, grage_max));
    }
    let grp_prog = into_string(ws.get_value(2, 0));
    if let Some(grp_prog) = grp_prog {
        if let Some(cap) = GROUPE_PROG_RE.captures(&grp_prog) {
            g.saison = Some(cap.name("prog").unwrap().as_str().into());
        }
    }
    g.discriminant = into_string(ws.get_value(1, 0));
    g.animateur = into_string(ws.get_value(3, 0));
    g.id = GroupeID(g.get_id_seed());
    Ok(g)
}

fn extract_membre_info(ln: &[DataType], dcc: &DataColConfig) -> Result<Membre, ExtractError> {
    let mut mbr = Membre::default();
    //todo!();
    Ok(mbr)
}
fn fill_membre_info(ln: &[DataType], dcc: &DataColConfig, membre: &mut Membre) {
    //todo!();
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
        cmpt.email = match email {
            None => None,
            Some(o) => o,
        };
    }
    if let Some(col_tel) = dcc.tel {
        let t = into_string(&ln[col_tel]).map(|s| Tel::from_str(&s).ok());
        cmpt.tel = match t {
            None => None,
            Some(o) => o,
        };
    }
    if let Some(col_adr) = dcc.adresse {
        let adr = into_string(&ln[col_adr]).map(|s| Adresse::from_full(&s).ok());
        cmpt.adresse = match adr {
            None => None,
            Some(o) => o,
        };
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
pub fn into_bool(data: &DataType) -> O<bool> {
    match data {
        DataType::Int(i) => Some(*i == 0),
        DataType::Float(f) => Some(*f == 0.0),
        DataType::String(s) => {
            if TRUE_DATA_RE.is_match(&s) {
                Some(true)
            } else if FALSE_DATA_RE.is_match(&s) {
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
            tel: DataColConfig::search(&cols, "Principal"),
            adresse: DataColConfig::search(&cols, "Adresse princ."),
            accompagnement: DataColConfig::search(&cols, "Accompagnement"),
            cam: DataColConfig::search(&cols, "assurance maladie"),
            auth_soins: DataColConfig::search(&cols, "Autorisation de soigner"),
            prob_comportement: DataColConfig::search(&cols, "Problème de comportement?"),
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