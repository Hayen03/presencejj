use std::{error::Error, fmt::Display};

use lazy_static::lazy_static;
use office::DataType;
use regex::Regex;

use crate::prelude::O;

pub mod presence;

#[derive(Debug, Clone, Copy)]
pub enum ExtractError {
    InvalidGroupFormat,
    InvalidGroupNameFormat,
}
impl Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ExtractError: {}",
            match self {
                ExtractError::InvalidGroupFormat => "Format de groupe invalide",
                ExtractError::InvalidGroupNameFormat => "Nom de groupe invalide",
            }
        )
    }
}
impl Error for ExtractError {}
impl From<GroupeInfoExtractError> for ExtractError {
    fn from(_value: GroupeInfoExtractError) -> Self {
        Self::InvalidGroupNameFormat
    }
}

lazy_static! {
    pub static ref TRUE_DATA_RE: Regex = Regex::new("^(?i)(?:oui|true|vrai|yes|o|y|v|t)$").unwrap();
    pub static ref FALSE_DATA_RE: Regex = Regex::new("^(?i)(?:non|false|no|faux|n|f)$").unwrap();
    pub static ref BOOL_W_COMMENT_DATA_RE: Regex = Regex::new("^(?i)(?P<bool>oui|true|vrai|yes|o|y|v|t|non|false|no|faux|n|f)(:?\\s*,\\s*(?P<comment>.+?)\\s*)?").unwrap();
    pub static ref GROUPE_RE: Regex = Regex::new(r"^(?i)\s*(?P<activite>[^|]+?)\s*\|\s*(?P<site>[^|]+?)\s*\|\s*(?P<grage_min>\d+)\s*(?:-\s*)?(?P<grage_max>\d+)\s*ans\s*\|\s*(?:sem|semaine)\.?\s*(?P<semaine>\d+)\s*$").unwrap();
}

pub struct GroupeInfoExtract {
    pub activite: String,
    pub site: String,
    pub grpage_min: i32,
    pub grpage_max: i32,
    pub semaine: i32,
}

#[derive(Debug)]
pub struct GroupeInfoExtractError {}
impl Error for GroupeInfoExtractError {}
impl Display for GroupeInfoExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mauvais format de nom de groupe. (Bon format \"{{activite}} | {{site}} | {{age-min}}-{{age-max}} ans | sem. {{semaine}}\")")
    }
}
pub fn extract_groupe_info_from_name(
    src: &str,
) -> Result<GroupeInfoExtract, GroupeInfoExtractError> {
    if let Some(cap) = GROUPE_RE.captures(src) {
        let activite = cap["activite"].into();
        let site = cap["site"].into();
        let grpage_min = cap["grage_min"].parse().unwrap();
        let grpage_max = cap["grage_max"].parse().unwrap();
        let semaine = cap["semaine"].parse().unwrap();
        Ok(GroupeInfoExtract {
            activite,
            site,
            grpage_max,
            grpage_min,
            semaine,
        })
    } else {
        Err(GroupeInfoExtractError {})
    }
}
impl Display for GroupeInfoExtract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | {}-{} ans | sem. {}",
            self.activite, self.site, self.grpage_min, self.grpage_max, self.semaine
        )
    }
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
