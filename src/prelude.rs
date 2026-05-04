use std::sync::{Arc, RwLock, Weak};
use chrono::NaiveDate;
use lazy_static::lazy_static;

pub type R<T, E> = Result<T, E>;
pub type O<T> = Option<T>;
#[allow(non_camel_case_types)]
pub type bstr = Box<str>;
#[allow(non_camel_case_types)]
pub type astr = Arc<str>;
pub type A<T> = Arc<T>;
pub type W<T> = Weak<T>;
pub type Rw<T> = RwLock<T>;
pub type Arw<T> = Arc<RwLock<T>>;
pub type Date = NaiveDate;
#[allow(unused)]
pub type DateTime = chrono::NaiveDateTime;
#[allow(unused)]
pub type Time = chrono::NaiveTime;
#[allow(unused)]
pub type Days = chrono::Weekday;
#[allow(unused)]
pub type Months = chrono::Month;

lazy_static! {
	pub static ref OMIS: String = String::from("Omis");
}

pub trait DayInMonth {
	fn day_in_month(&self) -> u32;
	fn days_in_month_year(&self, year: i32) -> u32;
}
impl DayInMonth for Months {
	fn day_in_month(&self) -> u32 {
		match self {
			chrono::Month::January => 31,
			chrono::Month::February => 29,
			chrono::Month::March => 31,
			chrono::Month::April => 30,
			chrono::Month::May => 31,
			chrono::Month::June => 30,
			chrono::Month::July => 31,
			chrono::Month::August => 31,
			chrono::Month::September => 30,
			chrono::Month::October => 31,
			chrono::Month::November => 30,
			chrono::Month::December => 31,
		}
	}
	fn days_in_month_year(&self, year: i32) -> u32 {
		let n = *self as u8;
		if n == 2 {
			let leap = (year % 4 == 0) ^ (year % 100 == 0) ^ (year % 400 == 0);
			if leap {
				29
			} else {
				28
			}
		} else {
			let n = (if n >= 8 { n - 7 } else { n }) - 1;
			if n % 2 == 0 {
				31
			} else {
				30
			}
		}
	}
}
pub fn today() -> Date {
	chrono::offset::Local::now().date_naive()
}

/// provient de https://nick.groenen.me/notes/capitalize-a-string-in-rust/
/// Capitalizes the first character in s.
pub fn capitalize(s: &str) -> String {
	let mut c = s.chars();
	match c.next() {
		None => String::new(),
		Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
	}
}

#[allow(clippy::mut_from_ref)]
/**
 * # Safety
 * This function is unsafe because it allows to create a mutable reference from an immutable one, which can lead to undefined behavior if the original reference is still used after the mutable one is created. It is the caller's responsibility to ensure that the original reference is not used after calling this function, and that the mutable reference is not used after the original reference is used again.
 */
pub unsafe fn immut2mut_shenanigans<T>(var: &T) -> &mut T {
	let p: *mut T = (var as *const T) as *mut T;
	p.as_mut().unwrap()
}

pub trait Swap {
	fn swap(&mut self, other: Self) -> Self;
}
impl<T> Swap for Option<T> {
	fn swap(&mut self, other: Self) -> Self {
		match other {
			None => self.take(),
			Some(t) => self.replace(t),
		}
	}
}

pub fn slice2array<T, const N: usize>(slice: &[T]) -> Result<&[T; N], &'static str> {
	if slice.len() < N {
		return Err("Given slice is not of an appropriate element");
	}
	let pointer: *const [T; N] = slice.as_ptr() as *const [T; N];
	unsafe { pointer.as_ref().ok_or("erreur weird") }
}

pub fn print_option<T>(opt: &O<T>) -> String
where
	T: ToString,
{
	match opt {
		None => "NONE".into(),
		Some(t) => t.to_string(),
	}
}

pub fn excel_col_to_num(col: &str) -> O<u32> {
	let mut n = 0;
	for c in col.trim().to_lowercase().chars() {
		if !c.is_ascii_alphabetic() {return None;}
		n = n*26 + ((c as u32) - 96);
	}
	Some(n)
}

pub fn read_int(msg: &str) -> i64 {
	while {
		let input: String = dialoguer::Input::new().with_prompt(msg).interact_text().expect("Erreur en lisant un nombre");
		match input.parse() {
			Ok(n) => {
				return n;
			},
			Err(_) => {
				true
			},
		}
	} {}
	0
}
pub fn read_int_option(msg: &str) -> Option<i64> {
	while {
		let input: String = dialoguer::Input::new().with_prompt(msg).allow_empty(true).interact_text().expect("Erreur en lisant un nombre");
		if input.is_empty() {
			return None;
		}
		match input.parse() {
			Ok(n) => {
				return Some(n);
			},
			Err(_) => {
				true
			},
		}
	} {}
	None
}

pub fn read_string_option(msg: &str) -> Option<String> {
	let input: String = dialoguer::Input::new().with_prompt(msg).allow_empty(true).interact_text().expect("Erreur en lisant un nombre");
	if input.is_empty() {
		None
	} else {
		Some(input)
	}
}

pub type Logger<'a> = &'a dyn Fn(&str);

#[derive(Debug)]
pub struct ErrorMessage {
	msg: String,
}
impl From<&str> for ErrorMessage {
	fn from(value: &str) -> Self {
		Self { msg: value.into() }
	}
}
impl std::fmt::Display for ErrorMessage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.msg)
	}
}
impl std::error::Error for ErrorMessage {}