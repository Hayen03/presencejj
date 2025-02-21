//use extract::presence::{GroupeExtractConfig, GroupeExtractData};

use std::collections::HashSet;

use config::Config;
use extract::excel::fill_regs;
use groupes::{comptes::{CompteReg, NULL_COMPTE}, groupes::{GroupeReg, NULL_GROUPE}, membres::{MembreReg, NULL_MEMBRE}};
use print::typst::{print_fiche_med, print_presence_anim, print_presence_sdj};

pub mod data;
pub mod extract;
pub mod groupes;
pub mod prelude;
pub mod print;
pub mod ui;
pub mod config;


fn main() {
    let out_term = console::Term::stdout();
    let err_term = console::Term::buffered_stderr();
    let config = Config::default();

    let mut groupe_reg = GroupeReg::default();
    let mut compte_reg = CompteReg::default();
    let mut membre_reg = MembreReg::default();

    let _ = groupe_reg.add(NULL_GROUPE.clone());
    let _ = compte_reg.add(NULL_COMPTE.clone());
    let _ = membre_reg.add(NULL_MEMBRE.clone());

    let filepath: String = dialoguer::Input::new()
        .with_prompt("Fichier Excel")
        .interact_text()
        .expect("N'a pu prendre l'entrée");
    // nettoyer le nom du fichier
    let mut f = filepath.trim();
    if f.starts_with("\"") && f.ends_with("\"") || f.starts_with("'") && f.ends_with("'") {
        f = &f[1..f.len()-1]
    }

    let _ = fill_regs(&mut compte_reg, &mut membre_reg, &mut groupe_reg, &config, &f, &out_term, &err_term);

    for membre in membre_reg.membres() {
        let compte = compte_reg.get(membre.compte.unwrap_or_default()).unwrap_or(&NULL_COMPTE);
        print_fiche_med(membre, compte, &config, "test", true).unwrap();
    }

    for grp in groupe_reg.groupes() {
        let _ = print_presence_anim(grp, None, &membre_reg, &compte_reg, &config);
    }

    // Trouver toutes les combinaisons de (saison, site, semaine)
    let mut grp_info = HashSet::new();
    for grp in groupe_reg.groupes() {
        if *grp == *NULL_GROUPE {
            continue
        }
        let gi = grp.get_sdj_info();
        grp_info.insert(gi);
    }
    for gi in grp_info.iter() {
        let _ = print_presence_sdj(gi, &groupe_reg, &membre_reg, &compte_reg, &config);
    }

}
