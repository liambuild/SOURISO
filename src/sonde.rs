//! Journal de la sonde : chaque ligne va au terminal et dans `souriso-sonde.txt`, à côté de
//! l'exécutable, car un lancement par double-clic n'a pas de console.

use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use crate::plateforme::TempsProcesseur;

pub const NOM_FICHIER: &str = "souriso-sonde.txt";

/// Début (octet) et texte de la ligne de bilan, tant qu'elle est la dernière du fichier.
type LigneBilan = Option<(u64, String)>;

static BILAN: Mutex<LigneBilan> = Mutex::new(None);

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
    ajouter_au_fichier(ligne, false);
    eprintln!("{ligne}");
}

/// Écrit ou remplace la ligne de bilan en cours, dans le fichier seulement : appelée toutes les
/// quelques secondes, elle garde un bilan lisible si le processus est tué.
pub fn bilan_en_cours(ligne: &str) {
    ajouter_au_fichier(ligne, true);
}

/// Écrit le bilan définitif, dans le fichier et sur la sortie d'erreur.
pub fn bilan_final(ligne: &str) {
    ajouter_au_fichier(ligne, true);
    eprintln!("{ligne}");
}

fn ajouter_au_fichier(ligne: &str, est_bilan: bool) {
    let (Some(chemin), Ok(mut etat)) = (chemin(), BILAN.lock()) else {
        return;
    };
    let _ = ajouter(&chemin, &mut etat, ligne, est_bilan);
}

/// Ajoute une ligne à la fin du fichier. Le bilan en cours reste la dernière ligne : il est
/// retiré avant l'ajout d'une autre ligne, puis réécrit derrière elle, ou remplacé par le nouveau.
fn ajouter(
    chemin: &Path,
    etat: &mut LigneBilan,
    ligne: &str,
    est_bilan: bool,
) -> std::io::Result<()> {
    let mut fichier = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(chemin)?;
    let precedent = etat.take();
    let debut = match &precedent {
        Some((debut, _)) => {
            fichier.set_len(*debut)?;
            *debut
        }
        None => fichier.metadata()?.len(),
    };
    fichier.seek(SeekFrom::Start(debut))?;
    writeln!(fichier, "{ligne}")?;
    if est_bilan {
        *etat = Some((debut, ligne.to_string()));
    } else if let Some((_, texte)) = precedent {
        let nouveau_debut = fichier.stream_position()?;
        writeln!(fichier, "{texte}")?;
        *etat = Some((nouveau_debut, texte));
    }
    Ok(())
}

/// Ce que la boucle a fait depuis le lancement, pour situer d'où vient le temps processeur.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Compteurs {
    /// Passages de la boucle d'événements (un par réveil).
    pub reveils: u64,
    /// Appels à la lecture de la position du curseur, tous écrans confondus.
    pub lectures_curseur: u64,
    /// Images réellement présentées, par écran (dans l'ordre des écrans retenus).
    pub images_par_ecran: Vec<u64>,
    /// Acquisitions d'image qui n'ont rien donné.
    pub echecs_acquisition: u64,
    /// Reconfigurations de la surface, sur surface périmée ou perdue.
    pub reconfigurations: u64,
}

/// Taux de rafraîchissement d'un écran pour la sonde, ou « inconnu » si le système ne le donne pas.
pub fn rafraichissement(millihertz: Option<u32>) -> String {
    millihertz.map_or("inconnu".to_string(), |m| {
        format!("{:.3} Hz", f64::from(m) / 1000.0)
    })
}

/// Ligne de bilan, pour calculer à la main la part d'un cœur logique : temps processeur ÷ durée
/// d'exécution. Les images sont celles réellement présentées.
pub fn bilan(
    duree: Duration,
    processeur: Option<TempsProcesseur>,
    compteurs: &Compteurs,
) -> String {
    let secondes = duree.as_secs_f64();
    let images: u64 = compteurs.images_par_ecran.iter().sum();
    let par_seconde = if secondes > 0.0 {
        images as f64 / secondes
    } else {
        0.0
    };
    let processeur = processeur.map_or("indisponible".to_string(), |p| {
        format!(
            "{:.3} s (noyau {:.3} s, utilisateur {:.3} s)",
            p.total().as_secs_f64(),
            p.noyau.as_secs_f64(),
            p.utilisateur.as_secs_f64()
        )
    });
    format!(
        "Bilan : durée d'exécution {secondes:.3} s, temps processeur {processeur}, \
         images dessinées {images} ({par_seconde:.2} images/s en moyenne), \
         images par écran {:?}, réveils de la boucle {}, lectures du curseur {}, \
         échecs d'acquisition {}, reconfigurations de surface {}",
        compteurs.images_par_ecran,
        compteurs.reveils,
        compteurs.lectures_curseur,
        compteurs.echecs_acquisition,
        compteurs.reconfigurations
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_bilan_donne_duree_processeur_cadence_et_compteurs() {
        let compteurs = Compteurs {
            reveils: 7500,
            lectures_curseur: 22500,
            images_par_ecran: vec![800, 400, 0],
            echecs_acquisition: 3,
            reconfigurations: 1,
        };
        let ligne = bilan(
            Duration::from_secs(60),
            Some(TempsProcesseur {
                noyau: Duration::from_millis(500),
                utilisateur: Duration::from_millis(1000),
            }),
            &compteurs,
        );
        assert_eq!(
            ligne,
            "Bilan : durée d'exécution 60.000 s, temps processeur 1.500 s \
             (noyau 0.500 s, utilisateur 1.000 s), images dessinées 1200 \
             (20.00 images/s en moyenne), images par écran [800, 400, 0], \
             réveils de la boucle 7500, lectures du curseur 22500, \
             échecs d'acquisition 3, reconfigurations de surface 1"
        );
    }

    #[test]
    fn le_rafraichissement_est_en_hertz_ou_inconnu() {
        assert_eq!(rafraichissement(Some(59_940)), "59.940 Hz");
        assert_eq!(rafraichissement(Some(144_000)), "144.000 Hz");
        assert_eq!(rafraichissement(None), "inconnu");
    }

    #[test]
    fn le_bilan_survit_a_une_duree_nulle_et_a_un_processeur_inconnu() {
        let ligne = bilan(Duration::ZERO, None, &Compteurs::default());
        assert!(ligne.contains("temps processeur indisponible"));
        assert!(ligne.contains("(0.00 images/s"));
    }

    fn fichier_temporaire(nom: &str) -> PathBuf {
        let chemin = std::env::temp_dir().join(format!(
            "souriso-sonde-test-{}-{nom}.txt",
            std::process::id()
        ));
        std::fs::write(&chemin, "").unwrap();
        chemin
    }

    #[test]
    fn un_nouveau_bilan_remplace_le_precedent() {
        let chemin = fichier_temporaire("remplace");
        let mut etat = None;
        ajouter(&chemin, &mut etat, "début", false).unwrap();
        ajouter(&chemin, &mut etat, "bilan 1", true).unwrap();
        ajouter(&chemin, &mut etat, "bilan 2, plus long", true).unwrap();
        ajouter(&chemin, &mut etat, "bilan 3", true).unwrap();
        let contenu = std::fs::read_to_string(&chemin).unwrap();
        std::fs::remove_file(&chemin).unwrap();
        assert_eq!(contenu, "début\nbilan 3\n");
    }

    #[test]
    fn une_ligne_ajoutee_apres_un_bilan_passe_devant_lui() {
        let chemin = fichier_temporaire("devant");
        let mut etat = None;
        ajouter(&chemin, &mut etat, "début", false).unwrap();
        ajouter(&chemin, &mut etat, "bilan 1", true).unwrap();
        ajouter(&chemin, &mut etat, "Projecteur coupé", false).unwrap();
        ajouter(&chemin, &mut etat, "bilan 2", true).unwrap();
        ajouter(&chemin, &mut etat, "Projecteur activé", false).unwrap();
        let contenu = std::fs::read_to_string(&chemin).unwrap();
        std::fs::remove_file(&chemin).unwrap();
        assert_eq!(
            contenu,
            "début\nProjecteur coupé\nProjecteur activé\nbilan 2\n"
        );
    }
}
