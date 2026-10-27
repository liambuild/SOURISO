//! État de l'ondulation au clic : les ondulations en cours sur un écran.

use std::time::{Duration, Instant};

use super::clic::Bouton;
use super::Point;

/// Nombre maximal d'ondulations simultanées par écran. Au-delà, la plus ancienne disparaît : des
/// clics très rapides ne font pas grossir sans limite ce que le GPU reçoit.
pub const MAX: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ondulation {
    position: Point,
    bouton: Bouton,
    debut: Instant,
}

#[derive(Debug, Default)]
pub struct Ondulations {
    en_cours: Vec<Ondulation>,
}

impl Ondulations {
    /// Démarre une ondulation à `position` (pixels logiques de l'écran).
    pub fn ajouter(&mut self, position: Point, bouton: Bouton, maintenant: Instant) {
        if self.en_cours.len() == MAX {
            self.en_cours.remove(0);
        }
        self.en_cours.push(Ondulation {
            position,
            bouton,
            debut: maintenant,
        });
    }

    /// Retire les ondulations terminées.
    pub fn retenir(&mut self, maintenant: Instant, duree: Duration) {
        self.en_cours
            .retain(|o| maintenant.saturating_duration_since(o.debut) < duree);
    }

    /// Vrai tant qu'une ondulation est en mémoire : le rendu doit encore produire des images, dont
    /// une dernière, vide, une fois `retenir` passé.
    pub fn en_cours(&self) -> bool {
        !self.en_cours.is_empty()
    }

    /// Position, bouton et avancement (de 0 à 1) de chaque ondulation en cours.
    pub fn avancements(
        &self,
        maintenant: Instant,
        duree: Duration,
    ) -> impl Iterator<Item = (Point, Bouton, f32)> + '_ {
        self.en_cours.iter().map(move |o| {
            let age = maintenant.saturating_duration_since(o.debut);
            let avancement = (age.as_secs_f32() / duree.as_secs_f32().max(1e-3)).min(1.0);
            (o.position, o.bouton, avancement)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: Point = Point { x: 10.0, y: 20.0 };
    const DUREE: Duration = Duration::from_millis(500);

    #[test]
    fn une_ondulation_vit_sa_duree_puis_disparait() {
        let t0 = Instant::now();
        let mut o = Ondulations::default();
        assert!(!o.en_cours());
        o.ajouter(P, Bouton::Gauche, t0);
        assert!(o.en_cours());
        o.retenir(t0 + Duration::from_millis(499), DUREE);
        assert!(o.en_cours());
        o.retenir(t0 + DUREE, DUREE);
        assert!(!o.en_cours());
    }

    #[test]
    fn l_avancement_va_de_zero_a_un() {
        let t0 = Instant::now();
        let mut o = Ondulations::default();
        o.ajouter(P, Bouton::Droit, t0);
        let a: Vec<_> = o
            .avancements(t0 + Duration::from_millis(250), DUREE)
            .collect();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].0, P);
        assert_eq!(a[0].1, Bouton::Droit);
        assert!((a[0].2 - 0.5).abs() < 1e-3);
    }

    #[test]
    fn au_dela_du_maximum_la_plus_ancienne_disparait() {
        let t0 = Instant::now();
        let mut o = Ondulations::default();
        for i in 0..MAX + 3 {
            o.ajouter(
                Point {
                    x: i as f32,
                    y: 0.0,
                },
                Bouton::Gauche,
                t0,
            );
        }
        let xs: Vec<f32> = o.avancements(t0, DUREE).map(|(p, _, _)| p.x).collect();
        assert_eq!(xs.len(), MAX);
        assert_eq!(xs[0], 3.0);
        assert_eq!(xs[MAX - 1], (MAX + 2) as f32);
    }
}
