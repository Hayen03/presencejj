use std::{collections::{HashMap, HashSet}, path::PathBuf};


use crate::{cdj::{comptes::NULL_COMPTE, groupes::NULL_GROUPE, membres::MembreID}, cli::{ProgramData, actions::ActionResult}, prelude::ErrorMessage, print::typst::print_fiche_med, ui::screens::Desc};


pub fn print_fiche_santes(program: &ProgramData) -> ActionResult {

    // Obtenir le dossier de sortie
    let out_dir = program.get_out_dir("Sélectionnez le dossier de sortie").map(PathBuf::from);
    let out_dir = if let Some(p) = out_dir {
        p
    } else {
        let _ = program.err.write_line("Aucun dossier de sortie sélectionné.");
        return Err(Box::new(ErrorMessage::from("Aucun dossier de sortie sélectionné.")).into());
    };
    let logger = |msg: Desc| {
        let _ = program.out.write_line(msg.as_str());
    };

    // identifie quel enfant est sur quel site
    let mut site_mbrs: HashMap<MembreID, HashSet<(&str, &str)>> = HashMap::new();
    {
        let groupes = &program.groupes;
        for grp in groupes.groupes().filter(|g| g.id != NULL_GROUPE.id) {
            let site = grp.get_site().unwrap_or("None");
            let saison = grp.get_saison().unwrap_or("None");
            for participant in grp.participants.iter() {
                site_mbrs.entry(*participant).or_default().insert((saison, site));
            }
        }
    }
    // imprimer les fiches santés
    for (mid, _sites) in site_mbrs {
        { // block to auto drop the locks after usage
            let disc = _sites.into_iter().collect::<Vec<(&str, &str)>>();
            let membres = &program.membres;
            let comptes = &program.comptes;
            let config = &program.config;
            if let Ok(membre) = membres.get(mid) {
                let compte = comptes.get(membre.compte.unwrap_or_default()).unwrap_or(&NULL_COMPTE);
                let _res = print_fiche_med(membre, compte, config, &disc, false, out_dir.to_str(), &logger);
                if let Err(err) = _res {
                    logger(Desc::Error(format!("Erreur lors de l'impression de la fiche santé pour {mid}: {err}")));
                }
            } else {
                logger(Desc::Error(format!("Membre {mid} inexistant")));
            }
        }
    }
    Ok(())
}