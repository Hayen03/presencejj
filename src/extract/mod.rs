use lazy_static::lazy_static;
use office::DataType;
use regex::Regex;

use crate::prelude::O;

pub mod presence;

pub enum ExtractError {
    InvalidGroupFormat,
}

lazy_static! {
    pub static ref TRUE_DATA_RE: Regex = Regex::new("^(?i)(?:oui|true|vrai|yes|o|y|v|t)$").unwrap();
    pub static ref FALSE_DATA_RE: Regex = Regex::new("^(?i)(?:non|false|no|faux|n|f)$").unwrap();
    pub static ref BOOL_W_COMMENT_DATA_RE: Regex = Regex::new("^(?i)(?P<bool>oui|true|vrai|yes|o|y|v|t|non|false|no|faux|n|f)(:?\\s*,\\s*(?P<comment>.+?)\\s*)?").unwrap();
    pub static ref GROUPE_RE: Regex = Regex::new(r"^(?i)\s*(?P<activite>[^|]+?)\s*\|\s*(?P<site>[^|]+?)\s*\|\s*(?P<grage_min>\d+)\s*(?:-\s*)?(?P<grage_max>\d+)\s*ans\s*\|\s*(?:sem|semaine)\.?\s*(?P<semaine>\d+)\s*$").unwrap();
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
