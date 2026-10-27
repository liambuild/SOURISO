//! Couche plateforme : seul endroit qui connaît le système.
//! Le contrat est : boucle d'événements, écrans, surcouche passe-clics, position du curseur,
//! icône de barre de menus ou de zone de notification, raccourci global, temps processeur.

use std::sync::Arc;
use std::time::Duration;

use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;

use crate::coeur::{Ecran, Point, Raccourci};

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

#[cfg(any(target_os = "windows", target_os = "macos"))]
mod raccourci_systeme;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use raccourci_systeme as raccourci;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod raccourci_factice;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use raccourci_factice as raccourci;

pub use raccourci::RaccourciGlobal;

#[cfg(not(target_os = "windows"))]
mod rusage;

/// Crée la boucle d'événements avec les réglages propres au système.
pub fn creer_boucle() -> Result<EventLoop<()>, String> {
    systeme::creer_boucle()
}

/// Décrit tous les écrans dans le repère normalisé de l'application, le principal en premier.
pub fn ecrans(event_loop: &ActiveEventLoop) -> Result<Vec<Ecran>, String> {
    let principal = event_loop.primary_monitor();
    let mut moniteurs: Vec<_> = event_loop.available_monitors().collect();
    if let Some(principal) = principal {
        if let Some(i) = moniteurs.iter().position(|m| *m == principal) {
            moniteurs.swap(0, i);
        }
    }
    if moniteurs.is_empty() {
        return Err("aucun écran détecté".into());
    }
    Ok(moniteurs
        .iter()
        .map(|moniteur| {
            let position = moniteur.position();
            let taille = moniteur.size();
            Ecran {
                origine: (position.x, position.y),
                taille: (taille.width, taille.height),
                echelle: moniteur.scale_factor(),
                rafraichissement_mhz: moniteur.refresh_rate_millihertz(),
            }
        })
        .collect())
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

/// Options de l'instance graphique propres au système (backend, mode de présentation).
/// Le rendu y ajoute seulement les variables d'environnement `WGPU_*`, utiles à la sonde.
#[cfg(target_os = "windows")]
pub use windows::options_instance;

/// Hors Windows, les options par défaut de wgpu conviennent (Metal sous macOS).
#[cfg(not(target_os = "windows"))]
pub fn options_instance(
    affichage: winit::event_loop::OwnedDisplayHandle,
) -> wgpu::InstanceDescriptor {
    wgpu::InstanceDescriptor::new_with_display_handle(Box::new(affichage))
}

/// Position du curseur sur l'écran donné, en pixels logiques (conversion faite ici).
pub fn position_curseur(ecran: &Ecran) -> Option<Point> {
    systeme::position_curseur(ecran)
}

/// Crée l'icône de barre de menus ou de zone de notification, avec une entrée « Quitter ».
pub fn creer_icone() -> Result<Icone, String> {
    Icone::creer()
}

/// Temps processeur du processus depuis son lancement, noyau et utilisateur séparés.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempsProcesseur {
    pub noyau: Duration,
    pub utilisateur: Duration,
}

impl TempsProcesseur {
    pub fn total(&self) -> Duration {
        self.noyau + self.utilisateur
    }
}

/// Temps processeur consommé par le processus (noyau et utilisateur), depuis son lancement.
pub fn temps_processeur() -> Option<TempsProcesseur> {
    systeme::temps_processeur()
}

/// Enregistre le raccourci global. Échoue si le système le refuse (combinaison déjà prise).
pub fn creer_raccourci(raccourci: &Raccourci) -> Result<RaccourciGlobal, String> {
    RaccourciGlobal::creer(raccourci)
}
