use std::collections::{HashMap, HashSet};

use console::style;

use crate::{cdj::{comptes::NULL_COMPTE, membres::MembreID}, cli::{ProgramData, actions::ActionResult}, prelude::ErrorMessage, print::typst::print_fiche_med};


pub fn print_fiche_santes(program: &ProgramData) -> ActionResult {

    // Obtenir le dossier de sortie
    let out_dir = program.get_out_dir("Sélectionnez le dossier de sortie");
    if out_dir.is_none() {
        let _ = program.err.write_line("Aucun dossier de sortie sélectionné.");
        return Err(Box::new(ErrorMessage::from("Aucun dossier de sortie sélectionné.")).into());
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