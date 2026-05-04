mod charger_liste_presence;
mod print_fiche_sante;
mod print_liste_presence;
mod print_presence_sdg;
mod charger_prog;
mod afficher_donnees;
mod build_sous_groupes;
mod estimation_chandail;
mod print_stats;

pub use charger_liste_presence::charger_from_list_presence;
pub use print_fiche_sante::print_fiche_santes;
pub use print_liste_presence::print_presences_anim;
pub use print_presence_sdg::print_presences_sdj;
pub use charger_prog::charger_from_prog;
pub use afficher_donnees::afficher_donnees;
pub use build_sous_groupes::build_sous_groupes;
pub use estimation_chandail::estimation_chandail;
pub use print_stats::print_stats;

#[derive(Debug)]
pub struct ActionError {
	pub src: Box<dyn std::error::Error + 'static>,
}
impl<T> From<Box<T>> for ActionError where T: std::error::Error + 'static {
	fn from(value: Box<T>) -> Self {
		ActionError {
			src: value,
		}
	}
}
impl std::fmt::Display for ActionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Action error: {}", self.src)
	}
}
impl std::error::Error for ActionError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		Some(self.src.as_ref())
	}
}

pub type ActionResult = Result<(), ActionError>;