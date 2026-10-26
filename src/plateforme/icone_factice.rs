//! Icône factice : pas de zone de notification hors Windows et macOS.

pub struct Icone;

impl Icone {
    pub fn creer() -> Result<Self, String> {
        Ok(Icone)
    }

    pub fn quitter_demande(&self) -> bool {
        false
    }
}
