use serde::{Deserialize, Serialize, ser::SerializeStruct};

use crate::cdj::{comptes::CompteReg, groupes::GroupeReg, membres::MembreReg};


pub struct SaveData<'a> {
	pub membres: &'a MembreReg,
	pub comptes: &'a CompteReg,
	pub groupes: &'a GroupeReg,
}
impl Serialize for SaveData<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("SaveData", 3)?;
		state.serialize_field("membres", self.membres)?;
		state.serialize_field("comptes", self.comptes)?;
		state.serialize_field("groupes", self.groupes)?;
		state.end()
	}
}

#[derive(Deserialize)]
pub struct LoadData {
	pub membres: MembreReg,
	pub comptes: CompteReg,
	pub groupes: GroupeReg,
}