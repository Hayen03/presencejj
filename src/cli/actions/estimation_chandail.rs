use std::collections::HashMap;

use crate::{cli::{EstimationChandailMode, ProgramData, actions::ActionResult, choose_option}, prelude::read_int};


pub fn estimation_chandail(program: &ProgramData) -> ActionResult {

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
        EstimationChandailMode::Complex => {
            let mut pred = HashMap::new();
            for cat in program.groupes.list_used_category() {
                let cap: usize = read_int(&format!("Nombre prévu de {}", cat)) as usize;
                pred.insert(cat, cap);
            }
            crate::stats::calcul_chandail_complex(&program.groupes, &program.membres, &pred)
        },
    };

    let mut total = 0;
    for (taille, nb) in estimation {
        let _ = program.out.write_line(&format!("{}: {}", taille, nb));
        total += nb;
    }
    let _ = program.out.write_line(&format!("Total: {}", total));

    Ok(())
}