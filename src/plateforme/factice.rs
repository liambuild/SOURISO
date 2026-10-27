//! Adaptateur factice : permet de compiler et de tester le cœur sous Linux.
//! Le produit ne cible pas Linux.

use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowLevel};

use crate::coeur::clic::Boutons;
use crate::coeur::{Ecran, Point};

pub fn creer_boucle() -> Result<EventLoop<()>, String> {
    EventLoop::new().map_err(|e| e.to_string())
}

pub fn creer_surcouche(event_loop: &ActiveEventLoop, ecran: &Ecran) -> Result<Window, String> {
    let attributs = WindowAttributes::default()
        .with_title("SOURISO")
        .with_decorations(false)
        .with_transparent(true)
        .with_visible(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_position(winit::dpi::PhysicalPosition::new(
            ecran.origine.0,
            ecran.origine.1,
        ))
        .with_inner_size(winit::dpi::PhysicalSize::new(
            ecran.taille.0,
            ecran.taille.1,
        ));
    event_loop
        .create_window(attributs)
        .map_err(|e| e.to_string())
}

/// Pas d'avis sur la carte qui porte chaque écran : le choix de wgpu s'applique.
pub fn cartes_ecrans(ecrans: &[Ecran]) -> Vec<Option<String>> {
    vec![None; ecrans.len()]
}

/// Rejoue une position fixe au centre de l'écran.
pub fn position_curseur(ecran: &Ecran) -> Option<Point> {
    Some(Point {
        x: ecran.taille.0 as f32 / ecran.echelle as f32 / 2.0,
        y: ecran.taille.1 as f32 / ecran.echelle as f32 / 2.0,
    })
}

/// Aucun bouton enfoncé : le factice ne rejoue pas de clic.
pub fn boutons() -> Option<Boutons> {
    Some(Boutons::default())
}

/// Ne joue rien et le dit.
pub fn jouer_son(_wav: &'static [u8]) -> bool {
    false
}

pub use super::rusage::temps_processeur;

/// Hors Windows et macOS : `$XDG_DATA_HOME/SOURISO`, sinon `~/.local/share/SOURISO`. Sert aux essais.
pub fn dossier_donnees() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|d| !d.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let maison = std::env::var_os("HOME").filter(|d| !d.is_empty())?;
            Some(
                std::path::PathBuf::from(maison)
                    .join(".local")
                    .join("share"),
            )
        })?;
    Some(base.join("SOURISO"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_curseur_factice_est_au_centre() {
        let ecran = Ecran {
            origine: (0, 0),
            taille: (200, 100),
            echelle: 2.0,
            rafraichissement_mhz: None,
        };
        assert_eq!(position_curseur(&ecran), Some(Point { x: 50.0, y: 25.0 }));
    }
}
