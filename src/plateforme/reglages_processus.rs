//! Le processus `souriso-reglages` : lancé à l'ouverture de la fenêtre de réglages, fermé avec
//! elle. Les messages passent par ses entrée et sortie standard (une ligne de JSON chacun), sans
//! réseau. Le processus principal n'embarque aucun moteur web : tout ce qui touche à la page vit
//! dans l'autre exécutable, livré à côté de celui-ci.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use super::systeme;

/// Nom de l'exécutable des réglages, sans son suffixe système.
const NOM_EXECUTABLE: &str = "souriso-reglages";

/// Temps laissé au processus pour se terminer de lui-même quand son entrée se ferme, avant de le
/// forcer.
const DELAI_ARRET: Duration = Duration::from_millis(1000);

/// Pourquoi la fenêtre ne s'ouvre pas.
#[derive(Debug, PartialEq)]
pub enum ErreurReglages {
    /// Le moteur web du système manque (WebView2 sous Windows) : une boîte native l'a dit.
    MoteurAbsent,
    /// L'exécutable n'est pas à côté de celui-ci.
    Introuvable(PathBuf),
    Lancement(String),
}

impl std::fmt::Display for ErreurReglages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErreurReglages::MoteurAbsent => write!(f, "moteur web du système absent (WebView2)"),
            ErreurReglages::Introuvable(c) => write!(f, "{} introuvable", c.display()),
            ErreurReglages::Lancement(e) => write!(f, "lancement impossible : {e}"),
        }
    }
}

impl ErreurReglages {
    /// Ce qu'on dit à l'utilisateur, avec la marche à suivre (README, section « Fenêtre de
    /// réglages »). SOURISO continue de fonctionner dans tous les cas.
    fn message_utilisateur(&self) -> String {
        let suite = "\n\nSOURISO continue de fonctionner avec le dernier profil utilisé. \
                     La marche à suivre est dans le fichier README, section « Fenêtre de réglages ».";
        match self {
            ErreurReglages::MoteurAbsent => format!(
                "La fenêtre de réglages a besoin du composant Microsoft Edge WebView2, qui n'est \
                 pas installé sur cet ordinateur.{suite}"
            ),
            ErreurReglages::Introuvable(_) => format!(
                "La fenêtre de réglages n'a pas pu s'ouvrir : le fichier {NOM_EXECUTABLE} doit \
                 se trouver à côté de SOURISO.{suite}"
            ),
            ErreurReglages::Lancement(raison) => {
                format!("La fenêtre de réglages n'a pas pu s'ouvrir ({raison}).{suite}")
            }
        }
    }
}

pub struct FenetreReglages {
    enfant: Child,
    entree: Option<ChildStdin>,
    recu: Receiver<String>,
}

/// Où se trouve l'exécutable des réglages : dans le dossier de celui-ci.
fn chemin_executable() -> Result<PathBuf, ErreurReglages> {
    let moi = std::env::current_exe().map_err(|e| ErreurReglages::Lancement(e.to_string()))?;
    Ok(moi.with_file_name(format!("{NOM_EXECUTABLE}{}", std::env::consts::EXE_SUFFIX)))
}

impl FenetreReglages {
    /// Ouvre la fenêtre. Si elle ne peut pas s'ouvrir, le dit dans une boîte native (sans
    /// bloquer) et renvoie la raison : l'application continue avec le profil en cours.
    pub fn ouvrir() -> Result<Self, ErreurReglages> {
        let resultat = Self::essayer();
        if let Err(e) = &resultat {
            systeme::afficher_boite(&e.message_utilisateur());
        }
        resultat
    }

    fn essayer() -> Result<Self, ErreurReglages> {
        if !systeme::moteur_web_disponible() {
            return Err(ErreurReglages::MoteurAbsent);
        }
        let chemin = chemin_executable()?;
        if !chemin.is_file() {
            return Err(ErreurReglages::Introuvable(chemin));
        }
        let mut commande = Command::new(chemin);
        systeme::preparer_enfant(&mut commande);
        Self::lancer(commande)
    }

    fn lancer(mut commande: Command) -> Result<Self, ErreurReglages> {
        let mut enfant = commande
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| ErreurReglages::Lancement(e.to_string()))?;
        let entree = enfant.stdin.take();
        let sortie = enfant
            .stdout
            .take()
            .ok_or_else(|| ErreurReglages::Lancement("sortie du processus indisponible".into()))?;
        let (envoi, recu) = mpsc::channel();
        // Un fil bloqué sur la lecture : il ne consomme rien tant que la fenêtre n'écrit pas, et
        // s'arrête à la fermeture du tuyau (fin du processus).
        std::thread::spawn(move || {
            for ligne in BufReader::new(sortie).lines().map_while(Result::ok) {
                if envoi.send(ligne).is_err() {
                    break;
                }
            }
        });
        Ok(FenetreReglages {
            enfant,
            entree,
            recu,
        })
    }

    /// Envoie une ligne à la fenêtre. Faux si le tuyau est fermé.
    pub fn envoyer(&mut self, ligne: &str) -> bool {
        let Some(entree) = self.entree.as_mut() else {
            return false;
        };
        let ecrit = entree
            .write_all(ligne.as_bytes())
            .and_then(|()| entree.write_all(b"\n"))
            .and_then(|()| entree.flush());
        if ecrit.is_err() {
            self.entree = None;
        }
        ecrit.is_ok()
    }

    /// Les lignes reçues depuis le dernier appel.
    pub fn lignes(&self) -> Vec<String> {
        self.recu.try_iter().collect()
    }

    /// Vrai quand le processus s'est terminé (fenêtre fermée, ou plantée).
    pub fn terminee(&mut self) -> bool {
        !matches!(self.enfant.try_wait(), Ok(None))
    }
}

impl Drop for FenetreReglages {
    /// Ferme l'entrée du processus, ce qui lui demande de s'arrêter, puis le force s'il tarde.
    fn drop(&mut self) {
        self.entree = None;
        let limite = Instant::now() + DELAI_ARRET;
        while Instant::now() < limite {
            if !matches!(self.enfant.try_wait(), Ok(None)) {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = self.enfant.kill();
        let _ = self.enfant.wait();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn echo() -> FenetreReglages {
        FenetreReglages::lancer(Command::new("cat")).unwrap()
    }

    fn attendre(f: &FenetreReglages, n: usize) -> Vec<String> {
        let mut lignes = Vec::new();
        let limite = Instant::now() + Duration::from_secs(5);
        while lignes.len() < n && Instant::now() < limite {
            lignes.extend(f.lignes());
            std::thread::sleep(Duration::from_millis(5));
        }
        lignes
    }

    #[test]
    fn les_lignes_vont_et_viennent_par_les_tuyaux() {
        let mut f = echo();
        assert!(f.envoyer(r#"{"type":"pret"}"#));
        assert!(f.envoyer("deuxième ligne"));
        assert_eq!(attendre(&f, 2), [r#"{"type":"pret"}"#, "deuxième ligne"]);
        assert!(!f.terminee());
    }

    #[test]
    fn fermer_l_entree_arrete_le_processus_sans_le_tuer() {
        let mut f = echo();
        let debut = Instant::now();
        drop(f.entree.take());
        while !f.terminee() && debut.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(f.terminee(), "cat s'arrête à la fin de son entrée");
        assert!(!f.envoyer("trop tard"));
    }

    #[test]
    fn un_processus_qui_ignore_la_fin_de_l_entree_est_force() {
        let f = FenetreReglages::lancer({
            let mut c = Command::new("sleep");
            c.arg("60");
            c
        })
        .unwrap();
        let debut = Instant::now();
        drop(f);
        assert!(debut.elapsed() < DELAI_ARRET + Duration::from_secs(3));
    }

    #[test]
    fn chaque_echec_a_un_message_et_renvoie_au_readme() {
        for e in [
            ErreurReglages::MoteurAbsent,
            ErreurReglages::Introuvable(PathBuf::from("x")),
            ErreurReglages::Lancement("refusé".into()),
        ] {
            let m = e.message_utilisateur();
            assert!(
                m.contains("README") && m.contains("continue de fonctionner"),
                "{m}"
            );
        }
        assert!(ErreurReglages::MoteurAbsent
            .message_utilisateur()
            .contains("WebView2"));
    }

    #[test]
    fn un_executable_absent_est_signale() {
        let e = FenetreReglages::lancer(Command::new("/nonexistent/souriso-reglages"));
        assert!(matches!(e, Err(ErreurReglages::Lancement(_))));
    }
}
