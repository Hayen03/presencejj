use std::{borrow::Borrow, fmt, str::{self, FromStr}};

use crate::prelude::{slice2array, today, Date, DayInMonth, Months};

use super::{Genre, ParsingError, ParsingResult};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
	pub static ref NAM_RE: Regex =
		Regex::new(r"(?P<nam>(?i)(?P<nama>[a-zA-Z]{4})\s*(?P<namb>[0-9]{4})\s*(?P<namc>[0-9]{4}))")
			.unwrap();
	pub static ref CAM_RE: Regex = Regex::new(
		format!(
			r"{}\s*?(:?\s|-)\s*(?P<exp>(?P<expan>[0-9]{{4}})\s*/\s*(?P<expm>[0-9]{{2}}))",
			NAM_RE.as_str()
		)
		.as_str()
	)
	.unwrap();
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Default, Copy)]
pub struct NAM([u8; 12]);
impl AsRef<[u8; 12]> for NAM {
	fn as_ref(&self) -> &[u8; 12] {
		&self.0
	}
}
impl Borrow<[u8; 12]> for NAM {
	fn borrow(&self) -> &[u8; 12] {
		&self.0
	}
}
impl AsRef<[u8]> for NAM {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}
impl Borrow<[u8]> for NAM {
	fn borrow(&self) -> &[u8] {
		&self.0
	}
}
impl NAM {
	pub fn as_str(&self) -> &str {
		unsafe { str::from_utf8_unchecked(&self.0) }
	}
	pub fn parse(s: &str) -> ParsingResult<NAM> {
		if let Some(caps) = NAM_RE.captures(s.trim()) {
			let nama = caps["nama"].to_uppercase();
			let namb = &caps["namb"];
			let namc = &caps["namc"];
			let sb = nama + namb + namc;
			let ba = sb.as_bytes();
			// validation du numero
			let nmois: u8 = sb[6..8].parse().unwrap();
			let nmois = if nmois > 50 { nmois - 50 } else { nmois };
			if nmois == 0 || nmois > 12 {
				return Err(ParsingError::from_msg("NAM invalide (mois de naissance)"));
			}
			let njour: u32 = sb[8..10].parse().unwrap();
			if njour == 0 || njour > Months::try_from(nmois).unwrap().day_in_month() {
				return Err(ParsingError::from_msg("NAM invalide (jour de naissance)"));
			}
			Ok(NAM(*slice2array::<u8, 12>(ba).unwrap()))
		} else {
			Err(ParsingError::from_msg("Format de NAM invalide"))
		}
	}
	pub fn nom(&self) -> &str {
		unsafe { str::from_utf8_unchecked(&self.0[..3]) }
	}
	pub fn prenom(&self) -> &str {
		unsafe { str::from_utf8_unchecked(&self.0[3..4]) }
	}
	pub fn an_naissance(&self) -> u8 {
		unsafe { str::from_utf8_unchecked(&self.0[4..6]).parse().unwrap() }
	}
	pub fn mois_naissance(&self) -> Months {
		let m: u8 = unsafe { str::from_utf8_unchecked(&self.0[6..8]).parse().unwrap() };
		let m = if m > 50 { m - 50 } else { m };
		Months::try_from(m).unwrap()
	}
	pub fn genre(&self) -> Genre {
		let m: u8 = unsafe { str::from_utf8_unchecked(&self.0[6..8]).parse().unwrap() };
		if m > 50 {
			Genre::Femme
		} else {
			Genre::Homme
		}
	}
	pub fn jour_naissance(&self) -> u8 {
		unsafe { str::from_utf8_unchecked(&self.0[8..10]).parse().unwrap() }
	}
	pub fn valid(&self) -> bool {
		let nmois: u8 = self.mois_naissance() as u8;
		if nmois == 0 || nmois > 12 {
			return false;
		}
		let njour: u32 = self.jour_naissance() as u32;
		if njour == 0 || njour > Months::try_from(nmois).unwrap().day_in_month() {
			return false;
		}
		true
	}
	/// # Safety
	/// ne fait pas les vérifications habituelles et ne s'assure pas que
	/// a) ce sont des charactères utf-8 valides
	/// b) c'est un NAM valide
	pub unsafe fn create(num: [u8; 12]) -> NAM {
		NAM(num)
	}
}
impl FromStr for NAM {
	type Err = ParsingError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(s)
	}
}
impl fmt::Display for NAM {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Default, Copy)]
pub struct CAM {
	num: NAM,
	exp: (i32, u8),
}
impl CAM {
	pub fn parse(s: &str) -> ParsingResult<CAM> {
		if let Some(caps) = CAM_RE.captures(s.trim()) {
			let nama = caps["nama"].to_uppercase();
			let namb = &caps["namb"];
			let namc = &caps["namc"];
			let sb = nama + namb + namc;
			let an: i32 = caps["expan"].parse().unwrap();
			let mois: u8 = caps["expm"].parse().unwrap();
			let ba = sb.as_bytes();
			if 0 == mois || mois > 12 {
				return Err(ParsingError::from_msg("Mois d'expiration invalide"));
			}
			// validation du numero
			let nmois: u8 = sb[6..8].parse().unwrap();
			let nmois = if nmois > 50 { nmois - 50 } else { nmois };
			if nmois == 0 || nmois > 12 {
				return Err(ParsingError::from_msg("NAM invalide (mois de naissance)"));
			}
			let njour: u32 = sb[8..10].parse().unwrap();
			if njour == 0 || njour > Months::try_from(nmois).unwrap().day_in_month() {
				return Err(ParsingError::from_msg("NAM invalide (jour de naissance)"));
			}
			let nam = unsafe { NAM::create(*slice2array::<u8, 12>(ba).unwrap()) };
			Ok(CAM {
				num: nam,
				exp: (an, mois),
			})
		} else {
			Err(ParsingError::from_msg("Format de NAM invalide"))
		}
	}
	pub fn expiration(&self) -> Date {
		Date::from_ymd_opt(
			self.exp.0,
			self.exp.1 as u32,
			Months::try_from(self.exp.1)
				.unwrap()
				.days_in_month_year(self.exp.0),
		)
		.unwrap()
	}
	pub fn is_expired_at(&self, at: Date) -> bool {
		at < self.expiration()
	}
	pub fn is_expired(&self) -> bool {
		today() < self.expiration()
	}
	pub fn numero(&self) -> NAM {
		self.num
	}
	/// # Safety
	/// Ne fait pas les vérifications habituelles. L'utilisateur doit donc s'assurer que
	/// a) le NAM est composé de caractère utf-8 valide
	/// b) le NAM est valide
	/// c) la date d'expiration est valide
	pub unsafe fn create(num: NAM, expan: i32, expm: u8) -> CAM {
		CAM {
			num,
			exp: (expan, expm),
		}
	}
}
impl fmt::Display for CAM {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{} {:0<4}/{:0<2}",
			self.num,
			self.exp.0 % 10000,
			self.exp.1
		)
	}
}
impl FromStr for CAM {
	type Err = ParsingError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(s)
	}
}