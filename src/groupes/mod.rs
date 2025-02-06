
pub mod comptes;
pub mod fiche_sante;
pub mod groupes;
pub mod membres;

#[derive(Debug)]
pub enum RegError<Key> {
    KeyAlreadyInReg(Key),
    NoSuchItem(Key),
}