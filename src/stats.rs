use std::collections::{BTreeSet, HashMap};

use crate::{data::Taille, groupes::{groupes::{Groupe, GroupeReg}, membres::{MembreID, MembreReg}}, prelude::read_int};


struct ChandailCalcul {
    capacite: usize,
    comptes: HashMap<Taille, usize>,
}
impl ChandailCalcul {
    fn new() -> Self {
        let mut comptes = HashMap::new();
        for t in Taille::tailles() {
            comptes.insert(*t, 0);
        }

        Self {
            capacite: 0,
            comptes,
        }
    }
    fn total(&self) -> usize {
        let mut tot = 0;
        for (_, c) in self.comptes.iter() {
            tot += c;
        }
        tot
    }
}

fn update_calcul_chandail(cc: &mut ChandailCalcul, grp: &Groupe, membres: &MembreReg, tried: &mut BTreeSet<MembreID>) {
    for p in grp.participants.iter() {
        let participant = membres.get(*p).expect("Membre inexistant");
        if !tried.contains(p) {
            if let Some(taille) = participant.taille {
                *cc.comptes.get_mut(&taille).unwrap() += 1;
            }
            cc.capacite += 1;
            tried.insert(*p);
        }
    }
}

pub fn calcul_chandail(groupes: &GroupeReg, membres: &MembreReg) -> HashMap<Taille, usize> {
    let mut chandails_cat: HashMap<String, ChandailCalcul> = HashMap::new();
    let mut tried = BTreeSet::new();
    for groupe in groupes.groupes() {
        let cat = groupe.category.as_ref().map_or("None".into(), String::clone);
        let cc = {
            if !chandails_cat.contains_key(&cat) {
                chandails_cat.insert(cat.clone(), ChandailCalcul::new());
            }
            chandails_cat.get_mut(&cat).unwrap()
        };
        update_calcul_chandail(cc, groupe, membres, &mut tried);
    }

    let mut res = HashMap::new();
    for t in Taille::tailles() {
        res.insert(*t, 0);
    }

    for (_, cc) in chandails_cat {
        let total = cc.total() as f32;
        let cap = cc.capacite as f32;
        for t in Taille::tailles() {
            if total > 0.0 {
                if cap > 0.0 {
                    let f: f32 = *cc.comptes.get(t).unwrap() as f32 / total;
                    *res.get_mut(t).unwrap() += (cap*f).ceil() as usize;
                } else {
                    *res.get_mut(t).unwrap() += *cc.comptes.get(t).unwrap();
                }
            }
        }
    }

    res
}

pub fn calcul_chandail_complex(groupes: &GroupeReg, membres: &MembreReg) -> HashMap<Taille, usize> {
    let mut chandails_cat: HashMap<String, ChandailCalcul> = HashMap::new();
    let mut tried = BTreeSet::new();
    for groupe in groupes.groupes() {
        let cat = groupe.category.as_ref().map_or("None".into(), String::clone);
        let cc = {
            if !chandails_cat.contains_key(&cat) {
                chandails_cat.insert(cat.clone(), ChandailCalcul::new());
            }
            chandails_cat.get_mut(&cat).unwrap()
        };
        update_calcul_chandail(cc, groupe, membres, &mut tried);
    }

    let mut res = HashMap::new();
    for t in Taille::tailles() {
        res.insert(*t, 0);
    }

    for (cat, cc) in chandails_cat {
        let total = cc.total() as f32;
        let cap: f32 = read_int(&format!("Nombre prévu de {}", cat)) as f32;
        for t in Taille::tailles() {
            if total > 0.0 {
                if cap > 0.0 {
                    let f: f32 = *cc.comptes.get(t).unwrap() as f32 / total;
                    *res.get_mut(t).unwrap() += (cap*f).ceil() as usize;
                } else {
                    *res.get_mut(t).unwrap() += *cc.comptes.get(t).unwrap();
                }
            }
        }
    }

    res
}