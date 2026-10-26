//! Raccourci clavier global (Windows et macOS). Ni droit administrateur ni permission système :
//! `RegisterHotKey` sous Windows, Carbon sous macOS. Les événements sont servis par la boucle
//! d'événements de winit, sur le même fil.

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

use crate::coeur::Raccourci;

pub struct RaccourciGlobal {
    _gestionnaire: GlobalHotKeyManager,
    id: u32,
}

impl RaccourciGlobal {
    pub fn creer(raccourci: &Raccourci) -> Result<Self, String> {
        if !raccourci.touche.is_ascii_alphabetic() {
            return Err(format!(
                "touche « {} » non prise en charge (lettre de A à Z)",
                raccourci.touche
            ));
        }
        let code: Code = format!("Key{}", raccourci.touche.to_ascii_uppercase())
            .parse()
            .map_err(|_| format!("touche « {} » inconnue", raccourci.touche))?;
        let mut modificateurs = Modifiers::empty();
        if raccourci.ctrl {
            modificateurs |= Modifiers::CONTROL;
        }
        if raccourci.alt {
            modificateurs |= Modifiers::ALT;
        }
        if raccourci.maj {
            modificateurs |= Modifiers::SHIFT;
        }
        let touche = HotKey::new(Some(modificateurs), code);
        let gestionnaire = GlobalHotKeyManager::new().map_err(|e| e.to_string())?;
        gestionnaire
            .register(touche)
            .map_err(|e| format!("raccourci refusé par le système (déjà pris ?) : {e}"))?;
        Ok(RaccourciGlobal {
            _gestionnaire: gestionnaire,
            id: touche.id(),
        })
    }

    /// Vrai si la combinaison a été pressée depuis le dernier appel.
    pub fn activation_demandee(&self) -> bool {
        let mut demande = false;
        while let Ok(evenement) = GlobalHotKeyEvent::receiver().try_recv() {
            demande |= evenement.id == self.id && evenement.state == HotKeyState::Pressed;
        }
        demande
    }
}
