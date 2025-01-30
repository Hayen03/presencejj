use std::sync::{Arc, RwLock, Weak};
use chrono::NaiveDateTime;

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
pub type Date = NaiveDateTime;