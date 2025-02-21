use std::{collections::{hash_map::Values, HashMap, HashSet}, fmt::Display, hash::{DefaultHasher, Hash, Hasher}, iter::{Filter, Map}};

use lazy_static::lazy_static;

use crate::data::adresse::Adresse;
use crate::data::tel::Tel;
use crate::prelude::*;
use crate::data::email::Email;

use super::{membres::{Membre, MembreID}, RegError};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CompteID(pub u32);
impl Display for CompteID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "C{:08x}", self.0)
    }
}

lazy_static!{
    pub static ref NULL_COMPTE: Compte = Compte::default();
}

#[derive(Debug, Clone, Default)]
pub struct Compte {
    pub id: CompteID,
    pub mandataire: String,
    pub email: O<Email>,
    pub tel: O<Tel>,
    pub adresse: O<Adresse>,
    pub membres: HashSet<MembreID>,
}
impl Compte {
    pub fn new(id: CompteID, mandataire: String) -> Self {
        Compte { id, mandataire, ..Self::default() }
    }

    pub fn contains_membre(&self, mid: MembreID) -> bool {
        self.membres.contains(&mid)
    }
    pub fn add_membre(&mut self, membre: &mut Membre) -> Result<(), CompteErr> {
        if let Some(cid) = membre.compte {
            if cid != self.id {
                return Err(CompteErr::MembreDejaDansUnCompte(membre.id));
            }
        }
        if self.membres.insert(membre.id) {
            membre.compte = Some(self.id);
            Ok(())
        }
        else {Err(CompteErr::MembreDejaExistant(membre.id))}
    }
    pub fn remove_membre(&mut self, membre: &mut Membre) -> Result<(), CompteErr> {
        if let None = membre.compte { Err(CompteErr::MembreSansCompte(membre.id)) }
        else if self.membres.remove(&membre.id) {
            membre.compte = None;
            Ok(())
        } else {Err(CompteErr::MembreInexistant(membre.id))}
    }
    pub fn equiv(&self, other: &Self) -> bool {
        self.mandataire == other.mandataire &&
        self.email == other.email &&
        self.tel == other.tel &&
        self.adresse == other.adresse
    }
    pub fn get_id_seed(&self) -> u32 {
        let mut hasher = DefaultHasher::new();
        self.mandataire.hash(&mut hasher);
        self.email.hash(&mut hasher);
        self.tel.hash(&mut hasher);
        self.adresse.hash(&mut hasher);
        hasher.finish() as u32
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompteErr {
    MembreDejaExistant(MembreID),
    MembreInexistant(MembreID),
    MembreDejaDansUnCompte(MembreID),
    MembreSansCompte(MembreID),
}

#[derive(Debug, Clone, Default)]
pub struct CompteReg {
    reg: HashMap<CompteID, Compte>,
}
impl CompteReg {
    pub fn get_new_id(&self) -> CompteID {
        let mut cid = CompteID(rand::random());
        while self.reg.contains_key(&cid) {
            cid = CompteID(cid.0+1)
        }
        cid
    }
    pub fn get_new_id_from_seed(&self, seed: u32) -> CompteID {
        let mut cid = CompteID(seed);
        while self.reg.contains_key(&cid) {
            cid = CompteID(cid.0+1)
        }
        cid
    }
    pub fn contains(&self, cid: CompteID) -> bool {
        self.reg.contains_key(&cid)
    }
    pub fn add(&mut self, compte: Compte) -> Result<(), RegError<CompteID>> {
        if self.reg.contains_key(&compte.id) {Err(RegError::KeyAlreadyInReg(compte.id))}
        else {
            self.reg.insert(compte.id, compte);
            Ok(())
        }
    }
    pub fn remove(&mut self, cid: CompteID) -> Result<Compte, RegError<CompteID>> {
        match self.reg.remove(&cid) {
            Option::None => Err(RegError::NoSuchItem(cid)),
            Option::Some(c) => Ok(c),
        }
    }
    pub fn get(&self, cid: CompteID) -> Result<&Compte, RegError<CompteID>> {
        match self.reg.get(&cid) {
            Option::None => Err(RegError::NoSuchItem(cid)),
            Option::Some(c) => Ok(c),
        }
    }
    pub fn get_mut(&mut self, cid: CompteID) -> Result<&mut Compte, RegError<CompteID>> {
        match self.reg.get_mut(&cid) {
            Option::None => Err(RegError::NoSuchItem(cid)),
            Option::Some(c) => Ok(c),
        }
    }
    pub fn search_by_name<'a>(&'a self, nom: &'a str) -> CompteIter<'a, impl Iterator<Item=&'a Compte>> {
        CompteIter(self.reg.values().filter(move |c| c.mandataire == nom))
    }
    pub fn comptes<'a>(&'a self) -> CompteIter<'a, impl Iterator<Item=&'a Compte>> {
        CompteIter(self.reg.values())
    }
    pub fn comptes_mut<'a>(&'a mut self) -> CompteIterMut<'a, impl Iterator<Item=&'a mut Compte>> {
        CompteIterMut(self.reg.values_mut())
    }
}

pub struct CompteIter<'a, Src: Iterator<Item=&'a Compte>> (Src);
impl<'a, Src: Iterator<Item=&'a Compte>> Iterator for CompteIter<'a, Src>  {
    type Item = &'a Compte;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    
}
pub struct CompteIterMut<'a, Src: Iterator<Item=&'a mut Compte>> (Src);
impl<'a, Src: Iterator<Item=&'a mut Compte>> Iterator for CompteIterMut<'a, Src>  {
    type Item = &'a mut Compte;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    
}