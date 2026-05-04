pub mod actions;

use std::{io::Write, sync::RwLock};

use console::Term;

use crate::{cdj::{comptes::CompteReg, groupes::GroupeReg, membres::MembreReg}, config::Config};

pub struct ProgramData {
    pub out: Term,
    pub err: Term,
    pub config: Config,
    pub groupes: GroupeReg,
    pub comptes: CompteReg,
    pub membres: MembreReg,
    old_out_dir: RwLock<String>,
}
impl ProgramData {
    pub fn new(out: Term, err: Term, config: Config, groupes: GroupeReg, comptes: CompteReg, membres: MembreReg) -> Self {
        ProgramData {
            out,
            err,
            config,
            groupes,
            comptes,
            membres,
            old_out_dir: RwLock::new("/".into()),
        }
    }
    pub fn get_out_dir(&self, title: &str) -> Option<String> {
        let mut old_dir = self.old_out_dir.write().unwrap();
        let new_dir = rfd::FileDialog::new()
            .set_title(title)
            .set_directory(old_dir.as_str())
            .pick_folder();
        new_dir.as_ref()?;
        let new_dir = new_dir.unwrap();
        let path = new_dir.to_str().unwrap().to_string();
        let dir = new_dir.parent().map(|p| p.to_str().unwrap().to_string()).unwrap_or("/".into());
        //println!("{}", dir);
        *old_dir = dir;
        Some(path)
    }
    pub fn get_out_xlsx(&self, title: &str) -> Option<String> {
        let old_dir = self.old_out_dir.read().unwrap();
        let file = rfd::FileDialog::new()
            .set_title(title)
            .set_directory(old_dir.as_str())
            .add_filter("xlsx", &["xlsx"])
            .pick_file();
        file.map(|p| {
            p.to_str().unwrap().to_string()
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ProgramActions {
    #[default]
    Quitter,
    ChargerDeProg,
    ChargerDePresence,
    ImprimerListesPresence,
    ImprimerFichesSante,
    EstimerChandails,
    FaireSousGroupes,
    ImprimerStats,
    AfficherDonnees,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum AfficherActions {
    Groupes,
    Membres,
    Comptes,
    #[default]
    Annuler,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum EstimationChandailMode {
    Simple,
    Complex,
    #[default]
    Annuler,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum YesNo {
    Yes,
    #[default]
    No,
}
impl YesNo {
    pub fn as_bool(&self) -> bool {
        match self {
            YesNo::Yes => true,
            YesNo::No => false,
        }
    }
}


pub fn choose_option<T: Default + Copy + Clone>(term: &Term, options: &[(&str, T)]) -> T {
    for (i, (txt, _)) in options.iter().enumerate() {
        let _ = term.write_line(&format!("[{}] {}", i+1, *txt));
    }
    while {
        let input: String = dialoguer::Input::new()
            .with_prompt("Entrez votre choix: ")
            .interact_text()
            .expect("N'a pu lire l'entrée");
        match input.parse::<usize>() {
            Ok(n) => {
                if n > 0 && n <= options.len() {
                    return options[n-1].1;
                } else {
                    let _ = term.write_line("Entrée invalide.");
                }
            },
            Err(_) => {
                let _ = term.write_line("Entrée invalide.");
            },
        }
        true
    } {}
    T::default()
}

pub fn wait_to_continue() -> bool {
    print!("Appuyez sur entrée pour continuer");
    let _ = std::io::stdout().flush();
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
    true
}

pub fn read_file_path(msg: &str) -> String {
    let mut filepath = String::new();
    while {
        let rep: Result<String, _> = dialoguer::Input::new().with_prompt(msg).interact_text();
        match rep {
            Ok(f) => {
                filepath = f.trim().into();
                false
            },
            Err(_e) => {
                true
            }
        }
    } {}
    // nettoyer le nom du fichier
    if filepath.starts_with("\"") && filepath.ends_with("\"") || filepath.starts_with("'") && filepath.ends_with("'") {
        filepath = filepath.as_str()[1..filepath.len()-1].into();
    }
    println!("Tentative: {}", filepath);
    filepath
}

pub fn print_banner(term: &Term) {
    let _ = term.write_line("============================================");
}