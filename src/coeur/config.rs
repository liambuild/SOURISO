//! Configuration unique de l'application : tous les paramètres des effets et des raccourcis.
//! Les paramètres des effets (`Effets`) viennent du profil actif (`profil.rs`) ; les raccourcis
//! qui activent ou coupent chaque effet sont communs à tous les profils. Les distances sont en
//! pixels logiques, les durées en secondes, les opacités de 0 (invisible) à 1 (opaque).

use serde::{Deserialize, Serialize};

use super::son::Son;

/// Combinaison de touches globale : modificateurs et une lettre.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Raccourci {
    pub ctrl: bool,
    pub alt: bool,
    pub maj: bool,
    /// Lettre de A à Z.
    pub touche: char,
}

/// Voile qui assombrit l'écran, avec une zone éclairée autour du curseur.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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

impl Default for Projecteur {
    fn default() -> Self {
        Projecteur {
            rayon: 150.0,
            adoucissement: 100.0,
            intensite: 0.6,
            couleur: [0.0, 0.0, 0.0],
        }
    }
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

/// Anneau coloré autour du curseur.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Anneau {
    /// Rayon de l'anneau, mesuré au milieu de son trait : sa taille.
    pub rayon: f32,
    /// Épaisseur du trait.
    pub epaisseur: f32,
    /// Couleur RGB, composantes de 0 à 1.
    pub couleur: [f32; 3],
    pub opacite: f32,
}

impl Default for Anneau {
    fn default() -> Self {
        Anneau {
            rayon: 28.0,
            epaisseur: 4.0,
            couleur: [1.0, 0.8, 0.1],
            opacite: 0.9,
        }
    }
}

impl Anneau {
    pub fn borne(&self) -> Anneau {
        Anneau {
            rayon: self.rayon.max(1.0),
            epaisseur: self.epaisseur.max(1.0),
            couleur: self.couleur.map(|c| c.clamp(0.0, 1.0)),
            opacite: self.opacite.clamp(0.0, 1.0),
        }
    }
}

/// Traînée : des disques le long du chemin récent du curseur, qui s'effacent.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Trainee {
    /// Longueur maximale de la traînée derrière le curseur.
    pub longueur: f32,
    /// Durée que met un point à s'effacer : plus elle est courte, plus le fondu est rapide et la
    /// traînée courte à vitesse égale.
    pub duree_fondu: f32,
    /// Rayon des disques au départ.
    pub rayon: f32,
    pub couleur: [f32; 3],
    pub opacite: f32,
}

impl Default for Trainee {
    fn default() -> Self {
        Trainee {
            longueur: 180.0,
            duree_fondu: 0.45,
            rayon: 7.0,
            couleur: [0.3, 0.8, 1.0],
            opacite: 0.6,
        }
    }
}

impl Trainee {
    pub fn borne(&self) -> Trainee {
        Trainee {
            longueur: self.longueur.max(1.0),
            duree_fondu: self.duree_fondu.clamp(0.05, 5.0),
            rayon: self.rayon.max(1.0),
            couleur: self.couleur.map(|c| c.clamp(0.0, 1.0)),
            opacite: self.opacite.clamp(0.0, 1.0),
        }
    }
}

/// Ondulation au clic : un clic gauche fait un anneau plein sur un léger disque, un clic droit
/// deux anneaux concentriques, dans une autre couleur.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ondulation {
    /// Rayon que l'ondulation atteint à la fin.
    pub rayon_max: f32,
    pub duree: f32,
    pub epaisseur: f32,
    pub couleur_gauche: [f32; 3],
    pub couleur_droit: [f32; 3],
    pub opacite: f32,
}

impl Default for Ondulation {
    fn default() -> Self {
        Ondulation {
            rayon_max: 70.0,
            duree: 0.6,
            epaisseur: 3.0,
            couleur_gauche: [0.2, 0.55, 1.0],
            couleur_droit: [1.0, 0.5, 0.15],
            opacite: 0.85,
        }
    }
}

impl Ondulation {
    pub fn borne(&self) -> Ondulation {
        Ondulation {
            rayon_max: self.rayon_max.max(1.0),
            duree: self.duree.clamp(0.05, 5.0),
            epaisseur: self.epaisseur.max(1.0),
            couleur_gauche: self.couleur_gauche.map(|c| c.clamp(0.0, 1.0)),
            couleur_droit: self.couleur_droit.map(|c| c.clamp(0.0, 1.0)),
            opacite: self.opacite.clamp(0.0, 1.0),
        }
    }
}

/// Son joué à chaque clic, gauche ou droit.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SonClic {
    /// De 0 (muet) à 1 (le plein niveau du fichier).
    pub volume: f32,
    pub choix: Son,
}

impl Default for SonClic {
    fn default() -> Self {
        SonClic {
            volume: 0.5,
            choix: Son::Clic,
        }
    }
}

impl SonClic {
    pub fn borne(&self) -> SonClic {
        SonClic {
            volume: self.volume.clamp(0.0, 1.0),
            choix: self.choix,
        }
    }
}

/// Réglages de tous les effets : ce que contient un profil.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Effets {
    pub projecteur: Projecteur,
    pub anneau: Anneau,
    pub trainee: Trainee,
    pub ondulation: Ondulation,
    pub son: SonClic,
}

impl Effets {
    /// Ramène chaque réglage dans ses bornes.
    pub fn borne(&self) -> Effets {
        Effets {
            projecteur: self.projecteur.borne(),
            anneau: self.anneau.borne(),
            trainee: self.trainee.borne(),
            ondulation: self.ondulation.borne(),
            son: self.son.borne(),
        }
    }
}

/// Choix de la carte graphique. Les variables d'environnement `WGPU_ADAPTER_NAME` et
/// `WGPU_POWER_PREF` l'emportent sur ce réglage (`docs/ARCHITECTURE.md`, 4.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Adaptateur {
    /// Carte qui porte l'écran principal, sinon choix de wgpu.
    #[default]
    Automatique,
    /// Carte la plus économe (en général la carte intégrée).
    Econome,
    /// Carte la plus performante (en général la carte dédiée).
    Performant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Réglages des effets : copie de ceux du profil actif.
    pub effets: Effets,
    /// Active ou coupe le projecteur.
    pub raccourci_projecteur: Raccourci,
    /// Active ou coupe l'anneau.
    pub raccourci_anneau: Raccourci,
    /// Active ou coupe la traînée.
    pub raccourci_trainee: Raccourci,
    /// Active ou coupe l'ondulation au clic.
    pub raccourci_ondulation: Raccourci,
    /// Active ou coupe le son de clic.
    pub raccourci_son: Raccourci,
}

impl Config {
    /// Les raccourcis qui activent ou coupent un effet, dans l'ordre des effets activables.
    pub fn raccourcis_effets(&self) -> [Raccourci; 5] {
        [
            self.raccourci_projecteur,
            self.raccourci_anneau,
            self.raccourci_trainee,
            self.raccourci_ondulation,
            self.raccourci_son,
        ]
    }
}

/// Ctrl + Alt + Maj + une lettre : combinaison à quatre touches, inutilisée par le système.
pub const fn raccourci(touche: char) -> Raccourci {
    Raccourci {
        ctrl: true,
        alt: true,
        maj: true,
        touche,
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            effets: Effets::default(),
            raccourci_projecteur: raccourci('P'),
            raccourci_anneau: raccourci('A'),
            raccourci_trainee: raccourci('T'),
            raccourci_ondulation: raccourci('O'),
            raccourci_son: raccourci('S'),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_valeurs_par_defaut_sont_dans_leurs_bornes() {
        let e = Config::default().effets;
        assert_eq!(e, e.borne());
    }

    #[test]
    fn les_raccourcis_par_defaut_sont_distincts() {
        let touches = Config::default().raccourcis_effets().map(|r| r.touche);
        for (i, a) in touches.iter().enumerate() {
            assert!(!touches[i + 1..].contains(a), "touche {a} en double");
        }
    }

    #[test]
    fn les_nouveaux_effets_ramenent_leurs_valeurs_hors_bornes() {
        let a = Anneau {
            rayon: -1.0,
            epaisseur: 0.0,
            couleur: [2.0, -1.0, 0.5],
            opacite: 9.0,
        }
        .borne();
        assert_eq!((a.rayon, a.epaisseur, a.opacite), (1.0, 1.0, 1.0));
        assert_eq!(a.couleur, [1.0, 0.0, 0.5]);
        let t = Trainee {
            longueur: 0.0,
            duree_fondu: 0.0,
            ..Trainee::default()
        }
        .borne();
        assert_eq!((t.longueur, t.duree_fondu), (1.0, 0.05));
        let o = Ondulation {
            duree: 100.0,
            opacite: -1.0,
            ..Ondulation::default()
        }
        .borne();
        assert_eq!((o.duree, o.opacite), (5.0, 0.0));
        let s = SonClic {
            volume: 4.0,
            choix: Son::Cloche,
        }
        .borne();
        assert_eq!((s.volume, s.choix), (1.0, Son::Cloche));
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
