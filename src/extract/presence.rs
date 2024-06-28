// Extrait les informations d'un groupe contenu dans une page du doc excel de présence

use std::fmt::Display;

use crate::{
    groupes::saisons,
    prelude::{O, R},
};

use super::{extract_groupe_info_from_name, into_string, ExtractError, GroupeInfoExtract};

/**
 * Contient les positions des informations de groupe dans le fichier excel
 */
pub struct GroupeExtractConfig {
    pub nom: (usize, usize),
    pub saison: (usize, usize),
    pub sous_groupe: (usize, usize),
    pub responsable: (usize, usize),
    pub profil: (usize, usize),
}
impl Default for GroupeExtractConfig {
    fn default() -> Self {
        GroupeExtractConfig {
            nom: (0, 0),
            saison: (2, 0),
            sous_groupe: (1, 0),
            responsable: (4, 0),
            profil: (3, 0),
        }
    }
}

/**
 * Contient les informations tirées du fichier excel avant le traitement
 */
pub struct GroupeExtractData {
    pub nom: String, // nom du groupe;
    pub desc: GroupeInfoExtract,
    pub saison: O<String>,
    pub sous_groupe: O<String>,
    pub responsable: O<String>,
    pub profil: O<String>,
}
impl GroupeExtractData {
    pub fn extract(config: &GroupeExtractConfig, range: &office::Range) -> R<Self, ExtractError> {
        let nom = match into_string(range.get_value(config.nom.0, config.nom.1)) {
            Some(n) => n,
            None => return Err(ExtractError::InvalidGroupFormat),
        };
        let saison = into_string(range.get_value(config.saison.0, config.saison.1));
        let sous_groupe = into_string(range.get_value(config.sous_groupe.0, config.sous_groupe.1));
        let responsable = into_string(range.get_value(config.responsable.0, config.responsable.1));
        let profil = into_string(range.get_value(config.profil.0, config.profil.1));
        let desc = extract_groupe_info_from_name(&nom)?;

        Ok(Self {
            nom,
            saison,
            sous_groupe,
            responsable,
            profil,
            desc,
        })
    }
}
impl Display for GroupeExtractData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = match &self.saison {
            None => self.desc.to_string(),
            Some(saison) => saison.clone() + " " + &self.desc.to_string(),
        };
        if self.sous_groupe.is_some() || self.responsable.is_some() || self.profil.is_some() {
            let mut first = true;
            s += " (";
            if let Some(sg) = &self.sous_groupe {
                s += sg;
                first = false;
            }
            if let Some(resp) = &self.responsable {
                if !first {
                    s += " | ";
                }
                s += "respo ";
                s += resp;
                first = false;
            }
            if let Some(pro) = &self.profil {
                if !first {
                    s += " | ";
                }
                s += "profil ";
                s += pro;
            }
            s += ")";
        }
        write!(f, "{}", s)
    }
}
