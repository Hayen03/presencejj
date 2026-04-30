use std::sync::Arc;

use crate::ui::{AppState, Screen, UIError, UpdateAction};

mod quit;
mod save;
mod afficher_donnees;
mod charger_de_fichier;
mod charger_de_presence;
mod charger_de_prog;
mod estimer_chandail;
mod faire_sous_groupes;
mod imprimer_fiche_sante;
mod imprimer_liste_presence;
mod imprimer_stats;

pub use quit::quit;
pub use save::sauvegarder;
pub use afficher_donnees::afficher_donnees;
pub use charger_de_fichier::charger_de_fichier;
pub use charger_de_presence::charger_de_presence;
pub use charger_de_prog::charger_de_prog;
pub use estimer_chandail::estimer_chandail;
pub use faire_sous_groupes::faire_sous_groupes;
pub use imprimer_fiche_sante::imprimer_fiche_sante;
pub use imprimer_liste_presence::imprimer_liste_presence;
pub use imprimer_stats::imprimer_stats;

pub type ActionResult = Result<UpdateAction, UIError>;
pub type Action = dyn Fn(Arc<AppState>) -> ActionResult;

#[derive(Debug, Default, Clone, Copy)]
pub enum MainActions {
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
	ChargerDeFichier,
	Sauvegarder,
}
impl MainActions {
	pub fn as_str(&self) -> &'static str {
		match self {
			MainActions::Quitter => "Quitter",
			MainActions::ChargerDeProg => "Charger de prog",
			MainActions::ChargerDePresence => "Charger de présence",
			MainActions::ImprimerListesPresence => "Imprimer listes de présence",
			MainActions::ImprimerFichesSante => "Imprimer fiches santé",
			MainActions::EstimerChandails => "Estimer chandails",
			MainActions::FaireSousGroupes => "Faire sous-groupes",
			MainActions::ImprimerStats => "Imprimer stats",
			MainActions::AfficherDonnees => "Afficher données",
			MainActions::ChargerDeFichier => "Charger de fichier",
			MainActions::Sauvegarder => "Sauvegarder",
		}
	}
}
impl std::fmt::Display for MainActions {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}