use crate::{cdj::groupes::NULL_GROUPE, cli::{AfficherActions, ProgramData, actions::ActionResult, choose_option, wait_to_continue}};


pub fn afficher_donnees(program: &ProgramData) -> ActionResult {

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