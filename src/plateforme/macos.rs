//! Adaptateur macOS. Lecture du curseur par CoreGraphics : aucune permission requise.

use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::macos::{
    ActivationPolicy, EventLoopBuilderExtMacOS, WindowAttributesExtMacOS,
};
use winit::window::{Window, WindowAttributes, WindowLevel};

use crate::coeur::{Ecran, Point};

/// Pas d'icône dans le Dock ni de menu d'application : seule l'icône de la barre de menus reste.
pub fn creer_boucle() -> Result<EventLoop<()>, String> {
    EventLoop::builder()
        .with_activation_policy(ActivationPolicy::Accessory)
        .with_default_menu(false)
        .build()
        .map_err(|e| e.to_string())
}

pub fn creer_surcouche(event_loop: &ActiveEventLoop, ecran: &Ecran) -> Result<Window, String> {
    let attributs = WindowAttributes::default()
        .with_title("SOURISO")
        .with_decorations(false)
        .with_transparent(true)
        .with_visible(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_has_shadow(false)
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

/// CoreGraphics renvoie des points (pixels logiques), origine en haut à gauche du bureau.
/// L'origine de l'écran fournie par winit est en pixels physiques.
pub fn position_curseur(ecran: &Ecran) -> Option<Point> {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    let evenement = CGEvent::new(source).ok()?;
    let p = evenement.location();
    let e = ecran.echelle;
    Some(Point {
        x: (p.x - ecran.origine.0 as f64 / e) as f32,
        y: (p.y - ecran.origine.1 as f64 / e) as f32,
    })
}

pub use super::rusage::temps_processeur;
