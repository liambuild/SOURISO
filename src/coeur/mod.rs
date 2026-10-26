//! Cœur : modèle indépendant du système. Testable sous Linux.

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
}

impl Ecran {
    /// Convertit une position logique de l'écran en pixels physiques de sa surface.
    pub fn vers_physique(&self, p: Point) -> (f32, f32) {
        let e = self.echelle as f32;
        (p.x * e, p.y * e)
    }
}

/// Paramètres du repère qui suit le curseur, en pixels logiques.
#[derive(Clone, Copy, Debug)]
pub struct Repere {
    pub rayon: f32,
    pub epaisseur: f32,
    /// Couleur RGB, composantes de 0 à 1.
    pub couleur: [f32; 3],
}

pub const REPERE: Repere = Repere {
    rayon: 28.0,
    epaisseur: 4.0,
    couleur: [1.0, 0.78, 0.1],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_logique_vers_physique() {
        let ecran = Ecran {
            origine: (0, 0),
            taille: (3840, 2160),
            echelle: 1.5,
        };
        assert_eq!(
            ecran.vers_physique(Point { x: 100.0, y: 40.0 }),
            (150.0, 60.0)
        );
    }
}
