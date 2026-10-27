//! Raccourci clavier global (Windows et macOS). Ni droit administrateur ni permission système :
//! `RegisterHotKey` sous Windows, Carbon sous macOS. Les événements sont servis par la boucle
//! d'événements de winit, sur le même fil.

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

use crate::coeur::Raccourci;

pub struct RaccourciGlobal {
    _gestionnaire: GlobalHotKeyManager,
    /// Identifiant de chaque raccourci demandé, dans l'ordre de la demande ; `None` s'il est refusé.
    ids: Vec<Option<u32>>,
    /// Raccourcis refusés : indice dans la demande et raison.
    pub refus: Vec<(usize, String)>,
}

fn touche(raccourci: &Raccourci) -> Result<HotKey, String> {
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
    Ok(HotKey::new(Some(modificateurs), code))
}

impl RaccourciGlobal {
    /// Un seul gestionnaire pour tous les raccourcis : les événements arrivent sur un canal
    /// commun, qu'un second gestionnaire viderait à la place du premier.
    pub fn creer(raccourcis: &[Raccourci]) -> Result<Self, String> {
        let gestionnaire = GlobalHotKeyManager::new().map_err(|e| e.to_string())?;
        let mut ids = Vec::new();
        let mut refus = Vec::new();
        for (indice, raccourci) in raccourcis.iter().enumerate() {
            let enregistre = touche(raccourci).and_then(|t| {
                gestionnaire
                    .register(t)
                    .map(|()| t.id())
                    .map_err(|e| format!("raccourci refusé par le système (déjà pris ?) : {e}"))
            });
            match enregistre {
                Ok(id) => ids.push(Some(id)),
                Err(e) => {
                    ids.push(None);
                    refus.push((indice, e));
                }
            }
        }
        Ok(RaccourciGlobal {
            _gestionnaire: gestionnaire,
            ids,
            refus,
        })
    }

    /// Indices (dans la demande) des raccourcis pressés depuis le dernier appel.
    pub fn activations_demandees(&self) -> Vec<usize> {
        let mut demandes = Vec::new();
        while let Ok(evenement) = GlobalHotKeyEvent::receiver().try_recv() {
            if evenement.state != HotKeyState::Pressed {
                continue;
            }
            if let Some(indice) = self.ids.iter().position(|id| *id == Some(evenement.id)) {
                if !demandes.contains(&indice) {
                    demandes.push(indice);
                }
            }
        }
        demandes
    }
}
