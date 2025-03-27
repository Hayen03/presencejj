//use extract::presence::{GroupeExtractConfig, GroupeExtractData};

use std::{collections::HashSet, io::Write};

use config::Config;
use console::{style, Term};
use extract::excel::fill_regs;
use groupes::{comptes::{CompteReg, NULL_COMPTE}, groupes::{GroupeReg, NULL_GROUPE}, membres::{MembreReg, NULL_MEMBRE}};
use office::Excel;
use print::typst::{print_fiche_med, print_presence_anim, print_presence_sdj};

pub mod data;
pub mod extract;
pub mod groupes;
pub mod prelude;
pub mod print;
pub mod ui;
pub mod config;
pub mod stats;

struct ProgramData {
    pub out: Term,
    pub err: Term,
    pub config: Config,
    pub groupes: GroupeReg,
    pub comptes: CompteReg,
    pub membres: MembreReg,
}

#[derive(Debug, Default, Clone, Copy)]
enum ProgramActions {
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
enum AfficherActions {
    Groupes,
    Membres,
    Comptes,
    #[default]
    Annuler,
}

#[derive(Debug, Default, Clone, Copy)]
enum EstimationChandailMode {
    Simple,
    Complex,
    #[default]
    Annuler,
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
            ("Lire à partir de la programmation.", ProgramActions::ChargerDeProg),
            ("Lire à partir des listes de présences.", ProgramActions::ChargerDePresence),
            ("Faire les sous-groupes.", ProgramActions::FaireSousGroupes),
            ("Faire les fiches médicales.", ProgramActions::ImprimerFichesSante),
            ("Faire les listes de présences.", ProgramActions::ImprimerListesPresence),
            ("Estimer la quantité de chandails.", ProgramActions::EstimerChandails),
            ("Faire les statistiques de camp.", ProgramActions::ImprimerStats),
            ("Afficher les données.", ProgramActions::AfficherDonnees),
            ("Quitter", ProgramActions::Quitter),
        ]);
        let _ = program.out.clear_screen();
        print_banner(&program.out);
        match action {
            ProgramActions::Quitter => {
                false
            },
            ProgramActions::ChargerDeProg => {
                let _res = charger_from_prog(&mut program);
                wait_to_continue()
            },
            ProgramActions::ChargerDePresence => {
                let _res = charger_from_list_presence(&mut program);
                wait_to_continue()
            },
            ProgramActions::ImprimerListesPresence => {
                let _res = print_presences_anim(&program);
                let _res = print_presences_sdj(&program);
                wait_to_continue()
            },
            ProgramActions::ImprimerFichesSante => {
                let _res = print_fiche_santes(&program);
                wait_to_continue()
            },
            ProgramActions::EstimerChandails => {
                let _res = estimation_chandail(&program);
                wait_to_continue()
            },
            ProgramActions::FaireSousGroupes => {
                let _res = program.out.write_line("Création des sous-groupes... (Pas encore implémenté)");
                wait_to_continue()
            },
            ProgramActions::ImprimerStats => {
                let _res = program.out.write_line("Calcul des statistiques... (Pas encore implémenté)");
                wait_to_continue()
            },
            ProgramActions::AfficherDonnees => {
                let _res = afficher_donnees(&program);
                true
            }
        }
    } {}

    //let _res = charger_from_list_presence(&mut program);

    //let _res = print_fiche_santes(&program);

    //let _res = print_presences_anim(&program);
    //let _res = print_presences_sdj(&program);

}

fn charger_from_list_presence(program: &mut ProgramData) -> Result<(), ()> {
    let filepath: String = read_file_path("Fichier xlsx: ");

    let res = fill_regs(&mut program.comptes, &mut program.membres, &mut program.groupes, &program.config, &filepath, &program.out, &program.err);
    if let Err(e) = res {
        let _ = program.err.write_line(&format!("{}", e));
        return Err(())
    }
    let _ = program.out.flush();
    let _ = program.err.flush();
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
    let filepath = read_file_path("fichier xlsx: ");
    let mut wb = match Excel::open(&filepath) {
        Ok(wb) => wb,
        Err(e) => {
            let _ = program.err.write_line(&format!("{}", e));
            let _ = program.err.flush();
            return Err(());
        },
    };
    let _ = program.out.write_line(&format!("Lecture de \"{}\"", style(filepath).green()));

    let sheets = wb.sheet_names().unwrap();
    for sheet in sheets {
        let rng = wb.worksheet_range(&sheet).unwrap();
        crate::extract::prog::fill_groupe_reg_from_prog(&rng, &mut program.groupes, &program.out, &program.err);
    }
    let _ = program.out.flush();
    let _ = program.err.flush();
    Ok(())
}

fn afficher_donnees(program: &ProgramData) -> Result<(), ()> {

    while {
        let _ = program.out.clear_screen();
        let _ = program.out.write_line("Quel données voulez vous afficher?");
        let action = choose_option(&program.out, &[
            ("Groupes", AfficherActions::Groupes),
            ("Membres", AfficherActions::Membres),
            ("Comptes", AfficherActions::Comptes),
            ("Retour", AfficherActions::Annuler),
        ]);
        let _ = program.out.clear_screen();
        match action {
            AfficherActions::Groupes => {
                for groupe in program.groupes.groupes() {
                    let _ = program.out.write_line(&format!("{id}: {desc} --- inscriptions: {insc}/{cap}",
                        id=groupe.id,
                        desc=groupe.desc(),
                        insc=groupe.participants.len(),
                        cap=match groupe.capacite {
                            None => String::from("-"),
                            Some(c) => c.to_string(),
                        },
                    ));
                }
                wait_to_continue()
            },
            AfficherActions::Membres => {
                let _res = program.out.write_line("Affichage des membres... (Pas encore implémenté)");
                wait_to_continue()
            },
            AfficherActions::Comptes => {
                let _res = program.out.write_line("Affichage des comptes... (Pas encore implémenté)");
                wait_to_continue()
            },
            AfficherActions::Annuler => {
                false
            },
        }
    } {}

    Ok(())
}

fn estimation_chandail(program: &ProgramData) -> Result<(), ()> {

    /* DONNÉES 2024
        Crocus: 85 enfants
        Balaous: 111 enfants
        Basaltes: 102 enfants
     */

    let _ = program.out.clear_screen();
    let _ = program.out.write_line("Quel mode d'estimation voulez-vous utiliser?");
    let mode = choose_option(&program.out, &[
        ("Partiel (n'utilise que les inscriptions courrantes)", EstimationChandailMode::Simple),
        ("Complet (utilise les données des années précédentes)", EstimationChandailMode::Complex),
        ("Retour", EstimationChandailMode::Annuler),
    ]);

    let estimation = match mode {
        EstimationChandailMode::Annuler => {return Ok(());},
        EstimationChandailMode::Simple => crate::stats::calcul_chandail(&program.groupes, &program.membres),
        EstimationChandailMode::Complex => crate::stats::calcul_chandail_complex(&program.groupes, &program.membres),
    };

    let mut total = 0;
    for (taille, nb) in estimation {
        let _ = program.out.write_line(&format!("{}: {}", taille, nb));
        total += nb;
    }
    let _ = program.out.write_line(&format!("Total: {}", total));

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

fn read_file_path(msg: &str) -> String {
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
    return filepath;
}

fn print_banner(term: &Term) {
    let _ = term.write_line("============================================");
}