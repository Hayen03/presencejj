//use extract::presence::{GroupeExtractConfig, GroupeExtractData};

use config::Config;
use extract::excel::fill_regs;
use groupes::{comptes::CompteReg, groupes::GroupeReg, membres::MembreReg};
use print::typst::print_fiche_med;

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
        let compte = compte_reg.get(membre.compte.unwrap()).unwrap();
        print_fiche_med(membre, compte, &config, "test", true).unwrap();
    }

}
