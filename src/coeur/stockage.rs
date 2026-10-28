//! Lecture et écriture du fichier des profils. Le dossier vient de la couche plateforme ; ici, un
//! chemin et du JSON. Un fichier absent ou inutilisable ne fait jamais échouer le démarrage : on
//! retombe sur les profils du premier lancement.

use std::io;
use std::path::{Path, PathBuf};

use super::profil::{Profils, VERSION_FORMAT, VERSION_MINIMALE};

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

/// Copie du fichier d'une version plus ancienne, gardée avant qu'il soit réécrit au format actuel.
fn copie_ancienne(chemin: &Path, version: u32) -> PathBuf {
    chemin.with_extension(format!("json.v{version}"))
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
        Ok((profils, version, corrections)) => {
            let mut messages = Vec::new();
            let migre = version != VERSION_FORMAT;
            if migre {
                let de_cote = copie_ancienne(chemin, version);
                let sort = match std::fs::write(&de_cote, &octets) {
                    Ok(()) => format!("copie conservée sous {}", de_cote.display()),
                    Err(e) => format!("copie non conservée ({e})"),
                };
                messages.push(format!(
                    "fichier des profils migré de la version {version} à la version \
                     {VERSION_FORMAT} (tous les effets actifs) : {sort}"
                ));
            }
            messages.extend(
                corrections
                    .into_iter()
                    .map(|c| format!("profils corrigés à la lecture : {c}")),
            );
            Chargement {
                profils,
                a_enregistrer: migre,
                messages,
            }
        }
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
/// version inconnue n'est pas interprété avec les règles de celle-ci. Une version plus ancienne
/// (de `VERSION_MINIMALE` à la courante) est lue telle quelle : les champs qu'elle n'a pas
/// prennent leur valeur par défaut (pour la 1, tous les effets actifs). Renvoie la version lue.
fn analyser(octets: &[u8]) -> Result<(Profils, u32, Vec<String>), String> {
    let valeur: serde_json::Value =
        serde_json::from_slice(octets).map_err(|e| format!("JSON invalide : {e}"))?;
    let version = match valeur.get("version").and_then(serde_json::Value::as_u64) {
        Some(v) if (VERSION_MINIMALE..=VERSION_FORMAT).contains(&(v as u32)) && v < 1 << 32 => {
            v as u32
        }
        Some(v) => return Err(format!("version {v} du format non prise en charge")),
        None => return Err("numéro de version absent".into()),
    };
    let mut profils: Profils =
        serde_json::from_value(valeur).map_err(|e| format!("contenu invalide : {e}"))?;
    let corrections = profils.normaliser();
    Ok((profils, version, corrections))
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
    fn chaque_choix_de_carte_est_ecrit_puis_relu_tel_quel() {
        for (choix, ecrit) in [
            (Adaptateur::Automatique, "automatique"),
            (Adaptateur::Econome, "econome"),
            (Adaptateur::Performant, "performant"),
        ] {
            let d = Dossier::new(&format!("carte-{ecrit}"));
            // On part d'un autre choix, pour qu'une relecture qui retomberait sur une valeur
            // fixe ou sur la valeur par défaut soit détectée.
            let mut p = Profils {
                adaptateur: if choix == Adaptateur::Performant {
                    Adaptateur::Econome
                } else {
                    Adaptateur::Performant
                },
                ..Profils::default()
            };
            enregistrer(&d.fichier(), &p).unwrap();
            p.adaptateur = choix;
            enregistrer(&d.fichier(), &p).unwrap();
            let texte = std::fs::read_to_string(d.fichier()).unwrap();
            assert!(
                texte.contains(&format!("\"adaptateur\": \"{ecrit}\"")),
                "le fichier doit porter le choix {ecrit} : {texte}"
            );
            assert_eq!(charger(&d.fichier()).profils.adaptateur, choix);
        }
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
        assert!(texte.contains("\"version\": 2"), "{texte}");
        assert!(texte.contains("\"actifs\""), "{texte}");
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
        corrompu("version-future", b"{\"version\": 3, \"profils\": []}");
        corrompu("version-zero", b"{\"version\": 0}");
    }

    #[test]
    fn un_fichier_de_version_1_est_migre_avec_tous_les_effets_actifs() {
        let d = Dossier::new("migration");
        let ancien = br#"{"version": 1, "actif": 1, "adaptateur": "econome", "profils": [
            {"nom": "Defaut"},
            {"nom": "Cours", "raccourci": {"ctrl": true, "alt": true, "maj": true, "touche": "K"},
             "effets": {"anneau": {"rayon": 50.0}}}]}"#;
        std::fs::write(d.fichier(), ancien).unwrap();
        let c = charger(&d.fichier());
        assert!(c.a_enregistrer, "à réécrire au format actuel");
        assert_eq!(c.messages.len(), 1, "{:?}", c.messages);
        assert!(
            c.messages[0].contains("migré de la version 1"),
            "{:?}",
            c.messages
        );
        let p = &c.profils;
        assert_eq!(
            (p.version, p.actif, p.profils.len()),
            (VERSION_FORMAT, 1, 2)
        );
        assert_eq!(p.adaptateur, Adaptateur::Econome);
        assert!(p.profils.iter().all(|x| x.actifs == Default::default()));
        assert_eq!(p.profils[1].effets.anneau.rayon, 50.0, "réglages conservés");
        assert_eq!(std::fs::read(d.0.join("profils.json.v1")).unwrap(), ancien);
        assert!(
            d.fichier().exists(),
            "le fichier d'origine reste jusqu'à la réécriture"
        );
        enregistrer(&d.fichier(), &c.profils).unwrap();
        let relu = charger(&d.fichier());
        assert_eq!(relu.profils, c.profils);
        assert!(
            !relu.a_enregistrer && relu.messages.is_empty(),
            "plus de migration"
        );
    }

    #[test]
    fn l_activation_enregistree_dans_un_profil_survit_a_un_aller_retour() {
        let d = Dossier::new("actifs");
        let mut p = Profils::default();
        p.profils[2].actifs.son = false;
        p.profils[1].actifs.projecteur = false;
        enregistrer(&d.fichier(), &p).unwrap();
        let c = charger(&d.fichier());
        assert_eq!(c.profils, p);
        assert!(!c.profils.profils[2].actifs.son && c.profils.profils[2].actifs.anneau);
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
