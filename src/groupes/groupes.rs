use std::{collections::{HashMap, HashSet}, fmt::Display, hash::{DefaultHasher, Hash, Hasher}};

use lazy_static::lazy_static;

use crate::{prelude::*, print::typst::PresenceSDJInfo};
use super::{membres::MembreID, RegError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct GroupeID(pub u32);
impl Display for GroupeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "G{:08x}", self.0)
    }
}

lazy_static!{
    pub static ref NULL_GROUPE: Groupe = Groupe::default();
}

#[derive(Debug, Clone, Default)]
pub struct Groupe {
    pub id: GroupeID,
    pub saison: O<String>,
    pub site: O<String>,
    pub category: O<String>,
    pub discriminant: O<String>,
    pub semaine: O<String>,
    pub activite: O<String>,
    pub participants: HashSet<MembreID>,
    pub sous_groupe: Vec<SousGroupe>,
}
impl PartialEq for Groupe {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Groupe {}
impl Groupe {
    pub fn new(gid: GroupeID) -> Self {
        Self {
            id: gid,
            ..Self::default()
        }
    }
    pub fn get_saison(&self) -> O<&str> {self.saison.as_ref().map(String::as_str)}
    pub fn get_site(&self) -> O<&str> {self.site.as_ref().map(String::as_str)}
    pub fn get_category(&self) -> O<&str> {self.category.as_ref().map(String::as_str)}
    pub fn get_discriminant(&self) -> O<&str> {self.discriminant.as_ref().map(String::as_str)}
   //pub fn get_animateur(&self) -> O<&str> {self.animateur.as_ref().map(String::as_str)}
    pub fn get_semaine(&self) -> O<&str> {self.semaine.as_ref().map(String::as_str)}

    pub fn has_participant(&self, mid: MembreID) -> bool {
        self.participants.contains(&mid)
    }
    pub fn add_participant(&mut self, mid: MembreID) -> bool {
        self.participants.insert(mid)
    }
    pub fn remove_participant(&mut self, mid: MembreID) -> bool {
        self.participants.remove(&mid)
    }

    pub fn equiv(&self, other: &Self) -> bool {
        self.saison == other.saison && 
        self.category == other.category &&
        self.discriminant == other.discriminant &&
        self.semaine == other.semaine &&
        self.activite == other.activite
    }
    pub fn get_id_seed(&self) -> u32 {
        let mut hasher = DefaultHasher::new();
        self.saison.hash(&mut hasher);
        self.category.hash(&mut hasher);
        self.discriminant.hash(&mut hasher);
        self.semaine.hash(&mut hasher);
        self.activite.hash(&mut hasher);
        hasher.finish() as u32
    }

    pub fn mk_sous_groupes(&mut self) {
        self.sous_groupe.clear();
        let sg = SousGroupe {
            participants: self.participants.clone(),
            ..SousGroupe::default()
        };
        self.sous_groupe.push(sg);
        // TODO: faire les vrais sous-groupes
    }

    pub fn desc(&self) -> String {
        format!("{}: {} | {} | {} | Sem. {} - {}", 
            print_option(&self.saison),
            print_option(&self.activite),
            print_option(&self.site),
            print_option(&self.category),
            print_option(&self.semaine),
            print_option(&self.discriminant),
            //print_option(&self.animateur),
        )
    }

    pub fn get_sdj_info<'a>(&'a self) -> PresenceSDJInfo<'a> {
        PresenceSDJInfo{
            saison: self.saison.as_ref().map(String::as_str),
            site: self.site.as_ref().map(String::as_str),
            semaine: self.semaine.as_ref().map(String::as_str),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GroupeReg {
    reg: HashMap<GroupeID, Groupe>,
}
impl GroupeReg {
    pub fn get_new_id(&self) -> GroupeID {
        let mut gid = GroupeID(rand::random());
        while self.reg.contains_key(&gid) {
            gid = GroupeID(gid.0+1)
        }
        gid
    }
    pub fn get_new_id_from_seed(&self, seed: u32) -> GroupeID {
        let mut gid = GroupeID(seed);
        while self.reg.contains_key(&gid) {
            gid = GroupeID(gid.0+1)
        }
        gid
    }
    pub fn contains(&self, gid: GroupeID) -> bool {
        self.reg.contains_key(&gid)
    }
    pub fn add(&mut self, groupe: Groupe) -> Result<(), RegError<GroupeID>> {
        if self.reg.contains_key(&groupe.id) {Err(RegError::KeyAlreadyInReg(groupe.id))}
        else {
            self.reg.insert(groupe.id, groupe);
            Ok(())
        }
    }
    pub fn remove(&mut self, gid: GroupeID) -> Result<Groupe, RegError<GroupeID>> {
        match self.reg.remove(&gid) {
            Option::None => Err(RegError::NoSuchItem(gid)),
            Option::Some(m) => Ok(m),
        }
    }
    pub fn get(&self, gid: GroupeID) -> Result<&Groupe, RegError<GroupeID>> {
        match self.reg.get(&gid) {
            Option::None => Err(RegError::NoSuchItem(gid)),
            Option::Some(m) => Ok(m),
        }
    }
    pub fn get_mut(&mut self, gid: GroupeID) -> Result<&mut Groupe, RegError<GroupeID>> {
        match self.reg.get_mut(&gid) {
            Option::None => Err(RegError::NoSuchItem(gid)),
            Option::Some(m) => Ok(m),
        }
    }
    pub fn groupes<'a>(&'a self) -> GroupeIter<'a, impl Iterator<Item=&'a Groupe>> {
        GroupeIter(self.reg.values())
    }
    pub fn groupes_mut<'a>(&'a mut self) -> GroupeIterMut<'a, impl Iterator<Item=&'a mut Groupe>> {
        GroupeIterMut(self.reg.values_mut())
    }
}

pub struct GroupeIter<'a, Src: Iterator<Item=&'a Groupe>> (Src);
impl<'a, Src: Iterator<Item=&'a Groupe>> Iterator for GroupeIter<'a, Src>  {
    type Item = &'a Groupe;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    
}
pub struct GroupeIterMut<'a, Src: Iterator<Item=&'a mut Groupe>> (Src);
impl<'a, Src: Iterator<Item=&'a mut Groupe>> Iterator for GroupeIterMut<'a, Src>  {
    type Item = &'a mut Groupe;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SousGroupe {
    pub profil: O<String>,
    pub disc: u32,
    pub participants: HashSet<MembreID>,
    pub groupe: GroupeID,
    pub animateur: O<String>,
}