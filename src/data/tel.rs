use crate::prelude::*;
use std::{fmt::{self}, str};

use super::{ParsingError, ParsingResult};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
	static ref TEL_RE: Regex = Regex::new(r"^\s*(?:\+?(?<pays>\d)\s*-?\s*)?(?<reg>(?:\d{3})|(?:\(\s*\d{3}\s*\)))\s*-?\s*(?<a>\d{3})\s*-?\s*(?<b>\d{4})\s*$").unwrap();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct Tel([u8; 10]);
#[allow(unused)]
impl Tel {
	// juste pour des numero canadien
	pub fn parse(orig: &str) -> ParsingResult<Self> {
		let cap = TEL_RE.captures(orig);
		match cap {
			None => Err(ParsingError::from_msg("Numéro de téléphone invalide")),
			Some(c) => {
				let mut s = String::new();
				if c["reg"].len() == 3 {
					s.push_str(&c["reg"]);
				} else {
					s.push_str(&c["reg"][1..4]);
				}
				
				s.push_str(&c["a"]);
				s.push_str(&c["b"]);
				let sb = s.as_bytes();
				Ok(Tel([
					sb[0], sb[1], sb[2], sb[3], sb[4], sb[5], sb[6], sb[7], sb[8], sb[9],
				]))
			}
		}
	}
	pub fn as_str(&self) -> &str {
		unsafe { str::from_utf8_unchecked(&self.0) }
	}
	pub fn compact(&self) -> bstr {
		let s = format!("+1{}", self.as_str());
		bstr::from(s)
	}
	/// # Safety
	/// ne fait pas les vérifications habituelles et ne s'assure pas que
	/// a) ce sont des charactères utf-8 valides
	/// b) c'est un numéro de téléphone canadien valide
	pub unsafe fn create(val: [u8; 10]) -> Tel {
		Tel(val)
	}
}
impl fmt::Display for Tel {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = self.as_str();
		write!(f, "+1 ({}) {}-{}", &s[..3], &s[3..6], &s[6..])
	}
}
impl Default for Tel {
	fn default() -> Self {
		unsafe { Self::create([48u8; 10]) } // 000-000-0000
	}
}
/*
impl PolarsObject for Tel {
	fn type_name() -> &'static str {
		"Tel"
	}
}
*/
impl str::FromStr for Tel {
	type Err = ParsingError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(s)
	}
}