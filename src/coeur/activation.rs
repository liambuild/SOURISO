//! Quels effets sont actifs. Un effet est identifié par son nom : en ajouter un ne demande
//! aucune modification ici.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

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

/// Les effets activables, dans l'ordre où l'interface les présente.
pub const NOMS: [&str; 5] = [PROJECTEUR, ANNEAU, TRAINEE, ONDULATION, SON];

/// Nom stable d'un effet activable, à partir d'un texte reçu de l'extérieur.
pub fn nom_connu(nom: &str) -> Option<&'static str> {
    NOMS.into_iter().find(|n| *n == nom)
}

/// Les effets qu'un profil active : enregistrés dans le profil, appliqués quand on passe à lui.
/// Un champ absent du fichier vaut « actif », comme tout fichier d'avant ce réglage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ActifsProfil {
    pub projecteur: bool,
    pub anneau: bool,
    pub trainee: bool,
    pub ondulation: bool,
    pub son: bool,
}

impl Default for ActifsProfil {
    fn default() -> Self {
        ActifsProfil {
            projecteur: true,
            anneau: true,
            trainee: true,
            ondulation: true,
            son: true,
        }
    }
}

impl ActifsProfil {
    /// Vrai si le profil active cet effet ; un nom inconnu n'est pas actif.
    pub fn est_actif(&self, nom: &str) -> bool {
        match nom {
            PROJECTEUR => self.projecteur,
            ANNEAU => self.anneau,
            TRAINEE => self.trainee,
            ONDULATION => self.ondulation,
            SON => self.son,
            _ => false,
        }
    }

    /// Fixe l'activation d'un effet. Renvoie faux si le nom est inconnu.
    pub fn definir(&mut self, nom: &str, actif: bool) -> bool {
        let champ = match nom {
            PROJECTEUR => &mut self.projecteur,
            ANNEAU => &mut self.anneau,
            TRAINEE => &mut self.trainee,
            ONDULATION => &mut self.ondulation,
            SON => &mut self.son,
            _ => return false,
        };
        *champ = actif;
        true
    }
}

/// Tous les effets sont actifs au départ : seuls les effets coupés sont mémorisés.
#[derive(Debug, Default)]
pub struct Activations {
    coupes: HashSet<&'static str>,
}

impl Activations {
    pub fn est_actif(&self, nom: &str) -> bool {
        !self.coupes.contains(nom)
    }

    /// L'état de session, sous la forme de celui d'un profil.
    pub fn actifs(&self) -> ActifsProfil {
        let mut actifs = ActifsProfil::default();
        for nom in NOMS {
            actifs.definir(nom, self.est_actif(nom));
        }
        actifs
    }

    /// Remplace l'état de session par celui d'un profil : ce que fait le passage à ce profil.
    pub fn appliquer(&mut self, actifs: &ActifsProfil) {
        for nom in NOMS {
            self.definir(nom, actifs.est_actif(nom));
        }
    }

    /// Active ou coupe l'effet. Ne touche pas au profil : seul l'état de session change.
    pub fn definir(&mut self, nom: &'static str, actif: bool) {
        if actif {
            self.coupes.remove(nom);
        } else {
            self.coupes.insert(nom);
        }
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
    fn appliquer_un_profil_remplace_l_etat_de_session() {
        let mut a = Activations::default();
        a.basculer(PROJECTEUR);
        let mut profil = ActifsProfil::default();
        profil.definir(SON, false);
        a.appliquer(&profil);
        assert!(
            a.est_actif(PROJECTEUR),
            "coupé par raccourci, rétabli par le profil"
        );
        assert!(!a.est_actif(SON), "coupé par le profil");
        assert!(a.est_actif(ANNEAU));
    }

    #[test]
    fn l_etat_de_session_se_relit_comme_un_profil() {
        let mut a = Activations::default();
        a.basculer(ANNEAU);
        let p = a.actifs();
        assert!(!p.anneau && p.projecteur && p.son);
    }

    #[test]
    fn definir_ne_touche_que_l_effet_demande() {
        let mut a = Activations::default();
        a.definir(TRAINEE, false);
        assert!(!a.est_actif(TRAINEE) && a.est_actif(ANNEAU));
        a.definir(TRAINEE, true);
        assert!(a.est_actif(TRAINEE));
    }

    #[test]
    fn les_activations_d_un_profil_se_lisent_et_se_fixent_par_nom() {
        let mut p = ActifsProfil::default();
        assert!(NOMS.iter().all(|n| p.est_actif(n)));
        assert!(p.definir(ONDULATION, false));
        assert!(!p.est_actif(ONDULATION) && p.est_actif(SON));
        assert!(!p.definir("inconnu", false));
        assert!(!p.est_actif("inconnu"));
        assert_eq!(nom_connu("son"), Some(SON));
        assert_eq!(nom_connu("autre"), None);
    }

    #[test]
    fn un_champ_absent_du_fichier_vaut_actif() {
        let p: ActifsProfil = serde_json::from_str(r#"{"son": false}"#).unwrap();
        assert!(!p.son && p.projecteur && p.anneau && p.trainee && p.ondulation);
    }

    #[test]
    fn les_effets_sont_independants() {
        let mut a = Activations::default();
        a.basculer(PROJECTEUR);
        assert!(a.est_actif("autre"));
    }
}
