//! Journal de la sonde : chaque ligne va au terminal et dans `souriso-sonde.txt`, à côté de
//! l'exécutable, car un lancement par double-clic n'a pas de console.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

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

/// Ligne de bilan écrite à la fermeture, pour calculer à la main la part d'un cœur logique :
/// temps processeur ÷ durée d'exécution. Les images sont celles réellement présentées.
pub fn bilan(duree: Duration, processeur: Option<Duration>, images: u64) -> String {
    let secondes = duree.as_secs_f64();
    let par_seconde = if secondes > 0.0 {
        images as f64 / secondes
    } else {
        0.0
    };
    let processeur = processeur.map_or("indisponible".to_string(), |p| {
        format!("{:.3} s", p.as_secs_f64())
    });
    format!(
        "Bilan : durée d'exécution {secondes:.3} s, temps processeur {processeur}, \
         images dessinées {images} ({par_seconde:.2} images/s en moyenne)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_bilan_donne_duree_processeur_et_cadence() {
        let ligne = bilan(
            Duration::from_secs(60),
            Some(Duration::from_millis(1500)),
            1200,
        );
        assert_eq!(
            ligne,
            "Bilan : durée d'exécution 60.000 s, temps processeur 1.500 s, \
             images dessinées 1200 (20.00 images/s en moyenne)"
        );
    }

    #[test]
    fn le_bilan_survit_a_une_duree_nulle_et_a_un_processeur_inconnu() {
        let ligne = bilan(Duration::ZERO, None, 0);
        assert!(ligne.contains("temps processeur indisponible"));
        assert!(ligne.contains("(0.00 images/s"));
    }
}
