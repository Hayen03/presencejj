use crate::{data::{tel::Tel, Genre, Taille}, prelude::*};
use std::{collections::HashMap, fmt::Display};

use super::{comptes::{Compte, CompteID}, fiche_sante::FicheSante, RegError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MembreID(u32);
impl Display for MembreID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "M{}", self.0)
    }
}

#[derive(Clone, Debug)]
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
    pub fn add(&mut self, membre: Membre) -> Result<(), RegError<MembreID>> {
        if self.reg.contains_key(&membre.id) {Err(RegError::KeyAlreadyInReg(membre.id))}
        else {
            self.reg.insert(membre.id, membre);
            Ok(())
        }
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
    pub fn membres<'a>(&'a self) -> MembreIter<'a, impl Iterator<Item=&'a Membre>> {
        MembreIter(self.reg.values())
    }
    pub fn membres_mut<'a>(&'a mut self) -> MembreIterMut<'a, impl Iterator<Item=&'a mut Membre>> {
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