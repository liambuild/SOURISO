//! Quels effets sont actifs. Un effet est identifié par son nom : en ajouter un ne demande
//! aucune modification ici.

use std::collections::HashSet;

/// Nom de l'effet projecteur.
pub const PROJECTEUR: &str = "projecteur";
/// Nom de l'effet anneau.
pub const ANNEAU: &str = "anneau";
/// Nom de l'effet traînée.
pub const TRAINEE: &str = "trainee";
/// Nom de l'effet ondulation au clic.
pub const ONDULATION: &str = "ondulation";
/// Nom du son de clic : pas un effet de rendu, mais activable de la même façon.
pub const SON: &str = "son";

/// Tous les effets sont actifs au départ : seuls les effets coupés sont mémorisés.
#[derive(Debug, Default)]
pub struct Activations {
    coupes: HashSet<&'static str>,
}

impl Activations {
    pub fn est_actif(&self, nom: &str) -> bool {
        !self.coupes.contains(nom)
    }

    /// Active l'effet s'il était coupé, le coupe sinon. Renvoie le nouvel état.
    pub fn basculer(&mut self, nom: &'static str) -> bool {
        if !self.coupes.remove(nom) {
            self.coupes.insert(nom);
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_effet_est_actif_au_depart() {
        assert!(Activations::default().est_actif(PROJECTEUR));
    }

    #[test]
    fn basculer_coupe_puis_reactive() {
        let mut a = Activations::default();
        assert!(!a.basculer(PROJECTEUR));
        assert!(!a.est_actif(PROJECTEUR));
        assert!(a.basculer(PROJECTEUR));
        assert!(a.est_actif(PROJECTEUR));
    }

    #[test]
    fn les_effets_sont_independants() {
        let mut a = Activations::default();
        a.basculer(PROJECTEUR);
        assert!(a.est_actif("autre"));
    }
}
