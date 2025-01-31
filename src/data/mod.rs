use crate::prelude::*;
use std::{error::Error, fmt::{self, Display}};
use lazy_static::lazy_static;
use regex::Regex;

pub mod adresse;
pub mod cam;
pub mod tel;
pub mod email;

lazy_static! {
	pub static ref email_re: Regex = Regex::new(r"([\w\-.]+)@(?:([\w-])+\.)+([\w-]+)").unwrap();
	pub static ref tel_re: Regex = Regex::new(r"(?:\+?(\d)\s*-?\s*)?((?:\d{3})|(?:\(\s*\d{3}\s*\)))\s*-?\s*(\d{3})\s*-?\s*(\d{4})").unwrap();
	pub static ref ZIP_re: Regex = Regex::new(r"([A-Za-z]\d[A-Za-z])\s*(\d[A-Za-z]\d)").unwrap();
	pub static ref NAM_re: Regex = Regex::new(r"((?P<nam1>[A-Z]{4})\s*(?P<nam2>\d{4})\s*(?P<nam3>\d{4})) - (?P<expan>\d{4})(?:/|-)(?P<expmois>\d{2})").unwrap();
	pub static ref date_naissance_re: Regex = Regex::new(r"(?P<an>\d{4})-(?P<mois>\d{2})-(?P<jour>\d{2})(?:\s*\((?P<age>\d+)\s+ans\))").unwrap();
	pub static ref adr_re: Regex = Regex::new(r"(:?[\w-]+\s*:\s*)?(?P<num>\d+)\s*,?\s*(?:(?:(?P<rue>[\w\s-]+?)\s*(?P<ville>[\w-]+))|(?:(?P<ruewapp>[\w\s-]+?)\s*#(?:(?P<app>\d+)|(?P<falseapp>-))\s*,\s*(?P<villewapp>[\w-]+)))\s*,\s*(?P<province>[\w -]+?)\s*,\s*(?P<pays>[\w -]+?)\s*,\s*(?P<code>[A-Za-z]\d[A-Za-z]\s*\d[A-Za-z]\d)").unwrap();
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum Genre {
    #[default]
    Homme,
    Femme,
    Autre,
}
impl Display for Genre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Autre => write!(f, "Autre"),
            Self::Homme => write!(f, "Homme"),
            Self::Femme => write!(f, "Femme"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Taille {
    XS,
    S,
    M,
    L,
    XL,
    XXL,
}
impl Display for Taille {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::XS => write!(f, "XS"),
            Self::S => write!(f, "S"),
            Self::M => write!(f, "M"),
            Self::L => write!(f, "L"),
            Self::XL => write!(f, "XL"),
            Self::XXL => write!(f, "XXL"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoolJustifie {
    pub reponse: bool,
    pub justification: O<String>,
}

#[derive(Debug)]
pub struct ParsingError {
	msg: Option<bstr>,
	at: Option<usize>,
	src: Option<Box<dyn Error>>,
}
#[allow(unused)]
impl ParsingError {
	pub fn new(msg: Option<bstr>, at: Option<usize>, src: Option<Box<dyn Error>>) -> Self {
		ParsingError { msg, at, src }
	}
	pub fn from_msg(msg: &str) -> Self {
		ParsingError {
			msg: Some(bstr::from(msg)),
			at: None,
			src: None,
		}
	}
	pub fn from_msg_at(msg: &str, at: usize) -> Self {
		ParsingError {
			msg: Some(bstr::from(msg)),
			at: Some(at),
			src: None,
		}
	}
	pub fn from_at(at: usize) -> Self {
		ParsingError {
			msg: None,
			at: Some(at),
			src: None,
		}
	}
	pub fn from_src<E: Error + 'static>(src: E) -> Self {
		ParsingError {
			msg: None,
			at: None,
			src: Some(Box::new(src)),
		}
	}
	pub fn from_msg_src<E: Error + 'static>(msg: &str, src: E) -> Self {
		ParsingError {
			msg: Some(bstr::from(msg)),
			at: None,
			src: Some(Box::new(src)),
		}
	}
	pub fn from_src_at<E: Error + 'static>(src: E, at: usize) -> Self {
		ParsingError {
			msg: None,
			at: Some(at),
			src: Some(Box::new(src)),
		}
	}
	pub fn from_msg_src_at<E: Error + 'static>(msg: &str, src: E, at: usize) -> Self {
		ParsingError {
			msg: Some(bstr::from(msg)),
			at: Some(at),
			src: Some(Box::new(src)),
		}
	}
}
impl fmt::Display for ParsingError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let msg_str = match &self.msg {
			Some(msg) => format!(": {}", msg),
			None => String::new(),
		};
		let at_str = match &self.at {
			Some(at) => format!(" at char {}", at),
			None => String::new(),
		};
		write!(f, "ParsingError{}{}", at_str, msg_str)
	}
}
impl Error for ParsingError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match &self.src {
			Some(e) => Some(e.as_ref()),
			None => None,
		}
	}
}
#[allow(unused)]
pub type ParsingResult<T> = Result<T, ParsingError>;