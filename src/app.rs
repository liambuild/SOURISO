//! Boucle principale : une surcouche sur l'écran principal, un repère qui suit le curseur.

use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

use crate::coeur::{Ecran, Point};
use crate::plateforme::{self, Icone};
use crate::rendu::Rendu;

/// Cadence d'interrogation du curseur (environ 120 Hz). Le dessin n'a lieu que si le curseur bouge.
const PERIODE: Duration = Duration::from_millis(8);

struct Surcouche {
    fenetre: Arc<Window>,
    ecran: Ecran,
    rendu: Rendu,
    icone: Icone,
    dernier: Option<Point>,
}

#[derive(Default)]
pub struct App {
    surcouche: Option<Surcouche>,
    pub echec: Option<String>,
}

impl App {
    fn demarrer(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let ecran = plateforme::ecran_principal(event_loop)?;
        let fenetre = plateforme::creer_surcouche(event_loop, &ecran)?;
        let mut rendu = Rendu::nouveau(fenetre.clone(), event_loop.owned_display_handle())?;
        let icone = plateforme::creer_icone()?;
        let dernier = plateforme::position_curseur(&ecran);
        rendu.dessiner(&ecran, dernier);
        fenetre.set_visible(true);
        self.surcouche = Some(Surcouche {
            fenetre,
            ecran,
            rendu,
            icone,
            dernier,
        });
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.surcouche.is_some() {
            return;
        }
        if let Err(erreur) = self.demarrer(event_loop) {
            self.echec = Some(erreur);
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        evenement: WindowEvent,
    ) {
        let Some(s) = self.surcouche.as_mut() else {
            return;
        };
        match evenement {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(taille) => s.rendu.redimensionner(taille.width, taille.height),
            WindowEvent::RedrawRequested => s.rendu.dessiner(&s.ecran, s.dernier),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(s) = self.surcouche.as_mut() {
            if s.icone.quitter_demande() {
                event_loop.exit();
                return;
            }
            let position = plateforme::position_curseur(&s.ecran);
            if position != s.dernier {
                s.dernier = position;
                s.fenetre.request_redraw();
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + PERIODE));
    }
}
