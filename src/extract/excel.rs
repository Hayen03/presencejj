use console::{style, Term};
use lazy_static::lazy_static;
use office::{DataType, Excel, Range};
use regex::Regex;

use crate::{groupes::{comptes::CompteReg, groupes::{Groupe, GroupeID, GroupeReg}, membres::MembreReg}, prelude::O, Config};

use super::{ExtractError, BOOL_W_COMMENT_DATA_RE, FALSE_DATA_RE, GROUPE_PROG_RE, GROUPE_RE, TRUE_DATA_RE};

pub fn fill_regs(comptes: &mut CompteReg, membres: &mut MembreReg, groupes: &mut GroupeReg, config: &Config, filepath: &str, out_term: &Term, err_term: &Term) -> Result<(), ExtractError>{
    let mut wb = match Excel::open(filepath) {
        Ok(wb) => wb,
        Err(_) => return Err(ExtractError::CouldNotReadFile),
    };
    let _ = out_term.write_line(&format!("Lecture de \"{}\"", style(filepath).green()));
    let sheets = wb.sheet_names().unwrap();
    for sheet in sheets {
        let mut rng = wb.worksheet_range(&sheet).unwrap();
        let mut g = extract_group_info(rng);
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
                // 2. Boucler sur le reste des lignes pour rajouter les membres
            },
            Err(e) => {
                let _ = err_term.write_line(&format!("Erreur en lisant la page '{}': {}", sheet, e));
            },
        }
    }

    Ok(())
}

fn extract_group_info(ws: Range) -> Result<Groupe, ExtractError> {
    let mut g = Groupe::default();
    let grp_desc = into_string(ws.get_value(0, 0));
    if grp_desc.is_none() {return Err(ExtractError::InvalidGroupFormat);}
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