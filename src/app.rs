//! Boucle principale : une surcouche par écran, les effets qui suivent le curseur sur l'écran où
//! il se trouve, un raccourci global pour activer ou couper le projecteur.

use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

use crate::coeur::activation::{Activations, PROJECTEUR};
use crate::coeur::{Config, Ecran, Point};
use crate::plateforme::{self, Icone, RaccourciGlobal};
use crate::rendu::{Gpu, Rendu};
use crate::sonde::{self, Compteurs};

/// Cadence d'interrogation du curseur (environ 120 Hz). Le dessin n'a lieu que si l'image change.
const PERIODE: Duration = Duration::from_millis(8);

/// Intervalle entre deux écritures du bilan en cours dans la sonde.
const PERIODE_BILAN: Duration = Duration::from_secs(10);

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
    derniere_bascule: Option<Instant>,
    pub echec: Option<String>,
    debut: Instant,
    dernier_bilan: Instant,
    reveils: u64,
    lectures_curseur: u64,
}

/// Position du curseur sur cet écran, ou `None` s'il est sur un autre.
fn curseur_sur(ecran: &Ecran, lectures: &mut u64) -> Option<Point> {
    *lectures += 1;
    plateforme::position_curseur(ecran).filter(|p| ecran.contient(*p))
}

impl App {
    /// `debut` est l'instant de lancement, point de départ de la durée du bilan.
    pub fn new(debut: Instant) -> Self {
        App {
            config: Config::default(),
            activations: Activations::default(),
            surcouches: Vec::new(),
            icone: None,
            raccourci: None,
            derniere_bascule: None,
            echec: None,
            debut,
            dernier_bilan: debut,
            reveils: 0,
            lectures_curseur: 0,
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

        let (gpu, premiere) = Gpu::nouveau(event_loop.owned_display_handle(), &fenetres[0])?;
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
        match plateforme::creer_raccourci(&self.config.raccourci_projecteur) {
            Ok(raccourci) => self.raccourci = Some(raccourci),
            // Sans raccourci, le projecteur reste actif en permanence : pas une raison de quitter.
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

    fn basculer_projecteur(&mut self) {
        let maintenant = Instant::now();
        if self
            .derniere_bascule
            .is_some_and(|t| maintenant.duration_since(t) < ANTI_REPETITION)
        {
            return;
        }
        self.derniere_bascule = Some(maintenant);
        let actif = self.activations.basculer(PROJECTEUR);
        crate::sonde::ecrire(&format!(
            "Projecteur {}",
            if actif { "activé" } else { "coupé" }
        ));
        for s in &mut self.surcouches {
            s.a_redessiner = true;
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
        if self
            .raccourci
            .as_ref()
            .is_some_and(RaccourciGlobal::activation_demandee)
        {
            self.basculer_projecteur();
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
            if s.a_redessiner {
                s.fenetre.request_redraw();
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + PERIODE));
    }
}
