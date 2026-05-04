use console::style;
use office::Excel;

use crate::{cli::{ProgramData, actions::ActionResult}, prelude::ErrorMessage};


pub fn charger_from_prog(program: &mut ProgramData) -> ActionResult {
    let filepath = rfd::FileDialog::new()
        .set_title("Sélectionner le fichier de programmation")
        .add_filter("excel", &["xlsx"])
        .set_directory("/")
        .pick_file();
    if filepath.is_none() {
        let _ = program.err.write_line("Aucun fichier sélectionné.");
        return Err(Box::new(ErrorMessage::from("Aucun fichier sélectionné.")).into());
    }
    let filepath = filepath.unwrap().to_str().unwrap().to_string();

    let mut wb = match Excel::open(&filepath) {
        Ok(wb) => wb,
        Err(e) => {
            let _ = program.err.write_line(&format!("{}", e));
            let _ = program.err.flush();
            return Err(Box::new(e).into());
        },
    };
    let _ = program.out.write_line(&format!("Lecture de \"{}\"", style(filepath).green()));

    let sheets = wb.sheet_names().unwrap();
    for sheet in sheets {
        let rng = wb.worksheet_range(&sheet).unwrap();
        crate::extract::prog::fill_groupe_reg_from_prog(&rng, &mut program.groupes, &program.out, &program.err);
    }
    let _ = program.out.flush();
    let _ = program.err.flush();
    Ok(())
}