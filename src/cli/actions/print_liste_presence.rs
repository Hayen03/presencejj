use crate::{cdj::groupes::NULL_GROUPE, cli::{ProgramData, actions::ActionResult}, print::typst::print_presence_anim};


pub fn print_presences_anim(program: &ProgramData, out_dir: Option<&str>) -> ActionResult {
    let mut compte = 0;
    for grp in program.groupes.groupes() {
        compte += 1;
        if grp == &(*NULL_GROUPE) {continue;}
        if grp.sous_groupe.is_empty() {
            print_presence_anim(grp, None, &program.membres, &program.comptes, &program.config, out_dir).expect("Oups");
        } else {
            for sg in &grp.sous_groupe {
                print_presence_anim(grp, Some(sg), &program.membres, &program.comptes, &program.config, out_dir).expect("AAAAAAh");
            }
        }
        
    }
    let _ = program.out.write_line(&format!("À imprimé {}/{} groupes", compte, program.groupes.len()));
    Ok(())
}