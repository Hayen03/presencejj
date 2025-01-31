use std::{fmt, str};

use crate::prelude::astr;

use super::{ParsingError, ParsingResult};

use lazy_static::lazy_static;
use regex::Regex;


lazy_static! {
	static ref EMAIL_RE: Regex = Regex::new(r"^\s*((?P<user>[a-zA-Z0-9!#$%&'*+\-/=?^_`{|}~]+(:?\.[a-zA-Z0-9!#$%&'*+\-/=?^_`{|}~]+)*)\s*@\s*(?P<domain>(:?[a-zA-Z0-9]+(:?-[a-zA-Z0-9]+)*)(:?\.(:?[a-zA-Z0-9]+(:?-[a-zA-Z0-9]+)*))*))\s*$").unwrap(); // on rajoute des espaces autour du @ à cause de la manière que proc_macro traite les entres
	static ref EMAIL_EMPTY: astr = astr::from("@");
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct Email {
	value: astr,
	sep: usize,
}
#[allow(unused)]
impl Email {
	pub fn parse(s: &str) -> ParsingResult<Email> {
		match EMAIL_RE.captures(s) {
			None => Err(ParsingError::from_msg("Format d'email invalide")),
			Some(cap) => {
				let domain = &cap["domain"];
				let user = &cap["user"];
				let s = format!("{}@{}", user, domain);
				Ok(Email {
					value: astr::from(s),
					sep: user.len(),
				})
			}
		}
	}
	/// # Safety
	/// creates an email without checking the format, thus it is the user's job to make sure it is
	/// a) valid utf-8 characters
	/// b) valid email
	///
	pub unsafe fn create(s: astr, sep: usize) -> Email {
		Email { value: s, sep }
	}
	pub fn as_str(&self) -> &str {
		&self.value
	}
	pub fn domain(&self) -> &str {
		&self.value[self.sep + 1..]
	}
	pub fn user(&self) -> &str {
		&self.value[..self.sep]
	}
	pub fn is_empty(&self) -> bool {
		self.value.eq(&EMAIL_EMPTY)
	}
}
impl fmt::Display for Email {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.value)
	}
}
impl Default for Email {
	fn default() -> Self {
		Email {
			value: EMAIL_EMPTY.clone(),
			sep: 0,
		}
	}
}
/*
impl PolarsObject for Email {
	fn type_name() -> &'static str {
		"Email"
	}
}
*/
impl str::FromStr for Email {
	type Err = ParsingError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(s)
	}
}