use serde::{Deserialize, Serialize};

use crate::{data::{cam::CAM, BoolJustifie}, prelude::*};

pub static MAL_ASTHME: &str = "Asthme";
pub static MAL_DIABETE: &str = "Diabète";
pub static MAL_EMOPHILIE: &str = "Émophilie";
pub static MAL_EPILEPSIE: &str = "Épilepsie";

pub static ALL_ALIMENTAIRE: &str = "Alimentaire";
pub static ALL_ANIMAUX: &str = "Animaux";
pub static ALL_INSECTES: &str = "Insectes";
pub static ALL_PENICILINE: &str = "Péniciline";

#[derive(Debug, Clone, Default, Hash, Serialize, Deserialize)]
pub struct FicheSante {
    pub allergies: Vec<String>,
    pub maladies: Vec<String>,
    pub auth_soins: O<bool>,
    pub probleme_comportement: O<BoolJustifie>,
    pub cam: O<CAM>,
    pub prise_med: O<BoolJustifie>,
    pub auth_medicaments: Medicaments,
}

#[derive(Debug, Clone, Copy, Default, Hash, Serialize, Deserialize)]
pub struct Medicaments {
    pub sirop_toux: O<bool>,
    pub anti_emetique: O<bool>,
    pub ibuprofene: O<bool>,
    pub anti_inflamatoire: O<bool>,
    pub anti_biotique: O<bool>,
    pub acetaminophene: O<bool>,
}
impl Medicaments {
    pub fn ls(&self) -> String {
        vec![
            self.acetaminophene.and_then(|p| if p {Some("Acétaminophène".to_string())} else {None}),
            self.anti_biotique.and_then(|p| if p {Some("Anti-biotique".to_string())} else {None}),
            self.anti_emetique.and_then(|p| if p {Some("Anti-émétique".to_string())} else {None}),
            self.anti_inflamatoire.and_then(|p| if p {Some("Anti-inflammatoire".to_string())} else {None}),
            self.ibuprofene.and_then(|p| if p {Some("Ibuprofène".to_string())} else {None}),
            self.sirop_toux.and_then(|p| if p {Some("Sirop pour la toux".to_string())} else {None}),
        ].into_iter().flatten().collect::<Vec<_>>().join(", ")
    }
}