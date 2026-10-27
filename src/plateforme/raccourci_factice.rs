//! Raccourci factice : pas de raccourci global hors Windows et macOS.

use crate::coeur::Raccourci;

pub struct RaccourciGlobal {
    /// Toujours vide : aucun raccourci n'est refusé, aucun n'est enregistré.
    pub refus: Vec<(usize, String)>,
}

impl RaccourciGlobal {
    pub fn creer(_raccourcis: &[Raccourci]) -> Result<Self, String> {
        Ok(RaccourciGlobal { refus: Vec::new() })
    }

    pub fn activations_demandees(&self) -> Vec<usize> {
        Vec::new()
    }
}
