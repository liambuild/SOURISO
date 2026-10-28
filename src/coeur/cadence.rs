//! Cadence de la boucle : le curseur et les boutons s'interrogent (aucun événement ne les
//! signale), donc chaque réveil coûte. Au repos la boucle s'espace ; au premier signe d'activité
//! elle revient à la cadence normale. Modèle indépendant du système, testable sous Linux.

use std::time::{Duration, Instant};

/// Intervalle entre deux interrogations en activité (environ 120 Hz).
pub const PERIODE_ACTIVE: Duration = Duration::from_millis(8);

/// Intervalle au repos (environ 60 Hz). Il borne le retard de la première image après un
/// mouvement (au plus une image à 60 Hz, une seule fois : la cadence normale revient aussitôt) et
/// la durée minimale d'un appui détecté, qui reste sous celle d'un clic humain, même bref.
pub const PERIODE_REPOS: Duration = Duration::from_millis(16);

/// Durée sans aucune activité avant de passer au repos.
pub const DELAI_REPOS: Duration = Duration::from_secs(1);

/// Repos ou activité de la boucle, d'après le dernier signe d'activité.
#[derive(Clone, Copy, Debug)]
pub struct Cadence {
    derniere_activite: Instant,
}

impl Cadence {
    /// L'application démarre en activité : le repos ne vient qu'après `DELAI_REPOS` de calme.
    pub fn new(maintenant: Instant) -> Self {
        Cadence {
            derniere_activite: maintenant,
        }
    }

    /// Note un signe d'activité : curseur bougé, bouton pressé, animation ou image en attente,
    /// fenêtre de réglages ouverte, raccourci ou menu de l'icône utilisé.
    pub fn noter_activite(&mut self, maintenant: Instant) {
        self.derniere_activite = maintenant;
    }

    pub fn au_repos(&self, maintenant: Instant) -> bool {
        maintenant.saturating_duration_since(self.derniere_activite) >= DELAI_REPOS
    }

    /// Intervalle avant le prochain réveil.
    pub fn periode(&self, maintenant: Instant) -> Duration {
        if self.au_repos(maintenant) {
            PERIODE_REPOS
        } else {
            PERIODE_ACTIVE
        }
    }
}

/// Vrai si l'écran doit recevoir une image : son contenu ne correspond plus à l'état courant, ou
/// un effet a une animation à faire avancer ou à effacer. Un écran qui ne porte pas le curseur et
/// n'a aucune animation n'en reçoit aucune.
pub fn image_necessaire(a_redessiner: bool, animation_en_cours: bool) -> bool {
    a_redessiner || animation_en_cours
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coeur::{Ecran, Point};

    fn t(ms: u64, depart: Instant) -> Instant {
        depart + Duration::from_millis(ms)
    }

    #[test]
    fn la_boucle_demarre_en_activite() {
        let depart = Instant::now();
        let c = Cadence::new(depart);
        assert!(!c.au_repos(depart));
        assert_eq!(c.periode(depart), PERIODE_ACTIVE);
    }

    #[test]
    fn la_boucle_ralentit_apres_un_calme_assez_long_et_pas_avant() {
        let depart = Instant::now();
        let c = Cadence::new(depart);
        assert_eq!(c.periode(t(999, depart)), PERIODE_ACTIVE);
        assert_eq!(c.periode(t(1000, depart)), PERIODE_REPOS);
        assert_eq!(c.periode(t(60_000, depart)), PERIODE_REPOS);
        assert!(PERIODE_REPOS > PERIODE_ACTIVE);
    }

    #[test]
    fn le_premier_signe_d_activite_ramene_la_cadence_normale() {
        let depart = Instant::now();
        let mut c = Cadence::new(depart);
        assert_eq!(c.periode(t(5_000, depart)), PERIODE_REPOS);
        c.noter_activite(t(5_000, depart));
        assert_eq!(c.periode(t(5_000, depart)), PERIODE_ACTIVE);
        assert_eq!(c.periode(t(5_999, depart)), PERIODE_ACTIVE);
        assert_eq!(c.periode(t(6_000, depart)), PERIODE_REPOS);
    }

    #[test]
    fn une_activite_continue_garde_la_cadence_normale() {
        let depart = Instant::now();
        let mut c = Cadence::new(depart);
        for ms in (0..10_000).step_by(8) {
            c.noter_activite(t(ms, depart));
            assert_eq!(c.periode(t(ms, depart)), PERIODE_ACTIVE);
        }
    }

    #[test]
    fn une_horloge_qui_recule_ne_met_pas_au_repos() {
        let depart = Instant::now();
        let c = Cadence::new(t(5_000, depart));
        assert!(!c.au_repos(depart));
    }

    #[test]
    fn sur_une_minute_de_repos_la_boucle_se_reveille_au_plus_au_rythme_du_repos() {
        // Même comptage que la boucle réelle : chaque réveil planifie le suivant `periode` plus tard.
        let reveils_en = |repos: bool| {
            let depart = Instant::now();
            let mut c = Cadence::new(depart);
            let (mut maintenant, mut reveils) = (depart, 0u64);
            while maintenant < t(60_000, depart) {
                reveils += 1;
                if !repos {
                    c.noter_activite(maintenant);
                }
                maintenant += c.periode(maintenant);
            }
            reveils
        };
        let (actif, repos) = (reveils_en(false), reveils_en(true));
        assert_eq!(actif, 7_500, "60 s à 8 ms");
        assert_eq!(repos, 125 + 3_687 + 1, "1 s à 8 ms, puis 59 s à 16 ms");
        assert!(repos * 2 < actif + 400, "environ moitié moins de réveils");
    }

    #[test]
    fn un_ecran_sans_curseur_ni_animation_ne_recoit_aucune_image() {
        assert!(!image_necessaire(false, false));
        assert!(image_necessaire(true, false));
        assert!(image_necessaire(false, true));
    }

    /// Conversion du bureau physique vers les pixels logiques d'un écran, comme la couche
    /// plateforme de Windows (`GetCursorPos`). La conversion réelle n'est pas testée ici.
    fn position_locale(ecran: &Ecran, x: i32, y: i32) -> Option<Point> {
        let e = ecran.echelle as f32;
        let p = Point {
            x: (x - ecran.origine.0) as f32 / e,
            y: (y - ecran.origine.1) as f32 / e,
        };
        ecran.contient(p).then_some(p)
    }

    /// Un réveil de la boucle sur plusieurs écrans : mêmes appels que `App::about_to_wait`.
    struct Simulation {
        ecrans: Vec<Ecran>,
        curseur: Vec<Option<Point>>,
        a_redessiner: Vec<bool>,
        images: Vec<u64>,
        cadence: Cadence,
    }

    impl Simulation {
        fn new(ecrans: Vec<Ecran>, x: i32, y: i32, depart: Instant) -> Self {
            let n = ecrans.len();
            Simulation {
                curseur: ecrans.iter().map(|e| position_locale(e, x, y)).collect(),
                a_redessiner: vec![true; n],
                images: vec![0; n],
                ecrans,
                cadence: Cadence::new(depart),
            }
        }

        /// Un réveil : le curseur est en (x, y). Renvoie les écrans qui ont reçu une image.
        fn reveil(&mut self, x: i32, y: i32, maintenant: Instant) -> Vec<usize> {
            let mut active = false;
            for (i, e) in self.ecrans.iter().enumerate() {
                let position = position_locale(e, x, y);
                if position != self.curseur[i] {
                    self.curseur[i] = position;
                    self.a_redessiner[i] = true;
                }
                active |= self.a_redessiner[i];
            }
            if active {
                self.cadence.noter_activite(maintenant);
            }
            let mut dessines = Vec::new();
            for i in 0..self.ecrans.len() {
                if image_necessaire(self.a_redessiner[i], false) {
                    self.images[i] += 1;
                    self.a_redessiner[i] = false;
                    dessines.push(i);
                }
            }
            dessines
        }
    }

    /// Trois écrans côte à côte, d'échelles 100 %, 150 % et 200 %.
    fn trois_ecrans() -> Vec<Ecran> {
        let ecran = |origine_x, largeur, echelle| Ecran {
            origine: (origine_x, 0),
            taille: (largeur, 1080),
            echelle,
            rafraichissement_mhz: None,
        };
        vec![
            ecran(0, 1920, 1.0),
            ecran(1920, 2880, 1.5),
            ecran(4800, 3840, 2.0),
        ]
    }

    #[test]
    fn au_repos_aucun_ecran_ne_dessine_et_le_premier_mouvement_redonne_la_cadence_normale() {
        let depart = Instant::now();
        let mut s = Simulation::new(trois_ecrans(), 500, 500, depart);
        assert_eq!(s.reveil(500, 500, depart), vec![0, 1, 2], "première image");
        let mut maintenant = depart;
        for _ in 0..200 {
            maintenant += s.cadence.periode(maintenant);
            assert!(s.reveil(500, 500, maintenant).is_empty());
        }
        assert!(
            s.cadence.au_repos(maintenant),
            "plus d'une seconde de calme"
        );
        assert_eq!(s.images, vec![1, 1, 1], "pas une image de plus au repos");

        maintenant += s.cadence.periode(maintenant);
        assert_eq!(
            s.reveil(510, 500, maintenant),
            vec![0],
            "seul l'écran du curseur"
        );
        assert!(!s.cadence.au_repos(maintenant));
        assert_eq!(s.cadence.periode(maintenant), PERIODE_ACTIVE);
        assert_eq!(s.images, vec![2, 1, 1]);
    }

    #[test]
    fn le_passage_d_un_ecran_a_l_autre_efface_le_premier_et_affiche_le_second_au_meme_reveil() {
        let depart = Instant::now();
        let mut s = Simulation::new(trois_ecrans(), 1900, 500, depart);
        s.reveil(1900, 500, depart);
        assert!(s.curseur[0].is_some() && s.curseur[1].is_none());

        // Franchissement de la jonction 0 → 1, puis 1 → 2 : l'écran quitté et l'écran atteint
        // reçoivent une image dans le même réveil, le troisième rien.
        assert_eq!(s.reveil(1925, 500, depart), vec![0, 1]);
        assert!(s.curseur[0].is_none(), "l'écran quitté est vidé du curseur");
        assert_eq!(
            s.curseur[1],
            Some(Point {
                x: 5.0 / 1.5,
                y: 500.0 / 1.5
            })
        );
        assert_eq!(s.reveil(1930, 500, depart), vec![1]);
        assert_eq!(s.reveil(4805, 500, depart), vec![1, 2]);
        assert_eq!(s.curseur[2], Some(Point { x: 2.5, y: 250.0 }));
        assert!(s.curseur[1].is_none());
        assert_eq!(s.images, vec![2, 4, 2]);
        // Retour d'un pixel en arrière sur la jonction 1 → 2.
        assert_eq!(s.reveil(4799, 0, depart), vec![1, 2]);
    }

    #[test]
    fn un_curseur_hors_de_tous_les_ecrans_ne_dessine_rien_de_plus() {
        let depart = Instant::now();
        let mut s = Simulation::new(trois_ecrans(), 100, 100, depart);
        s.reveil(100, 100, depart);
        assert_eq!(
            s.reveil(20_000, 100, depart),
            vec![0],
            "l'écran quitté est effacé"
        );
        assert!(s.reveil(20_050, 100, depart).is_empty());
    }
}
