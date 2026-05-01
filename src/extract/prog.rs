use std::str::FromStr;

use console::{style, Term};
use office::{DataType, Excel, Range};

use crate::{data::{BoolJustifie, Genre, Taille, adresse::Adresse, cam::CAM, email::Email, tel::Tel}, groupes::{comptes::{Compte, CompteID, CompteReg}, fiche_sante::{ALL_ALIMENTAIRE, ALL_ANIMAUX, ALL_INSECTES, ALL_PENICILINE, MAL_ASTHME, MAL_DIABETE, MAL_EMOPHILIE, MAL_EPILEPSIE}, groupes::{Groupe, GroupeID, GroupeReg}, membres::{Contact, Interet, Membre, MembreID, MembreReg}}, prelude::{Date, Logger, O, print_option}};
use crate::config::Config;

use super::{excel::{into_int, into_string}, ExtractError, BOOL_W_COMMENT_DATA_RE, DATE_NAISSANCE_RE, FALSE_DATA_RE, GROUPE_PROG_RE, GROUPE_RE, TRUE_DATA_RE};

pub fn extract_group_info_from_prog(ws: &[DataType], config: &ProgLnConfig, saison: O<&str>) -> Result<Groupe, ExtractError> {
    let mut g = Groupe::default();
    g.saison = saison.map(String::from);
    let grp_desc = match config.nom {
        Some(pos) => into_string(&ws[pos]),
        None => return Err(ExtractError::InvalidFormat),
    };
    if grp_desc.is_none() {return Err(ExtractError::InvalidGroupNameFormat);}
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
    } else {
        return Err(ExtractError::InvalidGroupNameFormat);
    }
    let grp_prog = match config.programmation {
        Some(pos) => into_string(&ws[pos]),
        None => None,
    };
    g.saison = g.saison.or(grp_prog);

    g.capacite = match config.capacite {
        Some(pos) => into_int(&ws[pos]).map(|n| n as usize),
        None => None,
    };
    g.id = GroupeID(g.get_id_seed());

    let mut missing = vec![];
    if g.activite.is_none() { missing.push("activité"); }
    if g.saison.is_none() { missing.push("saison"); }

    if !missing.is_empty() {
        Err(ExtractError::NotAGroup {missing})
    } else {
        Ok(g)
    }
}

pub fn fill_groupe_reg(ws: &Range, reg: &mut GroupeReg, out: Logger, err: Logger) {
    let mut config = ProgLnConfig::default();
    let mut saison = None;
    for (i, row) in ws.rows().enumerate() {
        //let _ = out.write_line(&format!("Reading {:?}", into_string(ws.get_value(i, 2))));
        if i == 0 {
            saison = into_string(&row[0]);
        } else if i == 1 {
            config = ProgLnConfig::guess(row);
            //println!("{:?}", config.programmation)
        } else if let Ok(mut grp) = extract_group_info_from_prog(row, &config, saison.as_deref()) {
            out(&format!("LECTURE {desc}", desc=grp.desc()));

            let cap = grp.capacite;

            // 1. Voir si le groupe existe déjà
            let existing_grp = reg.groupes().filter(|g| g.equiv(&grp)).map(|g| g.id).collect::<Vec<GroupeID>>();
            let gid = if existing_grp.is_empty() {
                // 1.1 Si non, rajouter le groupe
                let id = reg.get_new_id_from_seed(grp.id.0);
                grp.id = id;
                let _ = reg.add(grp);
                id
            } else {
                // 1.2 Si oui, prendre le premier groupe (devrait être le seul)
                existing_grp[0]
            };
            let groupe = reg.get_mut(gid).unwrap();

            // 2. mettre à jour certaines données
            if !existing_grp.is_empty() {
                groupe.capacite = match cap {
                    None => groupe.capacite,
                    Some(cap) => Some(cap),
                }
            }
        }
    }
}

pub fn fill_groupe_reg_from_prog(ws: &Range, reg: &mut GroupeReg, out: &Term, err: &Term) {
    fill_groupe_reg(ws, reg, &|s| { let _ = out.write_line(s); }, &|s| { let _ = err.write_line(s); });
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct ProgLnConfig {
    programmation: O<usize>,
    activite: O<usize>,
    nom: O<usize>,
    debut: O<usize>,
    fin: O<usize>,
    capacite: O<usize>,
    age_min: O<usize>,
    age_max: O<usize>,
}
impl Default for ProgLnConfig {
    fn default() -> Self {
        Self {
            programmation: Some(0),
            activite: Some(1),
            nom: Some(2),
            debut: Some(3), 
            fin: Some(4),
            capacite: Some(11),
            age_min: Some(12),
            age_max: Some(15),
        }
    }
}
impl ProgLnConfig {
    pub fn guess(range: &[DataType]) -> Self {
        Self {
            programmation: vec![
                Self::search(range, "Programmation"),
                Self::search(range, "Programmation\n"),
            ].into_iter().find(|o| o.is_some()).flatten(),
            activite: vec![
                Self::search(range, "Activité"),
                Self::search(range, "Activité\n"),
            ].into_iter().find(|o| o.is_some()).flatten(),
            nom: vec![
                Self::search(range, "Groupe"),
                Self::search(range, "Nom du groupe"),
                Self::search(range, "Nom du groupe\n"),
            ].into_iter().find(|o| o.is_some()).flatten(),
            debut: vec![
                Self::search(range, "Début"),
                Self::search(range, "Début du groupe"),
                Self::search(range, "Début du groupe\n"),
            ].into_iter().find(|o| o.is_some()).flatten(),
            fin: vec![
                Self::search(range, "Fin"),
                Self::search(range, "Fin du groupe"),
                Self::search(range, "Fin du groupe\n"),
            ].into_iter().find(|o| o.is_some()).flatten(),
            capacite: vec![
                Self::search(range, "Nombre de place max"),
                Self::search(range, "Nombre maximal de places"),
                Self::search(range, "Nombre maximal de places\n"),
            ].into_iter().find(|o| o.is_some()).flatten(),
            age_min: vec![
                Self::search(range, "Restriction âge min."),
                Self::search(range, "Restriction d'âge minimale"),
                Self::search(range, "Restriction d'âge minimale\n"),
            ].into_iter().find(|o| o.is_some()).flatten(),
            age_max: vec![
                Self::search(range, "Restriction âge max."),
                Self::search(range, "Restriction d'âge maximale"),
                Self::search(range, "Restriction d'âge maximale\n"),
            ].into_iter().find(|o| o.is_some()).flatten(),
        }
    }
    fn search(cols: &[DataType], trgt: &str) -> O<usize> {
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