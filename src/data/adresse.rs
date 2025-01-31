
use std::{
	borrow::Borrow, collections::BTreeMap, fmt::{self, Display}, num, str, sync::{Arc, RwLock}
};

use crate::{prelude::*};


use lazy_static::lazy_static;
use regex::Regex;

use super::{ParsingError, ParsingResult};

lazy_static! {
	pub static ref ADRESSE_RUE_REGEX: Regex =
		Regex::new(r"^\s*(?:(?P<app>\d+)\s*-\s*)?(?P<num>\d+)?(?:\s*,\s*|\s+)(?P<rue>.+?)\s*$")
			.unwrap();
	pub static ref ADRESSE_FULL_REGEX: Regex = Regex::new(r"^\s*(?:\w\s*:\s*)?(?P<num>\d+),?\s*(?P<rue>[a-zA-Z0-9éÉàÀùÙÇçïÏôÔêÊèÈ\- .]+)(?:\s*#(?P<app>\d+))?\s+,?(?P<ville>[a-zA-Z0-9éÉàÀùÙÇçïÏôÔêÊèÈ\- .]+),\s*(?P<province>[a-zA-Z0-9éÉàÀùÙÇçïÏôÔêÊèÈ\- .]+)\s*,\s*(?P<pays>[a-zA-Z0-9éÉàÀùÙÇçïÏôÔêÊèÈ\- .]+)\s*,\s*(?P<codepostal>[a-zA-Z0-9 ]+)$").unwrap();
}

#[derive(Debug, Clone, Default)]
pub struct Adresse {
	pub numero: O<i32>,
	pub rue: O<String>,
	pub appartement: O<i32>,
	pub code_postal: O<CodePostal>,
	pub pays: O<Pays>,
	pub province: O<Province>,
	pub ville: O<Ville>,
}
impl Adresse {
	pub fn from_num_rue(src: &str) -> Self {
		let mut adr = Adresse::default();
		if let Some(cap) = ADRESSE_RUE_REGEX.captures(src) {
			if let Some(num) = cap.name("num") {
				let num: i32 = num.as_str().parse().unwrap();
				adr.numero = Some(num);
			}
			if let Some(rue) = cap.name("rue") {
				adr.rue = Some(rue.as_str().into());
			}
			if let Some(app) = cap.name("app") {
				let app: i32 = app.as_str().parse().unwrap();
				adr.appartement = Some(app);
			}
		}
		adr
	}
	pub fn full(&self) -> String {
		let mut first = true;
		let mut virgule = false;
		let mut s = String::new();

		if let Some(rue) = &self.rue {
			if let Some(num) = self.numero {
				s += &format!("{num}");
				if let Some(app) = self.appartement {
					s += &format!(" app. {app}");
				}
				first = false;
			}
			if first {
				first = false;
			} else {
				s += " ";
			}
			s += &rue;
		}
		if let Some(ville) = &self.ville {
			if first {
				first = false;
			} else if virgule {
				s += " ";
			} else {
				s += ", ";
				virgule = true;
			}
			s += ville.borrow();
		}
		if let Some(province) = &self.province {
			if first {
				first = false;
			} else if virgule {
				s += " ";
			} else {
				s += ", ";
				virgule = true;
			}
			s += province.borrow();
		}
		if let Some(pays) = &self.pays {
			if first {
				first = false;
			} else if virgule {
				s += " ";
			} else {
				s += ", ";
				virgule = true;
			}
			s += pays.borrow();
		}
		if let Some(cp) = &self.code_postal {
			if first {
				first = false;
			} else if virgule {
				s += " ";
			} else {
				s += ", ";
				virgule = true;
			}
			s += cp.as_str();
		}

		// format!("{num} {rue} app. {app}, {ville} {prov} {pays} {cp}")
		s
	}
	pub fn from_full(full: &str) -> Result<Self, ()> {
		let mut adr = Adresse::default();
		if let Some(cap) = ADRESSE_FULL_REGEX.captures(full) {
			adr.numero = Some(cap.name("num").unwrap().as_str().parse().unwrap());
			adr.rue = Some(cap.name("rue").unwrap().as_str().into());
			if let Some(app) = cap.name("app") {
				let app: i32 = app.as_str().parse().unwrap();
				adr.appartement = Some(app);
			}
			adr.ville = Some(cap.name("ville").unwrap().as_str().into());
			adr.province = Some(cap.name("province").unwrap().as_str().into());
			let mut pays: String = cap.name("rue").unwrap().as_str().into();
			if pays == "CA" {
				pays = String::from("Canada");
			}
			adr.pays = Some(Pays::from(pays.as_str()));
			let code_postal = match CodePostal::parse(cap.name("codepostal").unwrap().as_str()) {
				Ok(cp) => Some(cp),
				Err(_) => return Err(()),
			};
		}
		Ok(adr)
	}
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Pays {
	ptr: Arc<str>,
}
impl Pays {
	pub fn as_str(&self) -> &str {
		&self.ptr
	}
}
impl Borrow<str> for Pays {
	fn borrow(&self) -> &str {
		&self.ptr
	}
}
impl Display for Pays {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", &self.ptr)
	}
}
impl From<&str> for Pays {
	fn from(value: &str) -> Self {
		//PAYS_REGISTRY.get(value)
		Self {ptr: Arc::from(value)}
	}
}
/*
impl SimpleRegItem<str> for Pays {
	fn ref_ptr(&self) -> &Arc<str> {
		&self.ptr
	}
	fn arc_from_ref(src: &str) -> Arc<str> {
		Arc::from(src)
	}
}
*/
impl From<Arc<str>> for Pays {
	fn from(value: Arc<str>) -> Self {
		Self { ptr: value }
	}
}
/*
lazy_static! {
	static ref PAYS_REGISTRY: SimpleReg<Pays, str> = SimpleReg::new();
}
*/

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Province {
	ptr: Arc<str>,
}
impl Province {
	pub fn as_str(&self) -> &str {
		&self.ptr
	}
}
impl Borrow<str> for Province {
	fn borrow(&self) -> &str {
		&self.ptr
	}
}
impl Display for Province {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", &self.ptr)
	}
}
impl From<&str> for Province {
	fn from(value: &str) -> Self {
		//PROVINCE_REGISTRY.get(value)
		Self {ptr: Arc::from(value)}
	}
}
/* impl SimpleRegItem<str> for Province {
	fn ref_ptr(&self) -> &Arc<str> {
		&self.ptr
	}
	fn arc_from_ref(src: &str) -> Arc<str> {
		Arc::from(src)
	}
} */
impl From<Arc<str>> for Province {
	fn from(value: Arc<str>) -> Self {
		Self { ptr: value }
	}
}
/* lazy_static! {
	static ref PROVINCE_REGISTRY: SimpleReg<Province, str> = SimpleReg::new();
} */

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Ville {
	ptr: Arc<str>,
}
impl Ville {
	pub fn as_str(&self) -> &str {
		&self.ptr
	}
}
impl Borrow<str> for Ville {
	fn borrow(&self) -> &str {
		&self.ptr
	}
}
impl Display for Ville {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", &self.ptr)
	}
}
impl From<&str> for Ville {
	fn from(value: &str) -> Self {
		//VILLE_REGISTRY.get(value)
		Self {ptr: Arc::from(value)}
	}
}
/* struct VilleReg {
	reg: RwLock<BTreeMap<Ville, ()>>,
} */
/* impl SimpleRegItem<str> for Ville {
	fn ref_ptr(&self) -> &Arc<str> {
		&self.ptr
	}
	fn arc_from_ref(src: &str) -> Arc<str> {
		Arc::from(src)
	}
} */
impl From<Arc<str>> for Ville {
	fn from(value: Arc<str>) -> Self {
		Self { ptr: value }
	}
}
/* lazy_static! {
	static ref VILLE_REGISTRY: SimpleReg<Ville, str> = SimpleReg::new();
} */

/*
#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
pub struct Adresse {
	pub num: O<astr>,
	pub rue: O<astr>,
	pub ville: O<astr>,
	pub code_postal: O<CodePostal>,
	pub app: O<astr>,
}
#[allow(unused)]
impl Adresse {
	pub fn parse(s: &str) -> Result<Self, &str> {
		match ADR_RE.captures(s) {
			None => Err("Format d'adresse invalide"),
			Some(cap) => {
				let num = cap.name("num").unwrap().as_str();
				let code_postal = CodePostal::parse(cap.name("code").unwrap().as_str()).unwrap();
				let rue = match cap.name("rue") {
					None => cap.name("ruewapp").unwrap().as_str(),
					Some(r) => r.as_str(),
				};
				let ville = match cap.name("ville") {
					None => cap.name("villewapp").unwrap().as_str(),
					Some(v) => v.as_str(),
				};
				let app = cap.name("app").map(|a| astr::from(a.as_str()));
				Ok(Adresse {
					num: Some(astr::from(num)),
					code_postal: Some(code_postal),
					rue: Some(astr::from(rue)),
					ville: Some(astr::from(ville)),
					app,
				})
			}
		}
	}
	pub fn new(
		num: Option<astr>,
		rue: Option<astr>,
		ville: Option<astr>,
		cp: Option<CodePostal>,
		app: Option<astr>,
	) -> Self {
		Adresse {
			num,
			rue,
			ville,
			code_postal: cp,
			app,
		}
	}
	pub fn sep_num_rue(src: &str) -> Result<(Option<bstr>, bstr), ParsingError> {
		if let Some(cap) = NUMRUE_RE.captures(src) {
			let num = cap.name("num").map(|s| bstr::from(s.as_str()));
			let rue = bstr::from(&cap["rue"]);
			Ok((num, rue))
		} else {
			Err(ParsingError::from_msg("Mauvais format de num-rue"))
		}
	}
}
impl fmt::Display for Adresse {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.app {
			Some(n) => write!(
				f,
				"{} {} #{}, {}, {}",
				if let Some(ref s) = self.num { s } else { "" },
				if let Some(ref s) = self.rue { s } else { "" },
				n,
				if let Some(ref s) = self.ville { s } else { "" },
				if let Some(ref s) = self.code_postal {
					s.as_str()
				} else {
					""
				}
			),
			_ => write!(
				f,
				"{} {}, {}, {}",
				if let Some(ref s) = self.num { s } else { "" },
				if let Some(ref s) = self.rue { s } else { "" },
				if let Some(ref s) = self.ville { s } else { "" },
				if let Some(ref s) = self.code_postal {
					s.as_str()
				} else {
					""
				}
			),
		}
	}
}
*/

lazy_static! {
	pub static ref ZIP_RE: Regex =
		Regex::new(r"^\s*([A-Za-z]\d[A-Za-z])\s*(\d[A-Za-z]\d)\s*$").unwrap();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct CodePostal([u8; 6]);
#[allow(unused)]
impl CodePostal {
	pub fn parse(s: &str) -> ParsingResult<Self> {
		match ZIP_RE.captures(s) {
			None => Err(ParsingError::from_msg("Format d'email invalide")),
			Some(cap) => {
				let ta = cap[1].to_uppercase();
				let tb = cap[2].to_uppercase();
				let a = ta.as_bytes();
				let b = tb.as_bytes();
				Ok(CodePostal([a[0], a[1], a[2], b[0], b[1], b[2]]))
			}
		}
	}
	pub fn try_new(mut arr: [u8; 6]) -> ParsingResult<Self> {
		// check si le lettres sont valide:
		for i in [0usize, 2, 4] {
			let mut c = arr[i];
			if c.is_ascii_lowercase() || c.is_ascii_uppercase() {
				if c > b'Z' {
					c ^= b' ';
					arr[i] = c;
				}
				if c == b'D' || c == b'F' || c == b'I' || c == b'O' || c == b'Q' || c == b'U' {
					return Err(ParsingError::from_msg_at(
						&format!(
							"'{}' n'est jamais utilisée dans un code postal canadien",
							c as char
						),
						i,
					));
				} else if i == 0 && (c == b'W' || c == b'Z') {
					return Err(ParsingError::from_msg_at(&format!("'{}' n'est jamais utilisée comme premier caractère dans un code postal canadien", c as char), i));
				}
			} else {
				return Err(ParsingError::from_msg_at(
					&format!("Le {}ième caractère du code doit être une lettre", i + 1),
					i,
				));
			}
		}
		// check si les chiffres sont valide
		for i in [1usize, 3, 5] {
			let c = arr[i];
			if !c.is_ascii_digit() {
				return Err(ParsingError::from_msg_at(
					&format!("Le {}ème caractère doit être un chiffre", i + 1),
					i,
				));
			}
		}
		Ok(Self(arr))
	}
	pub fn as_str(&self) -> &str {
		unsafe { str::from_utf8_unchecked(&self.0) }
	}
	/// # Safety
	/// creates a zip code without checking that
	/// a) it is valid utf-8 characters
	/// b) it is a valid zip code
	pub unsafe fn create(val: [u8; 6]) -> CodePostal {
		CodePostal(val)
	}
}
impl fmt::Display for CodePostal {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = self.as_str();
		write!(f, "{} {}", &s[..3], &s[3..])
	}
}
impl Default for CodePostal {
	fn default() -> Self {
		CodePostal([b'A', b'0', b'A', b'0', b'A', b'0'])
	}
}
/*
impl PolarsObject for CodePostal {
	fn type_name() -> &'static str {
		"CodePostal"
	}
}
*/
impl str::FromStr for CodePostal {
	type Err = ParsingError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(s)
	}
}

lazy_static! {
	pub static ref ADR_RE: Regex = Regex::new(r"(:?[\w-]+\s*:\s*)?(?P<num>\d+)\s*,?\s*(?:(?:(?P<rue>[\w\s-]+?)\s*(?P<ville>[\w-]+))|(?:(?P<ruewapp>[\w\s-]+?)\s*#(?:(?P<app>\d+)|(?P<falseapp>-))\s*,\s*(?P<villewapp>[\w-]+)))\s*,\s*(?P<province>[\w -]+?)\s*,\s*(?P<pays>[\w -]+?)\s*,\s*(?P<code>[A-Za-z]\d[A-Za-z]\s*\d[A-Za-z]\d)").unwrap();
	pub static ref NUMRUE_RE: Regex = Regex::new(r"^\s*(?P<num>\d+)?\s*(?:,\s*)?(?P<rue>.+?)\s*$").unwrap();
}