use std::{collections::{HashMap, HashSet}, fmt::Display, hash::{DefaultHasher, Hash, Hasher}};

use lazy_static::lazy_static;
use strum::IntoEnumIterator;

use crate::{prelude::*, print::typst::PresenceSDJInfo};
use super::{membres::{Interet, Membre, MembreID, MembreReg}, RegError};

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
    pub capacite: O<usize>,
    pub age_min: O<u32>,
    pub age_max: O<u32>,
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

    pub fn guess_category(min: O<u32>, max: O<u32>) -> String {
        match (min, max) {
            (Some(5), Some(6)) => "Crocus".into(),
            (Some(7), Some(8)) => "Balaous".into(),
            (Some(9), Some(12)) => "Basaltes".into(),
            (Some(mn), Some(mx)) => format!("{}-{} ans", mn, mx),
            (Some(mn), None) => format!(">{} ans", mn),
            (None, Some(mx)) => format!("<{} ans", mx),
            (None, None) => "Tous âge".into(),
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

pub fn mk_sous_groupe(membres: &[&Membre], nb_participants: usize) -> SousGroupe {
    // 1. Trouver si le groupe à un profil
    // Il faut premièrement cumuler tous les points des interêts des membres et voir si le plus grand couvre un certain pourcentage
    let profil ={
        let mut pts: HashMap<Interet, u32> = HashMap::new();
        for mbr in membres {
            for interet in mbr.interets.iter() {
                if let Some(it) = interet {
                    *pts.entry(*it).or_insert(0) += 1;
                }
            }
        }
        let pts_sum = pts.values().sum::<u32>();
        let mut percent = HashMap::new();
        for (it, pt) in pts.iter() {
            percent.insert(it, *pt as f32 / pts_sum as f32);
        }
        let mut pro = None;
        for it in Interet::iter() {
            if let Some(p) = percent.get(&it) {
                if *p > 0.5 {
                    pro = Some(it);
                    break
                }
            }
        }
        pro
    };
    // 2. Trouver les membres qui vont dans le groupe
    // 2 situations: a. on a un profil, on prend les membres qui ont cet interêt
    //               b. on a pas de profil, donc on regroupe par age
    let mut part = HashSet::new();
    match profil {
        None => {
            let mut parts = Vec::from(membres);
            parts.sort_by(|a, b| a.naissance.cmp(&b.naissance));
            for i in 0..nb_participants {
                part.insert(parts[i].id);
            }
        },
        Some(pro) => {
            // Dans ce cas, on veut rajouter les membres qui ont l'interet dans leur premier emplacement, puis ceux dans qui l'ont dans le deuxième, jusqu'à ce qu'on ait le bon nombre
            // On ne rajoute pas ceux qui l'ont dans le troisième ou quatrième
            let filter_fn = filter_by_interet(pro);
            let sort_fn = sort_by_interet(pro);
            let mut parts: Vec<&Membre> = membres.iter().filter(|mbr| filter_fn(mbr)).map(|mbr| *mbr).collect();
            parts.sort_by(|a, b| sort_fn(a, b));
            for i in 0..(nb_participants.min(parts.len())) {
                part.insert(parts[i].id);
            }
        },
    }

    SousGroupe { 
        profil,
        participants: part,
        ..SousGroupe::default()
     }
}

fn filter_by_interet(interet: Interet) -> impl Fn(&Membre) -> bool {
    move |mbr: &Membre| {
        mbr.interets[0] == Some(interet) || mbr.interets[1] == Some(interet)
    }
}
fn sort_by_interet(interet: Interet) -> impl Fn(&Membre, &Membre) -> std::cmp::Ordering {
    move |mbr1: &Membre, mbr2: &Membre| {
        let i1 = mbr1.interets.iter().position(|int| *int == Some(interet));
        let i2 = mbr2.interets.iter().position(|int| *int == Some(interet));
        match (i1, i2) {
            (Some(i1), Some(i2)) => i1.cmp(&i2),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
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
    pub profil: O<Interet>,
    pub disc: u32,
    pub participants: HashSet<MembreID>,
    pub groupe: GroupeID,
    pub animateur: O<String>,
}