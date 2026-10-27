//! Lecture et écriture du fichier des profils. Le dossier vient de la couche plateforme ; ici, un
//! chemin et du JSON. Un fichier absent ou inutilisable ne fait jamais échouer le démarrage : on
//! retombe sur les profils du premier lancement.

use std::io;
use std::path::{Path, PathBuf};

use super::profil::{Profils, VERSION_FORMAT};

pub const NOM_FICHIER: &str = "profils.json";

/// Ce qu'a donné la lecture du fichier.
#[derive(Debug, PartialEq)]
pub struct Chargement {
    pub profils: Profils,
    /// Vrai si le fichier doit être (ré)écrit : premier lancement, ou fichier mis de côté.
    pub a_enregistrer: bool,
    /// À écrire dans la sonde : ce qui s'est passé, une phrase par ligne.
    pub messages: Vec<String>,
}

fn copie_corrompue(chemin: &Path) -> PathBuf {
    chemin.with_extension("json.corrompu")
}

/// Lit les profils. Fichier absent : premier lancement. Fichier illisible, mal formé ou d'une
/// autre version du format : profils du premier lancement, et le fichier est mis de côté sous
/// `profils.json.corrompu` pour que l'utilisateur puisse le récupérer.
pub fn charger(chemin: &Path) -> Chargement {
    let octets = match std::fs::read(chemin) {
        Ok(octets) => octets,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Chargement {
                profils: Profils::default(),
                a_enregistrer: true,
                messages: vec!["premier lancement : profils d'exemple créés".into()],
            };
        }
        Err(e) => {
            return Chargement {
                profils: Profils::default(),
                a_enregistrer: false,
                messages: vec![format!(
                    "lecture des profils impossible ({e}) : valeurs par défaut, fichier laissé tel quel"
                )],
            };
        }
    };
    match analyser(&octets) {
        Ok((profils, corrections)) => Chargement {
            profils,
            a_enregistrer: false,
            messages: corrections
                .into_iter()
                .map(|c| format!("profils corrigés à la lecture : {c}"))
                .collect(),
        },
        Err(raison) => {
            let de_cote = copie_corrompue(chemin);
            let sort = match std::fs::rename(chemin, &de_cote) {
                Ok(()) => format!("fichier conservé sous {}", de_cote.display()),
                Err(e) => format!("fichier non conservé ({e})"),
            };
            Chargement {
                profils: Profils::default(),
                a_enregistrer: true,
                messages: vec![format!(
                    "fichier des profils inutilisable ({raison}) : valeurs par défaut, {sort}"
                )],
            }
        }
    }
}

/// Décode le contenu du fichier. Le numéro de version se lit avant le reste : un fichier d'une
/// autre version n'est pas interprété avec les règles de celle-ci.
fn analyser(octets: &[u8]) -> Result<(Profils, Vec<String>), String> {
    let valeur: serde_json::Value =
        serde_json::from_slice(octets).map_err(|e| format!("JSON invalide : {e}"))?;
    match valeur.get("version").and_then(serde_json::Value::as_u64) {
        Some(v) if v == u64::from(VERSION_FORMAT) => {}
        Some(v) => return Err(format!("version {v} du format non prise en charge")),
        None => return Err("numéro de version absent".into()),
    }
    let mut profils: Profils =
        serde_json::from_value(valeur).map_err(|e| format!("contenu invalide : {e}"))?;
    let corrections = profils.normaliser();
    Ok((profils, corrections))
}

/// Écrit les profils : dans un fichier voisin d'abord, puis renommé, pour qu'une coupure en cours
/// d'écriture ne laisse jamais un fichier tronqué.
pub fn enregistrer(chemin: &Path, profils: &Profils) -> io::Result<()> {
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier)?;
    }
    let contenu = serde_json::to_vec_pretty(profils).map_err(io::Error::other)?;
    let temporaire = chemin.with_extension("json.tmp");
    std::fs::write(&temporaire, contenu)?;
    std::fs::rename(&temporaire, chemin).inspect_err(|_| {
        let _ = std::fs::remove_file(&temporaire);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coeur::config::{raccourci, Adaptateur};

    /// Dossier neuf par essai, supprimé à la fin.
    struct Dossier(PathBuf);

    impl Dossier {
        fn new(nom: &str) -> Self {
            let d = std::env::temp_dir()
                .join(format!("souriso-profils-test-{}-{nom}", std::process::id()));
            let _ = std::fs::remove_dir_all(&d);
            std::fs::create_dir_all(&d).unwrap();
            Dossier(d)
        }

        fn fichier(&self) -> PathBuf {
            self.0.join(NOM_FICHIER)
        }
    }

    impl Drop for Dossier {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn un_fichier_absent_donne_le_premier_lancement() {
        let d = Dossier::new("absent");
        let c = charger(&d.fichier());
        assert_eq!(c.profils, Profils::default());
        assert!(c.a_enregistrer);
    }

    #[test]
    fn enregistrer_puis_charger_redonne_les_memes_profils() {
        let d = Dossier::new("aller-retour");
        let mut p = Profils::default();
        p.creer("Cours").unwrap();
        p.activer(3).unwrap();
        p.adaptateur = Adaptateur::Econome;
        p.definir_raccourci(3, Some(raccourci('K'))).unwrap();
        p.profils[3].effets.anneau.rayon = 61.5;
        enregistrer(&d.fichier(), &p).unwrap();
        let c = charger(&d.fichier());
        assert_eq!(c.profils, p);
        assert!(!c.a_enregistrer);
        assert_eq!(c.messages, Vec::<String>::new());
        assert!(!d.0.join("profils.json.tmp").exists());
    }

    #[test]
    fn enregistrer_cree_le_dossier_et_remplace_le_fichier() {
        let d = Dossier::new("cree");
        let chemin = d.0.join("sous").join(NOM_FICHIER);
        let mut p = Profils::default();
        enregistrer(&chemin, &p).unwrap();
        p.activer(1).unwrap();
        enregistrer(&chemin, &p).unwrap();
        assert_eq!(charger(&chemin).profils.actif, 1);
    }

    #[test]
    fn le_fichier_porte_son_numero_de_version() {
        let d = Dossier::new("version");
        enregistrer(&d.fichier(), &Profils::default()).unwrap();
        let texte = std::fs::read_to_string(d.fichier()).unwrap();
        assert!(texte.contains("\"version\": 1"), "{texte}");
    }

    fn corrompu(nom: &str, contenu: &[u8]) {
        let d = Dossier::new(nom);
        std::fs::write(d.fichier(), contenu).unwrap();
        let c = charger(&d.fichier());
        assert_eq!(c.profils, Profils::default(), "{nom}");
        assert!(c.a_enregistrer, "{nom}");
        assert_eq!(c.messages.len(), 1, "{nom}");
        assert!(
            c.messages[0].contains("inutilisable"),
            "{nom} : {:?}",
            c.messages
        );
        assert_eq!(
            std::fs::read(d.0.join("profils.json.corrompu")).unwrap(),
            contenu,
            "{nom} : copie conservée"
        );
        assert!(!d.fichier().exists(), "{nom}");
    }

    #[test]
    fn un_fichier_corrompu_ramene_aux_valeurs_par_defaut_et_est_conserve() {
        corrompu("vide", b"");
        corrompu("tronque", b"{\"version\": 1, \"profils\": [{\"nom\": \"Cou");
        corrompu("pas-du-json", b"\xff\xfe\x00 ceci n'est pas du JSON");
        corrompu("pas-un-objet", b"[1, 2, 3]");
        corrompu("mauvais-type", b"{\"version\": 1, \"actif\": \"trois\"}");
        corrompu("sans-version", b"{\"profils\": []}");
        corrompu("version-future", b"{\"version\": 2, \"profils\": []}");
        corrompu("version-zero", b"{\"version\": 0}");
    }

    #[test]
    fn un_fichier_partiel_est_complete_par_les_valeurs_par_defaut() {
        let d = Dossier::new("partiel");
        std::fs::write(
            d.fichier(),
            br#"{"version": 1, "profils": [{"nom": "Seul", "effets": {"anneau": {"rayon": 50.0}}}]}"#,
        )
        .unwrap();
        let c = charger(&d.fichier());
        assert_eq!(c.profils.profils.len(), 1);
        let p = &c.profils.profils[0];
        assert_eq!(p.nom, "Seul");
        assert_eq!(p.effets.anneau.rayon, 50.0);
        assert_eq!(
            p.effets.anneau.epaisseur, 4.0,
            "champ absent : valeur par défaut"
        );
        assert_eq!(c.profils.adaptateur, Adaptateur::Automatique);
        assert!(!d.0.join("profils.json.corrompu").exists());
    }

    #[test]
    fn un_fichier_aux_valeurs_absurdes_est_ramene_dans_ses_bornes_sans_planter() {
        let d = Dossier::new("absurde");
        std::fs::write(
            d.fichier(),
            br#"{"version": 1, "actif": 42, "profils": [
                {"nom": "A", "raccourci": {"ctrl": true, "alt": true, "maj": true, "touche": "P"},
                 "effets": {"projecteur": {"intensite": 12.0, "rayon": -3.0}}},
                {"nom": "a"}]}"#,
        )
        .unwrap();
        let c = charger(&d.fichier());
        let p = &c.profils;
        assert_eq!(p.actif, 0);
        assert_eq!(p.profils[0].effets.projecteur.intensite, 1.0);
        assert_eq!(p.profils[0].effets.projecteur.rayon, 0.0);
        assert_eq!(
            p.profils[0].raccourci, None,
            "P est le raccourci du projecteur"
        );
        assert_eq!(p.profils[1].nom, "a (2)");
        assert!(c.messages.len() >= 3, "{:?}", c.messages);
    }

    #[test]
    fn un_chemin_illisible_ne_plante_pas_et_ne_touche_a_rien() {
        let d = Dossier::new("illisible");
        // Un dossier à la place du fichier : la lecture échoue autrement que par « absent ».
        std::fs::create_dir(d.fichier()).unwrap();
        let c = charger(&d.fichier());
        assert_eq!(c.profils, Profils::default());
        assert!(!c.a_enregistrer);
        assert!(d.fichier().is_dir());
    }
}
