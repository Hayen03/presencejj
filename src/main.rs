//use extract::presence::{GroupeExtractConfig, GroupeExtractData};

use extract::excel::fill_regs;
use groupes::{comptes::CompteReg, groupes::GroupeReg, membres::MembreReg};
use lazy_static::lazy_static;
use prelude::excel_col_to_num;
use regex::Regex;

pub mod data;
pub mod extract;
pub mod groupes;
pub mod prelude;
pub mod print;
pub mod ui;

pub struct Config {
    out_dir: String, 
    verbose: bool,
    excel: ExcelConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            out_dir: "./out".into(),
            verbose: true,
            excel: ExcelConfig::default(),
        }
    }
}
pub struct ExcelConfig {
    ln_skip: usize,
    data_ln: usize,
}
impl Default for ExcelConfig {
    fn default() -> Self {
        Self {
            ln_skip: 6,
            data_ln: 5,
        }
    }
}

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

    fill_regs(&mut compte_reg, &mut membre_reg, &mut groupe_reg, &config, &f, &out_term, &err_term);

}
