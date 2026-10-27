//! Clics de souris : état des boutons lu par la plateforme, front d'appui détecté ici.

/// Bouton de souris, tel que l'utilisateur le voit : le clic gauche logique reste le clic gauche
/// même si le système permute les boutons (la plateforme fait cette conversion).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bouton {
    Gauche,
    Droit,
}

/// État des deux boutons à un instant donné : vrai si le bouton est enfoncé.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Boutons {
    pub gauche: bool,
    pub droit: bool,
}

/// Transforme des états successifs en clics : un clic maintenu n'en produit qu'un, au moment de
/// l'appui. Un appui plus court que la période de lecture peut passer inaperçu.
#[derive(Debug, Default)]
pub struct Detecteur {
    precedent: Boutons,
}

impl Detecteur {
    /// Boutons qui viennent d'être enfoncés depuis la lecture précédente. `None` (lecture
    /// impossible) ne change rien : l'état précédent est conservé.
    pub fn appuis(&mut self, etat: Option<Boutons>) -> Boutons {
        let Some(etat) = etat else {
            return Boutons::default();
        };
        let appuis = Boutons {
            gauche: etat.gauche && !self.precedent.gauche,
            droit: etat.droit && !self.precedent.droit,
        };
        self.precedent = etat;
        appuis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn etat(gauche: bool, droit: bool) -> Option<Boutons> {
        Some(Boutons { gauche, droit })
    }

    #[test]
    fn un_clic_maintenu_ne_donne_qu_un_appui() {
        let mut d = Detecteur::default();
        assert!(d.appuis(etat(true, false)).gauche);
        assert_eq!(d.appuis(etat(true, false)), Boutons::default());
        assert_eq!(d.appuis(etat(true, false)), Boutons::default());
    }

    #[test]
    fn relacher_puis_appuyer_redonne_un_appui() {
        let mut d = Detecteur::default();
        d.appuis(etat(true, false));
        assert_eq!(d.appuis(etat(false, false)), Boutons::default());
        assert!(d.appuis(etat(true, false)).gauche);
    }

    #[test]
    fn les_deux_boutons_sont_independants() {
        let mut d = Detecteur::default();
        assert_eq!(
            d.appuis(etat(true, false)),
            Boutons {
                gauche: true,
                droit: false
            }
        );
        // Le droit s'ajoute pendant que le gauche est maintenu : seul le droit est neuf.
        assert_eq!(
            d.appuis(etat(true, true)),
            Boutons {
                gauche: false,
                droit: true
            }
        );
    }

    #[test]
    fn une_lecture_impossible_ne_change_rien() {
        let mut d = Detecteur::default();
        d.appuis(etat(true, false));
        assert_eq!(d.appuis(None), Boutons::default());
        // Toujours enfoncé : pas de nouvel appui.
        assert_eq!(d.appuis(etat(true, false)), Boutons::default());
    }
}
