//! Couche plateforme : seul endroit qui connaît le système.
//! Le contrat est : boucle d'événements, écrans, surcouche passe-clics, position du curseur,
//! icône de barre de menus ou de zone de notification, raccourcis globaux, état des boutons de la
//! souris, lecture d'un son, temps processeur, lancement de la fenêtre de réglages (moteur web
//! du système compris).

use std::sync::Arc;
use std::time::Duration;

use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;

use crate::coeur::clic::Boutons;
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

/// Ce que l'utilisateur a demandé par l'icône depuis la dernière lecture.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EvenementsIcone {
    pub quitter: bool,
    pub reglages: bool,
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
mod raccourci_systeme;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use raccourci_systeme as raccourci;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod raccourci_factice;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use raccourci_factice as raccourci;

pub use raccourci::RaccourciGlobal;

mod reglages_processus;
pub use reglages_processus::FenetreReglages;

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

/// Nom de la carte graphique qui porte chaque écran, dans l'ordre de `ecrans`. `None` pour un
/// écran dont le système ne donne pas la carte ; macOS et l'adaptateur factice n'en donnent aucune.
pub fn cartes_ecrans(ecrans: &[Ecran]) -> Vec<Option<String>> {
    systeme::cartes_ecrans(ecrans)
}

/// Position du curseur sur l'écran donné, en pixels logiques (conversion faite ici).
pub fn position_curseur(ecran: &Ecran) -> Option<Point> {
    systeme::position_curseur(ecran)
}

/// État des boutons de la souris, tel que l'utilisateur le voit : si le système permute les
/// boutons, la conversion est faite ici. `None` si le système ne répond pas.
pub fn boutons() -> Option<Boutons> {
    systeme::boutons()
}

/// Lance la lecture d'un fichier WAV en mémoire, sans attendre sa fin. Un son en cours peut être
/// coupé par le suivant. Renvoie faux si le système a refusé. Les octets restent valides pendant
/// toute la lecture, d'où `'static`.
pub fn jouer_son(wav: &'static [u8]) -> bool {
    systeme::jouer_son(wav)
}

/// Dossier de données de l'utilisateur pour cette application (créé à l'écriture, pas ici) :
/// `%APPDATA%\SOURISO` sous Windows, `~/Library/Application Support/SOURISO` sous macOS. `None`
/// si le système n'en donne pas.
pub fn dossier_donnees() -> Option<std::path::PathBuf> {
    systeme::dossier_donnees()
}

/// Crée l'icône de barre de menus ou de zone de notification, avec les entrées « Réglages… » et
/// « Quitter ».
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

/// Enregistre les raccourcis globaux. Un raccourci que le système refuse (combinaison déjà prise)
/// est signalé par `RaccourciGlobal::refus` et n'empêche pas les autres.
pub fn creer_raccourcis(raccourcis: &[Raccourci]) -> Result<RaccourciGlobal, String> {
    RaccourciGlobal::creer(raccourcis)
}

/// Rend au système toutes les combinaisons de raccourcis globaux, le temps qu'une combinaison soit
/// saisie dans la fenêtre de réglages : sans cela, le système la détournerait vers ce processus
/// au lieu de la page. Les rétablir, c'est appeler à nouveau `creer_raccourcis`.
pub fn suspendre_raccourcis(raccourci: &mut Option<RaccourciGlobal>) {
    *raccourci = None;
}
