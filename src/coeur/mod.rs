//! Cœur : modèle indépendant du système. Testable sous Linux.

pub mod activation;
pub mod clic;
pub mod config;
pub mod ondulation;
pub mod son;
pub mod trainee;

pub use config::{Config, Raccourci};

/// Position en pixels logiques, origine en haut à gauche de l'écran.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// Écran sur lequel vit une surcouche.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ecran {
    /// Origine dans le bureau virtuel, en pixels physiques.
    pub origine: (i32, i32),
    /// Taille en pixels physiques.
    pub taille: (u32, u32),
    pub echelle: f64,
    /// Taux de rafraîchissement en millihertz, tel que le système le rapporte.
    pub rafraichissement_mhz: Option<u32>,
}

impl Ecran {
    /// Vrai si la position, en pixels logiques de cet écran, tombe sur lui.
    pub fn contient(&self, p: Point) -> bool {
        let e = self.echelle as f32;
        p.x >= 0.0 && p.y >= 0.0 && p.x < self.taille.0 as f32 / e && p.y < self.taille.1 as f32 / e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_ecran_ne_contient_que_ses_propres_pixels() {
        let ecran = Ecran {
            origine: (1920, 0),
            taille: (3000, 1500),
            echelle: 1.5,
            rafraichissement_mhz: None,
        };
        assert!(ecran.contient(Point { x: 0.0, y: 0.0 }));
        assert!(ecran.contient(Point {
            x: 1999.0,
            y: 999.0
        }));
        assert!(!ecran.contient(Point { x: 2000.0, y: 10.0 }));
        assert!(!ecran.contient(Point { x: -1.0, y: 10.0 }));
        assert!(!ecran.contient(Point { x: 10.0, y: 1000.0 }));
    }
}
