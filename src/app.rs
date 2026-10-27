//! Boucle principale : une surcouche par écran, les effets qui suivent le curseur sur l'écran où
//! il se trouve, un raccourci global par effet pour l'activer ou le couper, les clics lus dans la
//! même boucle que le curseur.

use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

use crate::coeur::activation::{Activations, ANNEAU, ONDULATION, PROJECTEUR, SON, TRAINEE};
use crate::coeur::clic::{Bouton, Detecteur};
use crate::coeur::son::avec_volume;
use crate::coeur::{Config, Ecran, Point};
use crate::plateforme::{self, Icone, RaccourciGlobal};
use crate::rendu::{Gpu, Rendu};
use crate::sonde::{self, Compteurs};

/// Cadence d'interrogation du curseur (environ 120 Hz). Le dessin n'a lieu que si l'image change.
const PERIODE: Duration = Duration::from_millis(8);

/// Intervalle entre deux écritures du bilan en cours dans la sonde.
const PERIODE_BILAN: Duration = Duration::from_secs(10);

/// Effets activables au clavier, dans l'ordre des raccourcis de la configuration.
const ACTIVABLES: [(&str, &str); 5] = [
    (PROJECTEUR, "Projecteur"),
    (ANNEAU, "Anneau"),
    (TRAINEE, "Traînée"),
    (ONDULATION, "Ondulation"),
    (SON, "Son"),
];

/// Délai minimal entre deux bascules : une touche maintenue répète l'événement du raccourci.
const ANTI_REPETITION: Duration = Duration::from_millis(400);

struct Surcouche {
    fenetre: Arc<Window>,
    ecran: Ecran,
    rendu: Rendu,
    /// Position du curseur sur cet écran, `None` s'il est ailleurs.
    curseur: Option<Point>,
    /// Vrai tant que l'image affichée ne correspond pas à l'état courant.
    a_redessiner: bool,
    /// Images réellement présentées sur cet écran, pour le bilan de la sonde.
    images: u64,
}

pub struct App {
    config: Config,
    activations: Activations,
    surcouches: Vec<Surcouche>,
    icone: Option<Icone>,
    raccourci: Option<RaccourciGlobal>,
    /// Dernière bascule de chaque effet de `ACTIVABLES`.
    dernieres_bascules: [Option<Instant>; ACTIVABLES.len()],
    detecteur: Detecteur,
    /// Son de clic mis au volume réglé, gardé pour toute l'exécution : la lecture asynchrone du
    /// système lit directement ces octets.
    son: &'static [u8],
    pub echec: Option<String>,
    debut: Instant,
    dernier_bilan: Instant,
    reveils: u64,
    lectures_curseur: u64,
    clics_gauche: u64,
    clics_droit: u64,
    sons_joues: u64,
}

/// Position du curseur sur cet écran, ou `None` s'il est sur un autre.
fn curseur_sur(ecran: &Ecran, lectures: &mut u64) -> Option<Point> {
    *lectures += 1;
    plateforme::position_curseur(ecran).filter(|p| ecran.contient(*p))
}

impl App {
    /// `debut` est l'instant de lancement, point de départ de la durée du bilan.
    pub fn new(debut: Instant) -> Self {
        let config = Config::default();
        let reglage_son = config.son.borne();
        // Fuite volontaire, unique : un seul son de quelques dizaines de kilo-octets pour toute la
        // durée du processus.
        let son =
            Box::leak(avec_volume(reglage_son.choix.wav(), reglage_son.volume).into_boxed_slice());
        App {
            config,
            activations: Activations::default(),
            surcouches: Vec::new(),
            icone: None,
            raccourci: None,
            dernieres_bascules: [None; ACTIVABLES.len()],
            detecteur: Detecteur::default(),
            son,
            echec: None,
            debut,
            dernier_bilan: debut,
            reveils: 0,
            lectures_curseur: 0,
            clics_gauche: 0,
            clics_droit: 0,
            sons_joues: 0,
        }
    }

    /// Ligne de bilan : durée, temps processeur et ce que la boucle a fait depuis le lancement.
    pub fn bilan(&self) -> String {
        let statistiques: Vec<_> = self
            .surcouches
            .iter()
            .map(|s| s.rendu.statistiques())
            .collect();
        let compteurs = Compteurs {
            reveils: self.reveils,
            lectures_curseur: self.lectures_curseur,
            images_par_ecran: self.surcouches.iter().map(|s| s.images).collect(),
            echecs_acquisition: statistiques.iter().map(|s| s.echecs_acquisition).sum(),
            reconfigurations: statistiques.iter().map(|s| s.reconfigurations).sum(),
            clics_gauche: self.clics_gauche,
            clics_droit: self.clics_droit,
            sons_joues: self.sons_joues,
        };
        sonde::bilan(
            self.debut.elapsed(),
            plateforme::temps_processeur(),
            &compteurs,
        )
    }

    fn demarrer(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let ecrans = plateforme::ecrans(event_loop)?;
        let mut fenetres = Vec::new();
        for ecran in &ecrans {
            fenetres.push(plateforme::creer_surcouche(event_loop, ecran)?);
        }

        let cartes = plateforme::cartes_ecrans(&ecrans);
        let (gpu, premiere) =
            Gpu::nouveau(event_loop.owned_display_handle(), &fenetres[0], &cartes)?;
        let mut premiere = Some(premiere);
        for (indice, (ecran, fenetre)) in ecrans.iter().zip(fenetres).enumerate() {
            let surface = match premiere.take() {
                Some(surface) => Ok(surface),
                None => gpu.surface(&fenetre),
            };
            let rendu = surface
                .and_then(|s| Rendu::nouveau(&gpu, s, &fenetre, ecran, indice, &self.config));
            match rendu {
                Ok(rendu) => self.surcouches.push(Surcouche {
                    fenetre,
                    ecran: *ecran,
                    rendu,
                    curseur: curseur_sur(ecran, &mut self.lectures_curseur),
                    a_redessiner: true,
                    images: 0,
                }),
                // L'écran principal est indispensable ; un écran supplémentaire qui échoue
                // reste simplement sans voile.
                Err(e) if indice == 0 => return Err(e),
                Err(e) => crate::sonde::ecrire(&format!("Sonde : écran {indice} ignoré : {e}")),
            }
        }

        self.icone = Some(plateforme::creer_icone()?);
        let raccourcis = [
            self.config.raccourci_projecteur,
            self.config.raccourci_anneau,
            self.config.raccourci_trainee,
            self.config.raccourci_ondulation,
            self.config.raccourci_son,
        ];
        match plateforme::creer_raccourcis(&raccourcis) {
            Ok(raccourci) => {
                for (indice, e) in &raccourci.refus {
                    crate::sonde::ecrire(&format!(
                        "Sonde : raccourci global « {} » indisponible : {e}",
                        ACTIVABLES[*indice].1
                    ));
                }
                self.raccourci = Some(raccourci);
            }
            // Sans raccourci, les effets restent actifs en permanence : pas une raison de quitter.
            Err(e) => crate::sonde::ecrire(&format!("Sonde : raccourci global indisponible : {e}")),
        }

        for s in &mut self.surcouches {
            let curseur = s.curseur;
            if s.rendu.dessiner(&s.ecran, || curseur, &self.activations) {
                s.images += 1;
                s.a_redessiner = false;
            }
            s.fenetre.set_visible(true);
        }
        Ok(())
    }

    /// Active ou coupe l'effet `ACTIVABLES[indice]`. Les effets se basculent indépendamment.
    fn basculer(&mut self, indice: usize) {
        let maintenant = Instant::now();
        if self.dernieres_bascules[indice]
            .is_some_and(|t| maintenant.duration_since(t) < ANTI_REPETITION)
        {
            return;
        }
        self.dernieres_bascules[indice] = Some(maintenant);
        let (nom, libelle) = ACTIVABLES[indice];
        let actif = self.activations.basculer(nom);
        crate::sonde::ecrire(&format!(
            "{libelle} {}",
            if actif { "activé" } else { "coupé" }
        ));
        for s in &mut self.surcouches {
            s.a_redessiner = true;
        }
    }

    /// Lit les boutons une fois, puis transmet chaque appui neuf à l'écran où se trouve le curseur
    /// et au son. Un appui compté l'est même si son effet est coupé : la sonde vérifie la détection.
    fn traiter_clics(&mut self) {
        let appuis = self.detecteur.appuis(plateforme::boutons());
        for (present, bouton) in [
            (appuis.gauche, Bouton::Gauche),
            (appuis.droit, Bouton::Droit),
        ] {
            if !present {
                continue;
            }
            match bouton {
                Bouton::Gauche => self.clics_gauche += 1,
                Bouton::Droit => self.clics_droit += 1,
            }
            for s in &mut self.surcouches {
                if let Some(position) = s.curseur {
                    // Pas de redessin demandé ici : si un effet a une animation à faire, la
                    // boucle le voit juste après avec `animation_en_cours`.
                    s.rendu.clic(bouton, position, &self.activations);
                }
            }
            if self.activations.est_actif(SON) && plateforme::jouer_son(self.son) {
                self.sons_joues += 1;
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.surcouches.is_empty() {
            return;
        }
        if let Err(erreur) = self.demarrer(event_loop) {
            self.echec = Some(erreur);
            event_loop.exit();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, evenement: WindowEvent) {
        let Some(s) = self.surcouches.iter_mut().find(|s| s.fenetre.id() == id) else {
            return;
        };
        match evenement {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(taille) => {
                s.rendu.redimensionner(taille.width, taille.height);
                s.a_redessiner = true;
            }
            WindowEvent::RedrawRequested => {
                let lectures = &mut self.lectures_curseur;
                let curseur = &mut s.curseur;
                let ecran = &s.ecran;
                let presentee = s.rendu.dessiner(
                    ecran,
                    || {
                        *curseur = curseur_sur(ecran, lectures);
                        *curseur
                    },
                    &self.activations,
                );
                s.a_redessiner = !presentee;
                s.images += u64::from(presentee);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.reveils += 1;
        if self.icone.as_ref().is_some_and(Icone::quitter_demande) {
            event_loop.exit();
            return;
        }
        for indice in self
            .raccourci
            .as_ref()
            .map(RaccourciGlobal::activations_demandees)
            .unwrap_or_default()
        {
            self.basculer(indice);
        }
        if self.dernier_bilan.elapsed() >= PERIODE_BILAN {
            self.dernier_bilan = Instant::now();
            sonde::bilan_en_cours(&self.bilan());
        }
        for s in &mut self.surcouches {
            let position = curseur_sur(&s.ecran, &mut self.lectures_curseur);
            if position != s.curseur {
                s.curseur = position;
                s.a_redessiner = true;
            }
        }
        // Après la lecture des curseurs : un clic va à l'écran où le curseur est maintenant.
        self.traiter_clics();
        for s in &mut self.surcouches {
            // Une animation en cours (ondulation, fondu de la traînée) demande des images, et
            // seulement elle : sans animation ni changement, aucune image n'est demandée.
            if s.a_redessiner || s.rendu.animation_en_cours(&self.activations) {
                s.fenetre.request_redraw();
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + PERIODE));
    }
}
