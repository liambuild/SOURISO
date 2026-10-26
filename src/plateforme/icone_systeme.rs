//! Icône de zone de notification (Windows) ou de barre de menus (macOS).

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct Icone {
    _icone: TrayIcon,
    quitter_id: tray_icon::menu::MenuId,
}

impl Icone {
    pub fn creer() -> Result<Self, String> {
        let quitter = MenuItem::new("Quitter SOURISO", true, None);
        let quitter_id = quitter.id().clone();
        let menu = Menu::new();
        menu.append(&quitter).map_err(|e| e.to_string())?;
        let icone = TrayIconBuilder::new()
            .with_tooltip("SOURISO")
            .with_menu(Box::new(menu))
            .with_icon(dessiner_icone()?)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Icone {
            _icone: icone,
            quitter_id,
        })
    }

    /// Vrai dès que l'entrée « Quitter » a été choisie.
    pub fn quitter_demande(&self) -> bool {
        let mut demande = false;
        while let Ok(evenement) = MenuEvent::receiver().try_recv() {
            demande |= evenement.id == self.quitter_id;
        }
        demande
    }
}

/// Anneau jaune sur fond transparent, 32 × 32, dessiné en code pour éviter tout fichier binaire.
fn dessiner_icone() -> Result<Icon, String> {
    const N: u32 = 32;
    let mut rgba = Vec::with_capacity((N * N * 4) as usize);
    for y in 0..N {
        for x in 0..N {
            let d = (((x as f32 + 0.5) - 16.0).powi(2) + ((y as f32 + 0.5) - 16.0).powi(2)).sqrt();
            let a = (1.0 - ((d - 11.0).abs() - 2.0).clamp(0.0, 1.0)) * 255.0;
            rgba.extend_from_slice(&[255, 200, 25, a as u8]);
        }
    }
    Icon::from_rgba(rgba, N, N).map_err(|e| e.to_string())
}
