use std::fmt::Display;
use std::str::from_utf8_unchecked;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NAM([u8; 12]);
impl Display for NAM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe {from_utf8_unchecked(&self.0)})
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CAM {
    pub num: NAM,
    pub exp_mois: u8,
    pub exp_an: u32,
}