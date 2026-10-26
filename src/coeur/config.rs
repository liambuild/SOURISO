//! Configuration unique de l'application : tous les paramètres des effets et des raccourcis.
//! Pas encore d'interface de réglage : les valeurs par défaut s'appliquent. Les distances sont en
//! pixels logiques.

/// Combinaison de touches globale : modificateurs et une lettre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Raccourci {
    pub ctrl: bool,
    pub alt: bool,
    pub maj: bool,
    /// Lettre de A à Z.
    pub touche: char,
}

/// Voile qui assombrit l'écran, avec une zone éclairée autour du curseur.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projecteur {
    /// Rayon de la zone entièrement éclairée.
    pub rayon: f32,
    /// Largeur de la bande où le voile passe progressivement de transparent à plein.
    pub adoucissement: f32,
    /// Opacité du voile loin du curseur, de 0 (invisible) à 1 (opaque).
    pub intensite: f32,
    /// Couleur RGB du voile, composantes de 0 à 1.
    pub couleur: [f32; 3],
}

impl Projecteur {
    /// Ramène les valeurs dans leurs bornes : une configuration fausse ne doit pas produire
    /// d'image absurde.
    pub fn borne(&self) -> Projecteur {
        Projecteur {
            rayon: self.rayon.max(0.0),
            adoucissement: self.adoucissement.max(0.0),
            intensite: self.intensite.clamp(0.0, 1.0),
            couleur: self.couleur.map(|c| c.clamp(0.0, 1.0)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub projecteur: Projecteur,
    /// Active ou coupe le projecteur.
    pub raccourci_projecteur: Raccourci,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            projecteur: Projecteur {
                rayon: 150.0,
                adoucissement: 100.0,
                intensite: 0.6,
                couleur: [0.0, 0.0, 0.0],
            },
            // Ctrl + Alt + Maj + P : combinaison à quatre touches, inutilisée par le système.
            raccourci_projecteur: Raccourci {
                ctrl: true,
                alt: true,
                maj: true,
                touche: 'P',
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_valeurs_par_defaut_sont_dans_leurs_bornes() {
        let c = Config::default();
        assert_eq!(c.projecteur, c.projecteur.borne());
    }

    #[test]
    fn les_valeurs_hors_bornes_sont_ramenees() {
        let p = Projecteur {
            rayon: -5.0,
            adoucissement: -1.0,
            intensite: 3.0,
            couleur: [-1.0, 0.5, 2.0],
        }
        .borne();
        assert_eq!(p.rayon, 0.0);
        assert_eq!(p.adoucissement, 0.0);
        assert_eq!(p.intensite, 1.0);
        assert_eq!(p.couleur, [0.0, 0.5, 1.0]);
    }
}
