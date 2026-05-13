#![feature(iter_intersperse)]
//use extract::presence::{GroupeExtractConfig, GroupeExtractData};
use config::Config;

use crate::{cdj::{RegError, comptes::{CompteErr, CompteID, CompteReg, NULL_COMPTE}, groupes::{GroupeID, GroupeReg, NULL_GROUPE}, membres::{MembreID, MembreReg, NULL_MEMBRE}}, cli::{ProgramActions, ProgramData, YesNo, choose_option, print_banner, wait_to_continue}, ui::{TextInputError, UIError, app::App, event::EventError, tui::{Tui, TuiError}}};

pub mod data;
pub mod extract;
pub mod cdj;
pub mod prelude;
pub mod print;
pub mod ui;
pub mod config;
pub mod stats;
pub mod cli;

#[allow(dead_code)]
#[derive(Debug)]
enum AppError {
    IOError{src: std::io::Error},
    EventError {src: EventError },
    ExcelError,
    Report{ report: color_eyre::Report},
    Panic,
    Runtime { src: Box<dyn std::any::Any> },
    Extract { src: extract::ExtractError },
    GroupeRegistry { src: RegError<GroupeID> },
    CompteRegistry { src: RegError<CompteID> },
    MembreRegistry { src: RegError<MembreID> },
    CancelAction { desc: String },
    Compte { src: CompteErr },
    Input { src: TextInputError },
    UnexpectedState { desc: String },
    Other { src: Box<dyn std::error::Error + Send + Sync> },
}
impl From<UIError> for AppError {
    fn from(_src: UIError) -> Self {
        match _src {
            UIError::IO { src } => AppError::IOError { src },
            UIError::Event { src } => AppError::EventError { src },
            UIError::Runtime { src } => AppError::Runtime { src },
            UIError::Extract { src } => AppError::Extract { src },
            UIError::GroupeRegistry { src } => AppError::GroupeRegistry { src },
            UIError::CompteRegistry { src } => AppError::CompteRegistry { src },
            UIError::MembreRegistry { src } => AppError::MembreRegistry { src },
            UIError::CancelAction { desc } => AppError::CancelAction { desc },
            UIError::Compte { src } => AppError::Compte { src },
            UIError::Input { src } => AppError::Input { src },
            UIError::UnexpectedState { desc } => AppError::UnexpectedState { desc },
            UIError::Others { src } => AppError::Other { src },
        }
    }
}
impl From<color_eyre::Report> for AppError {
    fn from(report: color_eyre::Report) -> Self {
        AppError::Report { report }
    }
}
impl From<TuiError> for AppError {
    fn from(value: TuiError) -> Self {
        match value {
            TuiError::IOError { src } => AppError::IOError { src },
            TuiError::Panic { info } => AppError::Runtime { src: info },
        }
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::IOError { src } => write!(f, "IO error: {}", src),
            AppError::EventError { src } => write!(f, "Event error: {}", src),
            AppError::ExcelError => write!(f, "Excel error"),
            AppError::Report { report } => write!(f, "{}", report),
            AppError::Panic => write!(f, "Panic error"),
            AppError::Runtime { .. } => write!(f, "Runtime error"),
            AppError::Extract { src } => write!(f, "Extract error: {}", src),
            AppError::GroupeRegistry { src } => write!(f, "Groupe registry error: {}", src),
            AppError::CompteRegistry { src } => write!(f, "Compte registry error: {}", src),
            AppError::MembreRegistry { src } => write!(f, "Membre registry error: {}", src),
            AppError::CancelAction { desc } => write!(f, "Action cancelled: {}", desc),
            AppError::Compte { src } => write!(f, "Compte error: {}", src),
            AppError::Input { src } => write!(f, "Input error: {}", src),
            AppError::UnexpectedState { desc } => write!(f, "Unexpected state: {}", desc),
            AppError::Other { src } => write!(f, "{}", src),
        }
    }
}
impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::IOError { src } => Some(src),
            AppError::EventError { src } => Some(src),
            AppError::Runtime { src } => None,
            AppError::Extract { src } => Some(src),
            AppError::GroupeRegistry { src } => Some(src),
            AppError::CompteRegistry { src } => Some(src),
            AppError::MembreRegistry { src } => Some(src),
            AppError::CancelAction { .. } => None,
            AppError::Compte { src } => Some(src),
            AppError::Input { src } => Some(src),
            AppError::Report { report } => None,
            AppError::Panic => None,
            AppError::ExcelError => None,
            AppError::UnexpectedState { .. } => None,
            AppError::Other { src } => Some(src.as_ref()),
        }
    }
}

fn main() {
    let result = main_new();
    println!("{:?}", result);
}

fn main_new() -> Result<(), AppError> {
    color_eyre::install()?;
    let mut terminal = Tui::enter()?;
    let result = App::default().run(&mut terminal);
    if let Err(err) = terminal.exit() {
        eprintln!("Erreur lors de la restauration du terminal. Rouvrir le terminal pour récupérer: {}", err);
        return Err(AppError::Panic);
    }
    if let Err(err) = result {
        eprintln!("Erreur lors de l'exécution de l'application: {}", err);
        return Err(AppError::from(err));
    }
    Ok(())
}

#[allow(dead_code)]
fn main_old() -> Result<(), AppError> {
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
                let _res = cli::actions::charger_from_prog(&mut program);
                wait_to_continue()
            },
            ProgramActions::ChargerDePresence => {
                let _res = cli::actions::charger_from_list_presence(&mut program);
                wait_to_continue()
            },
            ProgramActions::ImprimerListesPresence => {
                // Obtenir le dossier de sortie
                let out_dir = program.get_out_dir("Sélectionnez le dossier de sortie");
                if out_dir.is_none() {
                    let _ = program.err.write_line("Aucun dossier de sortie sélectionné.");
                    true
                } else {
                    let _res = cli::actions::print_presences_anim(&program, out_dir.as_deref());
                    let _res = cli::actions::print_presences_sdj(&program, out_dir.as_deref());
                    wait_to_continue()
                }
            },
            ProgramActions::ImprimerFichesSante => {
                let _res = cli::actions::print_fiche_santes(&program);
                wait_to_continue()
            },
            ProgramActions::EstimerChandails => {
                let _res = cli::actions::estimation_chandail(&program);
                wait_to_continue()
            },
            ProgramActions::FaireSousGroupes => {
                let _res = cli::actions::build_sous_groupes(&mut program);
                wait_to_continue()
            },
            ProgramActions::ImprimerStats => {
                let _ = program.out.write_line("Souhaitez-vous calculer les annulations (vous devez les entrer manuellement)?");
                let do_annulation = choose_option(&program.out, &[
                    ("Oui", YesNo::Yes),
                    ("Non", YesNo::No),
                ]).as_bool();
                let _ = program.out.write_line("Souhaitez-vous calculer les liste d'attente (vous devez les entrer manuellement)?");
                let do_attente = choose_option(&program.out, &[
                    ("Oui", YesNo::Yes),
                    ("Non", YesNo::No),
                ]).as_bool();
                let out_file = program.get_out_xlsx("Sélectionnez le dossier de sortie");
                if let Some(out_file) = out_file {
                    let _res = cli::actions::print_stats(&program, out_file, do_annulation, do_attente);
                } else {
                    let _ = program.err.write_line("Aucun fichier de sortie sélectionné.");
                }
                wait_to_continue()
            },
            ProgramActions::AfficherDonnees => {
                let _res = cli::actions::afficher_donnees(&program);
                true
            }
        }
    } {}

    //let _res = charger_from_list_presence(&mut program);

    //let _res = print_fiche_santes(&program);

    //let _res = print_presences_anim(&program);
    //let _res = print_presences_sdj(&program);

    Ok(())

}
