use console::style;

use crate::{cdj::groupes::{Groupe, NULL_GROUPE}, cli::{ProgramData, actions::ActionResult}, prelude::read_int_option};


pub fn build_sous_groupes(program: &mut ProgramData) -> ActionResult {
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
            let s1 = format!("Combien de sous groupe pour le groupe [{}]? ", grp.short_desc());
            let s2 = if let Some(cap) = &grp.capacite {
                format!("(capacité de {cap}): ")
            } else {
                String::new()
            };
            let msg = s1 + &s2;
            read_int_option(&msg).map(|n| n as usize)
        },
    }
}