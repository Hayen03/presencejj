use crate::{data::{tel::Tel, Genre, ParsingError, Taille}, prelude::*};
use std::{collections::HashMap, fmt::Display, hash::{DefaultHasher, Hash, Hasher}, str::FromStr};

use super::{comptes::CompteID, fiche_sante::FicheSante, RegError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MembreID(pub u32);
impl Display for MembreID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "M{:08x}", self.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Membre {
    pub id: MembreID,
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
    pub compte: O<CompteID>,
}
impl PartialEq for Membre {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Membre {}
impl Membre {
    pub fn new(mid: MembreID, nom: String, prenom: String, naissance: Date) -> Self {
        Self {
            id: mid,
            nom,
            prenom,
            naissance,
            ..Self::default()
        }
    }
    pub fn equiv(&self, other: &Self) -> bool {
        self.nom == other.nom &&
        self.prenom == other.prenom &&
        self.naissance == other.naissance &&
        self.compte == other.compte
    }
    pub fn get_id_seed(&self) -> u32 {
        let mut hasher = DefaultHasher::new();
        self.nom.hash(&mut hasher);
        self.prenom.hash(&mut hasher);
        self.naissance.hash(&mut hasher);
        hasher.finish() as u32
    }
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
impl FromStr for Interet {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "science" => Ok(Self::Science),
            "art" => Ok(Self::Art),
            "nature" => Ok(Self::Nature),
            "sport" => Ok(Self::Sport),
            _ => Err(ParsingError::from_msg("N'a pu lire l'intéret")),
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
    pub avec: Vec<String>,
    pub mdp: O<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Piscine {
    pub partage: O<bool>,
    pub vfi: O<bool>,
    pub tete_sous_eau: O<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct MembreReg {
    reg: HashMap<MembreID, Membre>,
}
impl MembreReg {
    pub fn get_new_id(&self) -> MembreID {
        let mut mid = MembreID(rand::random());
        while self.reg.contains_key(&mid) {
            mid = MembreID(mid.0+1)
        }
        mid
    }
    pub fn get_new_id_from_seed(&self, seed: u32) -> MembreID {
        let mut mid = MembreID(seed);
        while self.reg.contains_key(&mid) {
            mid = MembreID(mid.0+1)
        }
        mid
    }
    pub fn contains(&self, mid: MembreID) -> bool {
        self.reg.contains_key(&mid)
    }
    pub fn add(&mut self, membre: Membre) -> Result<(), RegError<MembreID>> {
        if let std::collections::hash_map::Entry::Vacant(e) = self.reg.entry(membre.id) {
            e.insert(membre);
            Ok(())
        } else {Err(RegError::KeyAlreadyInReg(membre.id))}
    }
    pub fn remove(&mut self, mid: MembreID) -> Result<Membre, RegError<MembreID>> {
        match self.reg.remove(&mid) {
            Option::None => Err(RegError::NoSuchItem(mid)),
            Option::Some(m) => Ok(m),
        }
    }
    pub fn get(&self, mid: MembreID) -> Result<&Membre, RegError<MembreID>> {
        match self.reg.get(&mid) {
            Option::None => Err(RegError::NoSuchItem(mid)),
            Option::Some(m) => Ok(m),
        }
    }
    pub fn get_mut(&mut self, mid: MembreID) -> Result<&mut Membre, RegError<MembreID>> {
        match self.reg.get_mut(&mid) {
            Option::None => Err(RegError::NoSuchItem(mid)),
            Option::Some(m) => Ok(m),
        }
    }
    pub fn membres(&self) -> MembreIter<'_, impl Iterator<Item=&'_ Membre>> {
        MembreIter(self.reg.values())
    }
    pub fn membres_mut(&mut self) -> MembreIterMut<'_, impl Iterator<Item=&'_ mut Membre>> {
        MembreIterMut(self.reg.values_mut())
    }
}

pub struct MembreIter<'a, Src: Iterator<Item=&'a Membre>> (Src);
impl<'a, Src: Iterator<Item=&'a Membre>> Iterator for MembreIter<'a, Src>  {
    type Item = &'a Membre;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    
}
pub struct MembreIterMut<'a, Src: Iterator<Item=&'a mut Membre>> (Src);
impl<'a, Src: Iterator<Item=&'a mut Membre>> Iterator for MembreIterMut<'a, Src>  {
    type Item = &'a mut Membre;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    
}