pub mod data;
pub mod extract;
pub mod groupes;
pub mod prelude;
pub mod print;
pub mod ui;

fn main() {
    let file = rfd::FileDialog::new().pick_file();
    if let Some(path) = file {
        match office::Excel::open(path) {
            Ok(mut wb) => {
                let nb_groupes = wb.sheet_names().unwrap().len() - 1;
                println!("{nb_groupes} groupes");
            }
            Err(err) => println!("Erreur de lecture de fichier: {err}"),
        }
    } else {
        println!("Annulation");
    }
}
