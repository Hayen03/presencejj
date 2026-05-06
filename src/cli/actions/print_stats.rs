use crate::{cdj::groupes::GroupeID, cli::{ProgramData, actions::ActionResult}, data::stats::{FillStats, StatsToExcel, get_unique_stats}, ui::screens::Desc};


pub fn print_stats(program: &ProgramData, out: String, do_annulation: bool, do_attente: bool) -> ActionResult {
    let get_annulation = if do_annulation { get_nb_annulations } else { no_input };
    let get_attente = if do_attente { get_nb_attente } else { no_input };
    let get_missing_capacite = |_gid: GroupeID, desc: &str| read_user_input(&format!("Entrez la capacite pour le groupe {}", desc), usize_validation);
    let logger = |msg: Desc| {
        let _ = program.out.write_line(msg.as_str());
    };

    let (stats, gstats) = {
        let mut grps = program.groupes.groupes();
        FillStats {
            groupes: &mut grps,
            membres: &program.membres,
            comptes: &program.comptes,
            do_annulation: &get_annulation,
            do_attente: &get_attente,
            get_missing_capacite: &get_missing_capacite,
            progress: None,
            cancel: None,
        }.fill().expect("Should be Some(_)")
    };
    let ustats = get_unique_stats(program.groupes.groupes(), None, None).expect("Should be Some(_)");

    let _res = StatsToExcel {
        stats: &stats,
        gstats: &gstats,
        groupes: &program.groupes,
        ustats: &ustats,
        out: &out,
        logger: &logger,
        progress: None,
        cancel: None,
    }.print();
    if let Err(e) = _res {
        let _ = program.err.write_line(&format!("Erreur lors de l'écriture des statistiques: {}", e));
        return Err(Box::new(e).into());
    }

    Ok(())
}

fn get_nb_annulations(_gid: GroupeID, desc: &str) -> Option<usize> {
    Some(read_user_input(&format!("Entrez le nombre d'annulation pour le groupe {desc}"), usize_validation))
}
fn get_nb_attente(_gid: GroupeID, desc: &str) -> Option<usize> {
    Some(read_user_input(&format!("Entrez le nombre de participant sur la liste d'attente pour le groupe {desc}"), usize_validation))
}
fn no_input(_gid: GroupeID, _desc: &str) -> Option<usize> {
    None
}

fn read_user_input<T>(prompt: &str, valid: impl Fn(&str) -> Result<T, String>) -> T {
	loop {
		let input: Result<String, dialoguer::Error> = dialoguer::Input::new().with_prompt(prompt).interact();
		match input {
			Ok(input) => {
				match valid(&input) {
					Ok(value) => {return value; },
					Err(err) => {
						println!("{}", err);
					}
				}
			},
			Err(_) => {
				println!("Entrée invalide, essayez à nouveau.");
			}
		}
	}
}
fn usize_validation(input: &str) -> Result<usize, String> {
	match input.parse::<usize>() {
		Ok(num) => Ok(num),
		Err(_) => Err("Veuillez entrer un nombre entier positif.".to_string()),
	}
}