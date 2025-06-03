//use extract::presence::{GroupeExtractConfig, GroupeExtractData};

use std::{collections::{HashMap, HashSet}, io::Write, sync::RwLock};

use config::Config;
use console::{style, Term};
use extract::excel::fill_regs;
use groupes::{comptes::{CompteReg, NULL_COMPTE}, groupes::{Groupe, GroupeReg, NULL_GROUPE}, membres::{MembreID, MembreReg, NULL_MEMBRE}};
use office::Excel;
use prelude::read_int_option;
use print::typst::{print_fiche_med, print_presence_anim, print_presence_sdj};

use crate::groupes::membres;

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
    let mut config = Config{
        working_dir: std::env::current_dir().unwrap().to_str().unwrap().into(),
        ..Config::default()
    };
    
    // get typst working dir from args
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        config.typst_working_dir = args[1].clone();
    }

    let mut groupe_reg = GroupeReg::default();
    let mut compte_reg = CompteReg::default();
    let mut membre_reg = MembreReg::default();

    let _ = groupe_reg.add(NULL_GROUPE.clone());
    let _ = compte_reg.add(NULL_COMPTE.clone());
    let _ = membre_reg.add(NULL_MEMBRE.clone());

    let mut program = ProgramData::new(out_term, err_term, config, groupe_reg, compte_reg, membre_reg);

    while {
        let _ = program.out.clear_screen();
        //println!("{:?}", std::env::current_dir());
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
                // Obtenir le dossier de sortie
                let out_dir = program.get_out_dir("Sélectionnez le dossier de sortie");
                if out_dir.is_none() {
                    let _ = program.err.write_line("Aucun dossier de sortie sélectionné.");
                    true
                } else {
                    let _res = print_presences_anim(&program, out_dir.as_deref());
                    let _res = print_presences_sdj(&program, out_dir.as_deref());
                    wait_to_continue()
                }
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
                let _res = build_sous_groupes(&mut program);
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
    let filepath = rfd::FileDialog::new()
        .set_title("Sélectionner le fichier de présence")
        .add_filter("excel", &["xlsx"])
        .set_directory("/")
        .pick_file();
    if filepath.is_none() {
        let _ = program.err.write_line("Aucun fichier sélectionné.");
        return Err(());
    }
    let filepath = filepath.unwrap().to_str().unwrap().to_string();
    //let filepath: String = read_file_path("Fichier xlsx: ");

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

    // Obtenir le dossier de sortie
    let out_dir = program.get_out_dir("Sélectionnez le dossier de sortie");
    if out_dir.is_none() {
        let _ = program.err.write_line("Aucun dossier de sortie sélectionné.");
        return Err(());
    }

    // identifie quel enfant est sur quel site
    let mut site_mbrs: HashMap<&str, HashSet<MembreID>> = HashMap::new();
    for grp in program.groupes.groupes() {
        let set = {
            let site = grp.site.as_deref().unwrap_or("None");
            if !site_mbrs.contains_key(site) {
                site_mbrs.insert(site, HashSet::new());
            }
            site_mbrs.get_mut(site).unwrap()
        };
        for part in grp.participants.iter() {
            set.insert(*part);
        }
    }

    // imprime les fiches med par site
    for (site, parts) in site_mbrs {
        for mid in parts {
            if let Ok(membre) = program.membres.get(mid) {
                let compte = program.comptes.get(membre.compte.unwrap_or_default()).unwrap_or(&NULL_COMPTE);

                let _res = print_fiche_med(membre, compte, &program.config, site, false, out_dir.as_deref());
                match _res {
                    Ok(_) => {
                        let _ = program.out.write_line(&format!("{}", style(format!("Impression de la fiche santé de [{} {}]", &membre.prenom, &membre.nom)).cyan()));
                    },
                    Err(_e) => {
                        let _ = program.err.write_line(&format!("{}", style(format!("Échec lors de l'impression de la fiche santé de [{} {}]", &membre.prenom, &membre.nom)).red()));
                    },
                }
            } else {
                let _ = program.err.write_line(&format!("{}", style(format!("Membre {mid} inexistant")).red()));
            }
        }
    }
    Ok(())
}

fn print_presences_anim(program: &ProgramData, out_dir: Option<&str>) -> Result<(), ()> {
    let mut compte = 0;
    for grp in program.groupes.groupes() {
        compte += 1;
        if grp == &(*NULL_GROUPE) {continue;}
        if grp.sous_groupe.is_empty() {
            print_presence_anim(grp, None, &program.membres, &program.comptes, &program.config, out_dir.as_deref()).expect("Oups");
        } else {
            for sg in &grp.sous_groupe {
                print_presence_anim(grp, Some(sg), &program.membres, &program.comptes, &program.config, out_dir.as_deref()).expect("AAAAAAh");
            }
        }
        
    }
    let _ = program.out.write_line(&format!("À imprimé {}/{} groupes", compte, program.groupes.len()));
    Ok(())
}

fn print_presences_sdj(program: &ProgramData, out_dir: Option<&str>) -> Result<(), ()> {
    // Trouver toutes les combinaisons de (saison, site, semaine)
    let mut grp_info = HashSet::new();
    for grp in program.groupes.groupes() {
        if grp == &(*NULL_GROUPE) {
            continue
        }
        let gi = grp.get_sdj_info();
        grp_info.insert(gi);
    }
    for gi in grp_info.iter() {
        let _ = print_presence_sdj(gi, &program.groupes, &program.membres, &program.comptes, &program.config, out_dir.as_deref());
    }
    Ok(())
}

fn charger_from_prog(program: &mut ProgramData) -> Result<(), ()> {
    let filepath = rfd::FileDialog::new()
        .set_title("Sélectionner le fichier de programmation")
        .add_filter("excel", &["xlsx"])
        .set_directory("/")
        .pick_file();
    if filepath.is_none() {
        let _ = program.err.write_line("Aucun fichier sélectionné.");
        return Err(());
    }
    let filepath = filepath.unwrap().to_str().unwrap().to_string();

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
                let mut compte = 0;
                for groupe in program.groupes.groupes() {
                    compte += 1;
                    if groupe == &(*NULL_GROUPE) {continue;}
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
                println!("{}", compte);
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

fn build_sous_groupes(program: &mut ProgramData) -> Result<(), ()> {
    for grp in program.groupes.groupes_mut() {
        if *grp == *NULL_GROUPE { continue; } // skip le groupe null
        let nb_sg = guess_nb_sous_groupes(grp);
        if let Some(nb_sg) = nb_sg {
            match grp.mk_sous_groupes(nb_sg, &program.membres) {
                Ok(_) => {
                    let _ = program.out.write_line(&format!("{}", style(format!("Création de {nb_sg} sous-groupes pour [{}]", grp.short_desc())).cyan()));
                },
                Err(_) => {
                    let _ = program.err.write_line(&format!("{}", style(format!("Échec lors de la création de {nb_sg} sous-groupes pour [{}]", grp.short_desc())).red()));
                },
            }
        }
    }
    Ok(())
}

fn guess_nb_sous_groupes(grp: &Groupe) -> Option<usize> {
    let cat = grp.category.as_ref().map(|s| s.to_lowercase());
    match (cat.as_deref(), grp.estime_cap()) {
        (_, 0) => None,
        (Some("crocus"), i) => { // crocus -> 10 par groupes
            Some((i as f32/10.0).ceil() as usize)
        },
        (Some("balaous"), i) => { // balaous -> 12 par groupes
            Some((i as f32/12.0).ceil() as usize)
        },
        (Some("basaltes"), i) => { // basaltes -> 15 par groupes
            Some((i as f32/15.0).ceil() as usize)
        },
        (Some("12-15 ans"), i) => {
            Some((i as f32/15.0).ceil() as usize)
        },
        (_c, _) => { // inconnu, on doit demander
            //println!("Cat de groupe inconnu: {:?}", c);
            let mut s1 = format!("Combien de sous groupe pour le groupe [{}]? ", grp.short_desc());
            let mut s2 = if let Some(cap) = &grp.capacite {
                format!("(capacité de {cap}): ")
            } else {
                String::new()
            };
            let msg = (s1 + &s2);
            read_int_option(&msg).map(|n| n as usize)
        },
    }
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
    filepath
}

fn print_banner(term: &Term) {
    let _ = term.write_line("============================================");
}