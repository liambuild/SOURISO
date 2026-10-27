//! État de la traînée : les derniers points du chemin du curseur sur un écran, chacun avec son
//! instant, pour qu'ils s'effacent au fil du temps.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::config;
use super::Point;

/// Nombre maximal de points mémorisés, quelle que soit la longueur demandée.
pub const MAX_POINTS: usize = 128;

#[derive(Debug, Default)]
pub struct Trainee {
    points: VecDeque<(Point, Instant)>,
    /// Position du dernier point ajouté, gardée quand les points sont effacés : un curseur
    /// immobile ne relance pas la traînée à chaque image dessinée pour une autre raison.
    ancre: Option<Point>,
}

/// Écart entre deux points de la traînée, en pixels logiques : la moitié du rayon, pour que les
/// disques se recouvrent.
pub fn pas(reglages: &config::Trainee) -> f32 {
    (reglages.rayon * 0.5).max(2.0)
}

impl Trainee {
    /// Ajoute le chemin parcouru jusqu'à `position`, interpolé tous les `pas` pixels pour qu'un
    /// mouvement rapide ne laisse pas de trous. Un curseur qui n'a pas bougé d'au moins un pas
    /// n'ajoute rien. Un saut plus long que la traînée (curseur venu d'un autre écran, par
    /// exemple) ou un chemin déjà tout effacé repart de zéro.
    pub fn ajouter(&mut self, position: Point, maintenant: Instant, reglages: &config::Trainee) {
        let pas = pas(reglages);
        let duree = Duration::from_secs_f32(reglages.duree_fondu);
        let distance = self
            .ancre
            .map(|a| (position.x - a.x).hypot(position.y - a.y));
        if distance.is_some_and(|d| d < pas) {
            return;
        }
        let recent = self
            .points
            .back()
            .copied()
            .filter(|(_, t)| maintenant.saturating_duration_since(*t) < duree);
        match (self.ancre, recent, distance) {
            (Some(ancre), Some((_, instant)), Some(distance))
                if distance <= reglages.longueur.max(pas) =>
            {
                let n = (distance / pas) as usize;
                let ecoule = maintenant.saturating_duration_since(instant);
                for k in 1..=n {
                    let f = k as f32 / n as f32;
                    self.points.push_back((
                        Point {
                            x: ancre.x + (position.x - ancre.x) * f,
                            y: ancre.y + (position.y - ancre.y) * f,
                        },
                        instant + ecoule.mul_f32(f),
                    ));
                }
            }
            _ => {
                self.points.clear();
                self.points.push_back((position, maintenant));
            }
        }
        self.ancre = Some(position);
        let maximum = ((reglages.longueur / pas).ceil() as usize + 1).clamp(1, MAX_POINTS);
        while self.points.len() > maximum {
            self.points.pop_front();
        }
    }

    /// Retire les points entièrement effacés.
    pub fn retenir(&mut self, maintenant: Instant, duree: Duration) {
        while self
            .points
            .front()
            .is_some_and(|(_, t)| maintenant.saturating_duration_since(*t) >= duree)
        {
            self.points.pop_front();
        }
    }

    /// Vrai tant qu'un point est en mémoire : le rendu doit encore produire des images, dont une
    /// dernière, vide, une fois `retenir` passé.
    pub fn en_cours(&self) -> bool {
        !self.points.is_empty()
    }

    /// Position et âge (de 0, neuf, à 1, effacé) de chaque point, du plus ancien au plus récent.
    pub fn ages(
        &self,
        maintenant: Instant,
        duree: Duration,
    ) -> impl Iterator<Item = (Point, f32)> + '_ {
        self.points.iter().map(move |(p, t)| {
            let age = maintenant.saturating_duration_since(*t);
            (
                *p,
                (age.as_secs_f32() / duree.as_secs_f32().max(1e-3)).min(1.0),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coeur::Config;

    fn reglages() -> config::Trainee {
        Config::default().effets.trainee
    }

    fn pt(x: f32) -> Point {
        Point { x, y: 0.0 }
    }

    fn duree(r: &config::Trainee) -> Duration {
        Duration::from_secs_f32(r.duree_fondu)
    }

    #[test]
    fn le_premier_point_est_la_position() {
        let r = reglages();
        let mut t = Trainee::default();
        assert!(!t.en_cours());
        t.ajouter(pt(5.0), Instant::now(), &r);
        assert!(t.en_cours());
    }

    #[test]
    fn un_mouvement_rapide_est_interpole_sans_trou() {
        let r = reglages();
        let p = pas(&r);
        let t0 = Instant::now();
        let mut t = Trainee::default();
        t.ajouter(pt(0.0), t0, &r);
        t.ajouter(pt(p * 5.0), t0 + Duration::from_millis(10), &r);
        let xs: Vec<f32> = t
            .ages(t0 + Duration::from_millis(10), duree(&r))
            .map(|(p, _)| p.x)
            .collect();
        assert_eq!(xs.len(), 6);
        assert!(xs.windows(2).all(|w| (w[1] - w[0] - p).abs() < 1e-3));
        assert!((xs[5] - p * 5.0).abs() < 1e-3);
    }

    #[test]
    fn la_traine_ne_depasse_pas_sa_longueur() {
        let r = reglages();
        let t0 = Instant::now();
        let mut t = Trainee::default();
        // Petits déplacements successifs bien plus longs que la traînée au total.
        for i in 0..400 {
            t.ajouter(pt(i as f32 * 2.0), t0, &r);
        }
        let n = t.ages(t0, duree(&r)).count();
        assert!(n <= MAX_POINTS);
        assert!(n as f32 * pas(&r) <= r.longueur + 2.0 * pas(&r));
    }

    #[test]
    fn un_curseur_immobile_ne_relance_pas_la_traine_une_fois_effacee() {
        let r = reglages();
        let d = duree(&r);
        let t0 = Instant::now();
        let mut t = Trainee::default();
        t.ajouter(pt(0.0), t0, &r);
        t.retenir(t0 + d, d);
        assert!(!t.en_cours());
        // Une image dessinée pour une autre raison, curseur toujours au même endroit.
        t.ajouter(pt(0.5), t0 + d, &r);
        assert!(!t.en_cours());
        // Dès qu'il bouge d'un pas, la traînée repart d'un seul point (pas d'interpolation depuis
        // un point effacé).
        t.ajouter(pt(pas(&r) * 3.0), t0 + d, &r);
        assert_eq!(t.ages(t0 + d, d).count(), 1);
    }

    #[test]
    fn un_saut_plus_long_que_la_traine_repart_de_zero() {
        let r = reglages();
        let t0 = Instant::now();
        let mut t = Trainee::default();
        t.ajouter(pt(0.0), t0, &r);
        t.ajouter(pt(r.longueur * 3.0), t0, &r);
        assert_eq!(t.ages(t0, duree(&r)).count(), 1);
    }

    #[test]
    fn les_points_s_effacent_avec_le_temps_puis_l_effet_s_arrete() {
        let r = reglages();
        let d = duree(&r);
        let t0 = Instant::now();
        let mut t = Trainee::default();
        t.ajouter(pt(0.0), t0, &r);
        t.retenir(t0 + d / 2, d);
        assert!(t.en_cours());
        let (_, age) = t.ages(t0 + d / 2, d).next().unwrap();
        assert!((age - 0.5).abs() < 1e-2);
        t.retenir(t0 + d, d);
        assert!(!t.en_cours());
    }
}
