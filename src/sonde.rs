//! Journal de la sonde : chaque ligne va au terminal et dans `souriso-sonde.txt`, à côté de
//! l'exécutable, car un lancement par double-clic n'a pas de console.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub const NOM_FICHIER: &str = "souriso-sonde.txt";

fn chemin() -> Option<PathBuf> {
    Some(std::env::current_exe().ok()?.with_file_name(NOM_FICHIER))
}

/// Vide le fichier au lancement : il ne contient que la dernière exécution.
pub fn initialiser() {
    if let Some(chemin) = chemin() {
        let _ = std::fs::write(chemin, "");
    }
}

/// Écrit la ligne dans le fichier puis sur la sortie d'erreur. Un échec d'écriture est ignoré :
/// la sonde ne doit jamais empêcher l'application de démarrer ni de signaler son arrêt.
pub fn ecrire(ligne: &str) {
    if let Some(chemin) = chemin() {
        if let Ok(mut fichier) = OpenOptions::new().create(true).append(true).open(chemin) {
            let _ = writeln!(fichier, "{ligne}");
        }
    }
    eprintln!("{ligne}");
}
