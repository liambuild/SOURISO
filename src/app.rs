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

/// Cadence d'interrogation du curseur (environ 120 Hz). Le dessin n'a lieu que si l'image change.
const PERIODE: Duration = Duration::from_millis(8);

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
}

#[derive(Default)]
pub struct App {
    config: Config,
    activations: Activations,
    surcouches: Vec<Surcouche>,
    icone: Option<Icone>,
    raccourci: Option<RaccourciGlobal>,
    derniere_bascule: Option<Instant>,
    pub echec: Option<String>,
    /// Images réellement présentées depuis le lancement, pour le bilan de la sonde.
    pub images: u64,
}

/// Position du curseur sur cet écran, ou `None` s'il est sur un autre.
fn curseur_sur(ecran: &Ecran) -> Option<Point> {
    plateforme::position_curseur(ecran).filter(|p| ecran.contient(*p))
}

impl App {
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
            let rendu =
                surface.and_then(|s| Rendu::nouveau(&gpu, s, &fenetre, indice, &self.config));
            match rendu {
                Ok(rendu) => self.surcouches.push(Surcouche {
                    fenetre,
                    ecran: *ecran,
                    rendu,
                    curseur: curseur_sur(ecran),
                    a_redessiner: true,
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
            if s.rendu.dessiner(&s.ecran, s.curseur, &self.activations) {
                self.images += 1;
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
                let presentee = s.rendu.dessiner(&s.ecran, s.curseur, &self.activations);
                s.a_redessiner = !presentee;
                self.images += u64::from(presentee);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
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
        for s in &mut self.surcouches {
            let position = curseur_sur(&s.ecran);
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
