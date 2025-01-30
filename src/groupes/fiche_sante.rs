use crate::{data::{cam::CAM, BoolJustifie}, prelude::*};

#[derive(Debug, Clone)]
pub struct FicheSante {
    allergies: Vec<String>,
    maladies: Vec<String>,
    auth_soins: O<bool>,
    probleme_comportement: O<BoolJustifie>,
    cam: O<CAM>,
    prise_med: O<BoolJustifie>,
    auth_medicaments: Medicaments,
}

#[derive(Debug, Clone, Copy)]
pub struct Medicaments {
    pub sirop_toux: O<bool>,
    pub anti_emetique: O<bool>,
    pub ibuprofene: O<bool>,
    pub anti_inflamatoire: O<bool>,
    pub anti_biotique: O<bool>,
    pub acetaminophene: O<bool>,
}