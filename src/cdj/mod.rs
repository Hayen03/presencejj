
pub mod comptes;
pub mod fiche_sante;
pub mod groupes;
pub mod membres;

#[derive(Debug)]
pub enum RegError<Key> {
    KeyAlreadyInReg(Key),
    NoSuchItem(Key),
}
impl<Key> std::fmt::Display for RegError<Key> where Key: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegError::KeyAlreadyInReg(key) => write!(f, "Key already in registry: {}", key),
            RegError::NoSuchItem(key) => write!(f, "No such item in registry: {}", key),
        }
    }
}
impl<Key> std::error::Error for RegError<Key> where Key: std::fmt::Display + std::fmt::Debug {}