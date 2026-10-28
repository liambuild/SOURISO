//! Adaptateur macOS. Lecture du curseur par CoreGraphics : aucune permission requise.

use std::cell::RefCell;

use core_graphics::event::{CGEvent, CGMouseButton};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use objc2::rc::Retained;
use objc2::AnyThread;
use objc2_app_kit::NSSound;
use objc2_foundation::NSData;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::macos::{
    ActivationPolicy, EventLoopBuilderExtMacOS, WindowAttributesExtMacOS,
};
use winit::window::{Window, WindowAttributes, WindowLevel};

use crate::coeur::clic::Boutons;
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

/// Pas d'avis sur la carte qui porte chaque écran : le choix de wgpu s'applique.
pub fn cartes_ecrans(ecrans: &[Ecran]) -> Vec<Option<String>> {
    vec![None; ecrans.len()]
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

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    /// Vrai si le bouton est enfoncé dans l'état demandé. Interrogation de l'état : aucun
    /// moniteur d'événements, donc en principe aucune permission « Surveillance de l'entrée ».
    fn CGEventSourceButtonState(state_id: CGEventSourceStateID, button: CGMouseButton) -> bool;
}

/// Lecture de l'état de la session. Non testé sur macOS : la permutation des boutons (réglage du
/// bouton principal) et l'absence d'invite de permission restent à vérifier en recette.
pub fn boutons() -> Option<Boutons> {
    // SAFETY : fonction CoreGraphics sans pointeur, appelée avec deux constantes valides.
    let (gauche, droit) = unsafe {
        (
            CGEventSourceButtonState(
                CGEventSourceStateID::CombinedSessionState,
                CGMouseButton::Left,
            ),
            CGEventSourceButtonState(
                CGEventSourceStateID::CombinedSessionState,
                CGMouseButton::Right,
            ),
        )
    };
    Some(Boutons { gauche, droit })
}

thread_local! {
    /// Son en cours : `NSSound` s'arrête quand il est libéré, il faut donc le garder jusqu'au
    /// suivant (qui coupe le précédent). La boucle d'événements est sur le fil principal.
    static EN_COURS: RefCell<Option<Retained<NSSound>>> = const { RefCell::new(None) };
}

/// `NSSound` initialisé depuis la mémoire, joué en asynchrone. Les octets sont copiés par
/// `NSData`, au prix d'une copie de quelques dizaines de kilo-octets par clic.
pub fn jouer_son(wav: &'static [u8]) -> bool {
    let donnees = NSData::with_bytes(wav);
    let Some(son) = NSSound::initWithData(NSSound::alloc(), &donnees) else {
        return false;
    };
    EN_COURS.with(|en_cours| {
        let mut en_cours = en_cours.borrow_mut();
        if let Some(precedent) = en_cours.take() {
            precedent.stop();
        }
        let lance = son.play();
        *en_cours = Some(son);
        lance
    })
}

pub use super::rusage::temps_processeur;

/// `~/Library/Application Support/SOURISO`.
pub fn dossier_donnees() -> Option<std::path::PathBuf> {
    let maison = std::env::var_os("HOME").filter(|d| !d.is_empty())?;
    Some(
        std::path::PathBuf::from(maison)
            .join("Library")
            .join("Application Support")
            .join("SOURISO"),
    )
}

/// WKWebView fait partie du système : le moteur web est toujours là.
pub fn moteur_web_disponible() -> bool {
    true
}

/// Boîte de dialogue du système, par `osascript` (aucune permission : il ne pilote aucune autre
/// application). Dans un fil, pour ne pas arrêter la lecture du curseur. Le texte ne contient ni
/// guillemet ni barre oblique inverse : il est écrit par ce programme.
pub fn afficher_boite(texte: &str) {
    let script = format!(
        "display dialog \"{}\" with title \"SOURISO\" buttons {{\"OK\"}} default button 1 with icon note",
        texte.replace(['"', '\\'], "'").replace('\n', "\" & return & \"")
    );
    std::thread::spawn(move || {
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .status();
    });
}

/// Rien à régler pour lancer le processus des réglages.
pub fn preparer_enfant(_commande: &mut std::process::Command) {}
