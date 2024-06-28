use extract::presence::{GroupeExtractConfig, GroupeExtractData};

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
                let grpconfig = GroupeExtractConfig::default();
                for sheet in wb.sheet_names().unwrap() {
                    if sheet == "Groupes vides" {
                    } else {
                        let range = wb.worksheet_range(&sheet).unwrap();
                        match GroupeExtractData::extract(&grpconfig, &range) {
                            Err(e) => println!("Erreur dans le groupe"),
                            Ok(grp) => println!("{}", grp),
                        }
                    }
                }
            }
            Err(err) => println!("Erreur de lecture de fichier: {err}"),
        }
    } else {
        println!("Annulation");
    }
}
