//! Icône factice : pas de zone de notification hors Windows et macOS.

use super::EvenementsIcone;

pub struct Icone;

impl Icone {
    pub fn creer() -> Result<Self, String> {
        Ok(Icone)
    }

    pub fn evenements(&self) -> EvenementsIcone {
        EvenementsIcone::default()
    }
}
