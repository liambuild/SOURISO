//! Raccourci factice : pas de raccourci global hors Windows et macOS.

use crate::coeur::Raccourci;

pub struct RaccourciGlobal;

impl RaccourciGlobal {
    pub fn creer(_raccourci: &Raccourci) -> Result<Self, String> {
        Ok(RaccourciGlobal)
    }

    pub fn activation_demandee(&self) -> bool {
        false
    }
}
