use crate::prelude::*;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tel([u8; 11]);
impl Display for Tel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       // let s = format!();
        write!(f, "+{} ({}{}{}) {}{}{}-{}{}{}{}", self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7], self.0[8], self.0[9], self.0[10])
    }
}