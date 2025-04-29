use std::{cmp::min, collections::{HashMap, HashSet}, fmt::Display, hash::{DefaultHasher, Hash, Hasher}};

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
        self.activite == other.activite &&
        self.site == other.site
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

    pub fn mk_sous_groupes(&mut self, nb_sg: usize, membres: &MembreReg) -> Result<(), ()> {
        let old_sg = self.sous_groupe.clone();
        self.sous_groupe = Vec::new();
        if nb_sg == 0 { return Ok(()); }

        // Faire la liste des candidats
        let mut candidats = Vec::new();
        for mbr in self.participants.iter() {
            match membres.get(*mbr) {
                Ok(mbr) => {
                    candidats.push(mbr);
                },
                Err(_e) => { // membre inexistant
                    self.sous_groupe = old_sg;
                    return Err(())
                }, 
            }
        }

        // nombre de participants par sous_groupe
        let sg_size = ((candidats.len() as f32)/(nb_sg as f32)).ceil() as usize;

        println!("Forme {} sous-groupe de {} pour {}", nb_sg, sg_size, self.short_desc());

        // création des sous groupes
        for disc in 1..=nb_sg {
            let mut sg = mk_sous_groupe(&candidats, sg_size);
            sg.groupe = self.id;
            sg.disc = disc as u32;
            // retirer des candidats les membres ajoutés au sg
            candidats = candidats.into_iter().filter(|mbr| !sg.contains(*mbr)).collect();

            self.sous_groupe.push(sg);
        }

        // Ajouter au sg les candidats restants
        for sg in self.sous_groupe.iter_mut() {
            while sg.participants.len() < sg_size && candidats.len() > 0 {
                sg.participants.insert(candidats.pop().unwrap().id);
            }
        }

        // S'il reste encore des candidats après ça, on envoie une erreur...
        if !candidats.is_empty() {
            self.sous_groupe = old_sg;
            return Err(());
        }

        Ok(())
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

    pub fn short_desc(&self) -> String {
        let l = [
            self.saison.as_ref().map(String::from),
            self.activite.as_ref().map(String::from),
            self.site.as_ref().map(String::from),
            self.category.as_ref().map(String::from),
            self.discriminant.as_ref().map(|s| format!("Sem. {}", s)),
        ];
        let s: String = l.into_iter().flatten().collect::<Vec<String>>().join(" | ");
        if let Some(disc) = &self.discriminant {
            s + &format!(" - {disc}")
        } else {
            s
        }
    }

    pub fn estime_cap(&self) -> usize {
        match &self.capacite {
            None => self.participants.len(),
            Some(c) => *c,
        }
    }

    pub fn get_sdj_info<'a>(&'a self) -> PresenceSDJInfo<'a> {
        PresenceSDJInfo{
            saison: self.saison.as_deref(),
            site: self.site.as_deref(),
            semaine: self.semaine.as_deref(),
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

    pub fn get_sous_groupe_for(&self, mid: MembreID) -> Option<&SousGroupe> {
        for sg in self.sous_groupe.iter() {
            if sg.participants.contains(&mid) {
                return Some(sg)
            }
        }
        None
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
        if let std::collections::hash_map::Entry::Vacant(e) = self.reg.entry(groupe.id) {
            e.insert(groupe);
            Ok(())
        } else {Err(RegError::KeyAlreadyInReg(groupe.id))}
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
    pub fn groupes(&self) -> GroupeIter<'_, impl Iterator<Item=&'_ Groupe>> {
        GroupeIter(self.reg.values())
    }
    pub fn groupes_mut(&mut self) -> GroupeIterMut<'_, impl Iterator<Item=&'_ mut Groupe>> {
        GroupeIterMut(self.reg.values_mut())
    }
    pub fn len(&self) -> usize {
        self.reg.len()
    }
}

pub fn rank_points(rank: usize) -> u32 {
    match rank {
        x if x < 1 => 8,
        1 => 4,
        2 => 2,
        _ => 0,
    }
}

pub fn mk_sous_groupe(membres: &[&Membre], nb_participants: usize) -> SousGroupe {
    // 1. Trouver si le groupe à un profil
    // Il faut premièrement cumuler tous les points des interêts des membres et voir si le plus grand couvre un certain pourcentage
    let profil ={
        let mut pts: HashMap<Interet, u32> = HashMap::new();
        for mbr in membres {
            for (rank, interet) in mbr.interets.iter().enumerate() {
                if let Some(it) = interet {
                    *pts.entry(*it).or_insert(0) += rank_points(rank);
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
                if *p > 0.4 {
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
            for i in 0..min(nb_participants, parts.len()) {
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

impl SousGroupe {
    fn contains(&self, mbr: &Membre) -> bool {
        self.participants.iter().any(|mid| mbr.id == *mid)
    }
}