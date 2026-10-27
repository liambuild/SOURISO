//! Adaptateur Windows. Aucun droit administrateur requis.

use std::time::Duration;

use windows_sys::Win32::Foundation::{FILETIME, POINT};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW,
    DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_MIRRORING_DRIVER, DM_POSITION,
    ENUM_CURRENT_SETTINGS,
};
use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetSystemMetrics, SM_SWAPBUTTON};
use winit::event_loop::{ActiveEventLoop, EventLoop, OwnedDisplayHandle};
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{Window, WindowAttributes, WindowLevel};

use super::TempsProcesseur;
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

/// Carte graphique qui porte chaque écran, dans l'ordre de `ecrans`. `EnumDisplayDevicesW` liste
/// les sorties du bureau avec le nom de la carte qui les porte ; on rapproche chaque sortie de
/// l'écran de même position (pixels physiques, comme winit). `None` pour un écran introuvable.
pub fn cartes_ecrans(ecrans: &[Ecran]) -> Vec<Option<String>> {
    let mut sorties: Vec<((i32, i32), String)> = Vec::new();
    for indice in 0.. {
        let mut sortie = DISPLAY_DEVICEW {
            cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        // SAFETY : `sortie` est valide et exclusive, `cb` est renseigné ; le premier argument
        // nul demande les sorties de la machine et non les moniteurs d'une sortie.
        if unsafe { EnumDisplayDevicesW(std::ptr::null(), indice, &mut sortie, 0) } == 0 {
            break;
        }
        let etat = sortie.StateFlags;
        if etat & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP == 0
            || etat & DISPLAY_DEVICE_MIRRORING_DRIVER != 0
        {
            continue;
        }
        let mut mode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        // SAFETY : `DeviceName` est terminé par un zéro et `mode` est valide et exclusif.
        let ok = unsafe {
            EnumDisplaySettingsW(sortie.DeviceName.as_ptr(), ENUM_CURRENT_SETTINGS, &mut mode)
        };
        if ok == 0 || mode.dmFields & DM_POSITION == 0 {
            continue;
        }
        // SAFETY : `DM_POSITION` est posé, c'est donc la variante « affichage » de l'union.
        let position = unsafe { mode.Anonymous1.Anonymous2.dmPosition };
        let carte = String::from_utf16_lossy(&sortie.DeviceString);
        sorties.push((
            (position.x, position.y),
            carte.trim_end_matches('\0').to_string(),
        ));
    }
    ecrans
        .iter()
        .map(|ecran| {
            sorties
                .iter()
                .find(|(position, _)| *position == ecran.origine)
                .map(|(_, carte)| carte.clone())
        })
        .collect()
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

/// `GetAsyncKeyState` lit les boutons physiques : `VK_LBUTTON` est le bouton physique de gauche,
/// même quand l'utilisateur a permuté les boutons. Avec `SM_SWAPBUTTON`, le bouton physique de
/// gauche fait le clic droit logique ; on rétablit ici le sens que l'utilisateur voit à l'écran.
pub fn boutons() -> Option<Boutons> {
    // SAFETY : appels sans pointeur ni état partagé.
    let (physique_gauche, physique_droit, permutes) = unsafe {
        (
            GetAsyncKeyState(i32::from(VK_LBUTTON)),
            GetAsyncKeyState(i32::from(VK_RBUTTON)),
            GetSystemMetrics(SM_SWAPBUTTON) != 0,
        )
    };
    // Le bit de poids fort (valeur négative) indique le bouton enfoncé.
    let (gauche, droit) = (physique_gauche < 0, physique_droit < 0);
    Some(if permutes {
        Boutons {
            gauche: droit,
            droit: gauche,
        }
    } else {
        Boutons { gauche, droit }
    })
}

/// `PlaySoundW` depuis la mémoire, en asynchrone : le fil de la boucle n'attend pas la fin du
/// son, et un nouveau son coupe le précédent. `SND_NODEFAULT` évite le bip système en cas d'échec.
pub fn jouer_son(wav: &'static [u8]) -> bool {
    // SAFETY : avec `SND_MEMORY`, le premier argument pointe sur le fichier WAV en mémoire ; il
    // est `'static`, donc valide pendant toute la lecture asynchrone.
    unsafe {
        PlaySoundW(
            wav.as_ptr().cast(),
            std::ptr::null_mut(),
            SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
        ) != 0
    }
}

/// Temps noyau et utilisateur du processus, séparés. `FILETIME` compte des intervalles de 100 ns.
pub fn temps_processeur() -> Option<TempsProcesseur> {
    let nul = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let (mut creation, mut fin, mut noyau, mut utilisateur) = (nul, nul, nul, nul);
    // SAFETY : le pseudo-descripteur du processus courant est toujours valide et les quatre
    // structures sont valides et exclusives pour la durée de l'appel.
    let ok = unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &mut creation,
            &mut fin,
            &mut noyau,
            &mut utilisateur,
        )
    };
    if ok == 0 {
        return None;
    }
    let duree = |t: FILETIME| {
        Duration::from_nanos((((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64) * 100)
    };
    Some(TempsProcesseur {
        noyau: duree(noyau),
        utilisateur: duree(utilisateur),
    })
}

/// `%APPDATA%\SOURISO` : le dossier de données itinérant de l'utilisateur, sans droit particulier.
pub fn dossier_donnees() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("APPDATA").filter(|d| !d.is_empty())?;
    Some(std::path::PathBuf::from(base).join("SOURISO"))
}
