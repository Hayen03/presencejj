use crate::{cli::{ProgramData, actions::ActionResult}, data::stats::{fill_stats, get_unique_stats, print_stats_to_excel}};


pub fn print_stats(program: &ProgramData, out: String, do_annulation: bool, do_attente: bool) -> ActionResult {
    let (stats, gstats) = fill_stats(program.groupes.groupes(), &program.membres, &program.comptes, do_annulation, do_attente);
    let ustats = get_unique_stats(program.groupes.groupes());

    let _res = print_stats_to_excel(&stats, &gstats, &program.groupes, &ustats, &out.to_string(), &program.out, &program.err);
    if let Err(e) = _res {
        let _ = program.err.write_line(&format!("Erreur lors de l'écriture des statistiques: {}", e));
        return Err(Box::new(e).into());
    }

    Ok(())
}