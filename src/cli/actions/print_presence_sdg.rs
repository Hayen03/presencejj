use std::collections::HashSet;

use crate::{cdj::groupes::NULL_GROUPE, cli::{ProgramData, actions::ActionResult}, print::typst::print_presence_sdj};


pub fn print_presences_sdj(program: &ProgramData, out_dir: Option<&str>) -> ActionResult {
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
        let _ = print_presence_sdj(gi, &program.groupes, &program.membres, &program.comptes, &program.config, out_dir);
    }
    Ok(())
}