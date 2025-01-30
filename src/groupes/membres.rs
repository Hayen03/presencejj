use crate::{data::{tel::Tel, Genre, Taille}, prelude::*};
use std::fmt::Display;

use super::{comptes::Compte, fiche_sante::FicheSante};

struct Membre {
    pub nom: String,
    pub prenom: String,
    pub naissance: Date,
    pub genre: O<Genre>,
    pub fiche_sante: Box<FicheSante>,
    pub interets: Interets,
    pub contacts: [O<Contact>; 2],
    pub accompagnement: O<bool>,
    pub quitte: Quitte,
    pub piscine: Piscine,
    pub auth_photo: O<bool>,
    pub taille: O<Taille>,
    pub commentaire: O<String>,
    pub compte: O<Arw<Compte>>,
}

pub type Interets = [O<Interet>; 4];
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Interet {
    Science,
    Sport,
    Art,
    Nature,
}
impl Display for Interet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Science => write!(f, "Science"),
            Self::Art => write!(f, "Art"),
            Self::Nature => write!(f, "Nature"),
            Self::Sport => write!(f, "Sport"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Contact {
    pub nom: String,
    pub tel: O<Tel>,
    pub lien: O<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Quitte {
    avec: Vec<String>,
    mdp: O<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Piscine {
    partage: O<bool>,
    vfi: O<bool>,
    tete_sous_eau: O<bool>,
}