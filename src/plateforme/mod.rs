//! Couche plateforme : seul endroit qui connaît le système.
//! Le contrat est : boucle d'événements, surcouche passe-clics, position du curseur,
//! icône de barre de menus ou de zone de notification.

use std::sync::Arc;

use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;

use crate::coeur::{Ecran, Point};

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod factice;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use factice as systeme;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as systeme;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as systeme;

#[cfg(any(target_os = "windows", target_os = "macos"))]
mod icone_systeme;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use icone_systeme as icone;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod icone_factice;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use icone_factice as icone;

pub use icone::Icone;

/// Crée la boucle d'événements avec les réglages propres au système.
pub fn creer_boucle() -> Result<EventLoop<()>, String> {
    systeme::creer_boucle()
}

/// Décrit l'écran principal dans le repère normalisé de l'application.
pub fn ecran_principal(event_loop: &ActiveEventLoop) -> Result<Ecran, String> {
    let moniteur = event_loop
        .primary_monitor()
        .or_else(|| event_loop.available_monitors().next())
        .ok_or("aucun écran détecté")?;
    let position = moniteur.position();
    let taille = moniteur.size();
    Ok(Ecran {
        origine: (position.x, position.y),
        taille: (taille.width, taille.height),
        echelle: moniteur.scale_factor(),
    })
}

/// Crée la surcouche : transparente, sans cadre, au-dessus de tout, qui laisse passer les clics.
/// Elle reste invisible tant que le rendu n'est pas prêt.
pub fn creer_surcouche(event_loop: &ActiveEventLoop, ecran: &Ecran) -> Result<Arc<Window>, String> {
    let fenetre = systeme::creer_surcouche(event_loop, ecran)?;
    fenetre
        .set_cursor_hittest(false)
        .map_err(|e| format!("surcouche non traversable : {e}"))?;
    Ok(Arc::new(fenetre))
}

/// Position du curseur sur l'écran donné, en pixels logiques (conversion faite ici).
pub fn position_curseur(ecran: &Ecran) -> Option<Point> {
    systeme::position_curseur(ecran)
}

/// Crée l'icône de barre de menus ou de zone de notification, avec une entrée « Quitter ».
pub fn creer_icone() -> Result<Icone, String> {
    Icone::creer()
}
