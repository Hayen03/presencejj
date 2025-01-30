use crate::prelude::*;
use std::fmt::Display;

pub mod adresse;
pub mod cam;
pub mod tel;


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