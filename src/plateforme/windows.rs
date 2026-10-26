//! Adaptateur Windows. Aucun droit administrateur requis.

use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
use winit::event_loop::{ActiveEventLoop, EventLoop, OwnedDisplayHandle};
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{Window, WindowAttributes, WindowLevel};

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
        .with_skip_taskbar(true)
        .with_undecorated_shadow(false)
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

/// DX12 avec présentation par composition (DirectComposition) : c'est le seul chemin qui offre
/// une surface transparente sous Windows. Le mode par défaut de wgpu (`DxgiFromHwnd`) et Vulkan
/// n'annoncent que l'alpha opaque.
pub fn options_instance(affichage: OwnedDisplayHandle) -> wgpu::InstanceDescriptor {
    let mut options = wgpu::InstanceDescriptor::new_with_display_handle(Box::new(affichage));
    options.backends = wgpu::Backends::DX12;
    options.backend_options.dx12.presentation_system = wgpu::Dx12SwapchainKind::DxgiFromVisual;
    options
}

/// `GetCursorPos` renvoie des pixels physiques du bureau virtuel (processus sensible au DPI,
/// réglé par winit) : on les ramène en pixels logiques de l'écran.
pub fn position_curseur(ecran: &Ecran) -> Option<Point> {
    let mut p = POINT { x: 0, y: 0 };
    // SAFETY : `p` est une structure valide et exclusive pour la durée de l'appel.
    let ok = unsafe { GetCursorPos(&mut p) };
    if ok == 0 {
        return None;
    }
    let e = ecran.echelle as f32;
    Some(Point {
        x: (p.x - ecran.origine.0) as f32 / e,
        y: (p.y - ecran.origine.1) as f32 / e,
    })
}
