use std::{error::Error, fmt::Display};

use lazy_static::lazy_static;
use office::DataType;
use regex::Regex;

use crate::prelude::O;

pub mod excel;

#[derive(Debug, Clone, Copy)]
pub enum ExtractError {
    InvalidFormat,
    InvalidGroupNameFormat,
    CouldNotReadFile,
    MissingInformations(&'static str),
}
impl Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ExtractError: {}",
            match self {
                ExtractError::InvalidFormat => "Format de donnée invalide".into(),
                ExtractError::InvalidGroupNameFormat => "Nom de groupe invalide".into(),
                ExtractError::CouldNotReadFile => "N'a pu lire le fichier".into(),
                ExtractError::MissingInformations(s) => format!("Information manquante ({})", s),
            }
        )
    }
}
impl Error for ExtractError {}


lazy_static! {
    pub static ref TRUE_DATA_RE: Regex = Regex::new("^(?i)(?:oui|true|vrai|yes|o|y|v|t)$").unwrap();
    pub static ref FALSE_DATA_RE: Regex = Regex::new("^(?i)(?:non|false|no|faux|n|f)$").unwrap();
    pub static ref BOOL_W_COMMENT_DATA_RE: Regex = Regex::new(r"^(?i)(?P<bool>oui|true|vrai|yes|o|y|v|t|non|false|no|faux|n|f)(:?\s*,\s*(?P<comment>(?:.|\n)+)\s*)?").unwrap();
    pub static ref GROUPE_RE: Regex = Regex::new(r"^(?i)\s*(?P<activite>[^|]+?)\s*\|\s*(?P<site>[^|]+?)\s*\|\s*(?P<grage_min>\d+)\s*(?:-\s*)?(?P<grage_max>\d+)\s*ans\s*\|\s*(?:sem|semaine)\.?\s*(?P<semaine>\d+)\s*$").unwrap();
    pub static ref GROUPE_PROG_RE: Regex = Regex::new(r"^(?i)Programmation:\s*(?P<prog>.*)\s*$").unwrap();
    pub static ref DATE_NAISSANCE_RE: Regex = Regex::new(r"(?P<an>\d{4})-(?P<mois>\d{2})-(?P<jour>\d{2})").unwrap();
}

