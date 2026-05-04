use crate::{cli::{ProgramData, actions::ActionResult}, extract::excel::fill_regs, prelude::ErrorMessage};



pub fn charger_from_list_presence(program: &mut ProgramData) -> ActionResult {
    let filepath = rfd::FileDialog::new()
        .set_title("Sélectionner le fichier de présence")
        .add_filter("excel", &["xlsx"])
        .set_directory("/")
        .pick_file();
    if filepath.is_none() {
        let _ = program.err.write_line("Aucun fichier sélectionné.");
        return Err(Box::new(ErrorMessage::from("Aucun fichier sélectionné.")).into());
    }
    let filepath = filepath.unwrap().to_str().unwrap().to_string();
    //let filepath: String = read_file_path("Fichier xlsx: ");

    let res = fill_regs(&mut program.comptes, &mut program.membres, &mut program.groupes, &program.config, &filepath, &program.out, &program.err);
    if let Err(e) = res {
        let _ = program.err.write_line(&format!("{}", e));
        return Err(Box::new(e).into())
    }
    let _ = program.out.flush();
    let _ = program.err.flush();
    Ok(())
}