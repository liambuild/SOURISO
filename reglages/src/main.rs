//! `souriso-reglages` : la fenêtre de réglages de SOURISO. Une page web embarquée (dossier `ui`),
//! sans réseau, qui parle au processus principal par ses entrée et sortie standard, une ligne de
//! JSON par message. Ce programme ne comprend pas ces messages : il les transmet à la page et
//! reprend ce qu'elle écrit. Il ajoute seulement la position de la fenêtre, que le processus
//! principal utilise pour ne pas assombrir la fenêtre sous le voile du projecteur.
//! Il s'arrête quand la fenêtre se ferme, ou quand son entrée se ferme (le processus principal
//! a disparu) : le moteur web du système part avec lui.

#[cfg(test)]
mod ui_tests;

use std::io::{BufRead, Write};
use std::time::Duration;

use tauri::{Emitter, WebviewWindow, Window, WindowEvent};

/// Nom de l'événement qui porte, vers la page, chaque ligne reçue du processus principal.
const EVENEMENT: &str = "souriso-message";

/// Délai accordé à l'arrêt normal avant d'y mettre fin de force.
const ARRET_FORCE: Duration = Duration::from_secs(2);

/// Écrit une ligne pour le processus principal. S'il n'écoute plus, il n'y a plus de raison de
/// vivre.
fn ecrire(ligne: &str) {
    let mut sortie = std::io::stdout().lock();
    if writeln!(sortie, "{ligne}")
        .and_then(|()| sortie.flush())
        .is_err()
    {
        std::process::exit(0);
    }
}

/// La page envoie sa ligne au processus principal.
#[tauri::command]
fn envoyer(ligne: String) {
    ecrire(&ligne);
}

/// La page est prête (ou le processus principal demande à la ramener devant) : montre la fenêtre.
#[tauri::command]
fn afficher(fenetre: WebviewWindow) {
    let _ = fenetre.unminimize();
    let _ = fenetre.show();
    let _ = fenetre.set_focus();
    signaler_position(&fenetre.as_ref().window());
}

/// Position et taille de la fenêtre, en pixels physiques du bureau, pour le processus principal.
fn signaler_position(fenetre: &Window) {
    let visible = fenetre.is_visible().unwrap_or(false) && !fenetre.is_minimized().unwrap_or(true);
    if let (Ok(p), Ok(t)) = (fenetre.outer_position(), fenetre.outer_size()) {
        ecrire(&format!(
            r#"{{"type":"fenetre","x":{},"y":{},"l":{},"h":{},"visible":{visible}}}"#,
            p.x, p.y, t.width, t.height
        ));
    }
}

fn main() {
    let resultat = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![envoyer, afficher])
        .setup(|app| {
            let app = app.handle().clone();
            std::thread::spawn(move || {
                for ligne in std::io::stdin().lock().lines().map_while(Result::ok) {
                    let _ = app.emit(EVENEMENT, ligne);
                }
                // Plus d'entrée : le processus principal n'est plus là.
                app.exit(0);
                std::thread::sleep(ARRET_FORCE);
                std::process::exit(0);
            });
            Ok(())
        })
        .on_window_event(|fenetre, evenement| match evenement {
            WindowEvent::Moved(_)
            | WindowEvent::Resized(_)
            | WindowEvent::Focused(_)
            | WindowEvent::ScaleFactorChanged { .. } => signaler_position(fenetre),
            WindowEvent::CloseRequested { .. } => {
                ecrire(r#"{"type":"fermee"}"#);
                // La dernière fenêtre fermée met fin au programme ; au besoin, de force.
                std::thread::spawn(|| {
                    std::thread::sleep(ARRET_FORCE);
                    std::process::exit(0);
                });
            }
            _ => {}
        })
        .run(tauri::generate_context!());
    if let Err(e) = resultat {
        eprintln!("souriso-reglages : {e}");
        std::process::exit(2);
    }
}
