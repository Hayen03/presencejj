//use extract::presence::{GroupeExtractConfig, GroupeExtractData};

use std::{collections::HashSet, io::Write};

use config::Config;
use console::Term;
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

struct ProgramData {
    pub out: Term,
    pub err: Term,
    pub config: Config,
    pub groupes: GroupeReg,
    pub comptes: CompteReg,
    pub membres: MembreReg,
}

#[derive(Debug, Default, Clone, Copy)]
enum ProgramAction {
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

    let mut program = ProgramData {
        out: out_term,
        err: err_term,
        config,
        groupes: groupe_reg,
        comptes: compte_reg,
        membres: membre_reg,
    };

    while {
        let _ = program.out.clear_screen();
        let action = choose_option(&program.out, &[
            ("Lire à partir de la programmation.", ProgramAction::ChargerDeProg),
            ("Lire à partir des listes de présences.", ProgramAction::ChargerDePresence),
            ("Faire les sous-groupes.", ProgramAction::FaireSousGroupes),
            ("Faire les fiches médicales.", ProgramAction::ImprimerFichesSante),
            ("Faire les listes de présences.", ProgramAction::ImprimerListesPresence),
            ("Estimer la quantité de chandails.", ProgramAction::EstimerChandails),
            ("Faire les statistiques de camp.", ProgramAction::ImprimerStats),
            ("Afficher les données.", ProgramAction::AfficherDonnees),
            ("Quitter", ProgramAction::Quitter),
        ]);
        let _ = program.out.clear_screen();
        match action {
            ProgramAction::Quitter => {
                false
            },
            ProgramAction::ChargerDeProg => {
                let _res = charger_from_prog(&mut program);
                wait_to_continue()
            },
            ProgramAction::ChargerDePresence => {
                let _res = charger_from_list_presence(&mut program);
                wait_to_continue()
            },
            ProgramAction::ImprimerListesPresence => {
                let _res = print_presences_anim(&program);
                let _res = print_presences_sdj(&program);
                wait_to_continue()
            },
            ProgramAction::ImprimerFichesSante => {
                let _res = print_fiche_santes(&program);
                wait_to_continue()
            },
            ProgramAction::EstimerChandails => {
                let _res = program.out.write_line("Estimation de chandail... (Pas encore implémenté)");
                wait_to_continue()
            },
            ProgramAction::FaireSousGroupes => {
                let _res = program.out.write_line("Création des sous-groupes... (Pas encore implémenté)");
                wait_to_continue()
            },
            ProgramAction::ImprimerStats => {
                let _res = program.out.write_line("Calcul des statistiques... (Pas encore implémenté)");
                wait_to_continue()
            },
            ProgramAction::AfficherDonnees => {
                let _res = program.out.write_line("Affichage des données... (Pas encore implémenté)");
                wait_to_continue()
            }
        }
    } {}

    //let _res = charger_from_list_presence(&mut program);

    //let _res = print_fiche_santes(&program);

    //let _res = print_presences_anim(&program);
    //let _res = print_presences_sdj(&program);

}

fn charger_from_list_presence(program: &mut ProgramData) -> Result<(), ()> {
    let filepath: String = dialoguer::Input::new()
        .with_prompt("Fichier Excel")
        .interact_text()
        .expect("N'a pu prendre l'entrée");
    // nettoyer le nom du fichier
    let mut f = filepath.trim();
    if f.starts_with("\"") && f.ends_with("\"") || f.starts_with("'") && f.ends_with("'") {
        f = &f[1..f.len()-1]
    }

    let _ = fill_regs(&mut program.comptes, &mut program.membres, &mut program.groupes, &program.config, &f, &program.out, &program.err);
    Ok(())
}

fn print_fiche_santes(program: &ProgramData) -> Result<(), ()> {
    for membre in program.membres.membres() {
        let compte = program.comptes.get(membre.compte.unwrap_or_default()).unwrap_or(&NULL_COMPTE);
        print_fiche_med(membre, compte, &program.config, "test", true).unwrap();
    }
    Ok(())
}

fn print_presences_anim(program: &ProgramData) -> Result<(), ()> {
    for grp in program.groupes.groupes() {
        let _ = print_presence_anim(grp, None, &program.membres, &program.comptes, &program.config);
    }
    Ok(())
}

fn print_presences_sdj(program: &ProgramData) -> Result<(), ()> {
    // Trouver toutes les combinaisons de (saison, site, semaine)
    let mut grp_info = HashSet::new();
    for grp in program.groupes.groupes() {
        if *grp == *NULL_GROUPE {
            continue
        }
        let gi = grp.get_sdj_info();
        grp_info.insert(gi);
    }
    for gi in grp_info.iter() {
        let _ = print_presence_sdj(gi, &program.groupes, &program.membres, &program.comptes, &program.config);
    }
    Ok(())
}

fn charger_from_prog(program: &mut ProgramData) -> Result<(), ()> {
    Ok(())
}

fn choose_option<T: Default + Copy + Clone>(term: &Term, options: &[(&str, T)]) -> T {
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

fn wait_to_continue() -> bool {
    print!("Appuyez sur entrée pour continuer");
    let _ = std::io::stdout().flush();
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
    true
}