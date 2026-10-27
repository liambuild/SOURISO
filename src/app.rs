//! Boucle principale : une surcouche par écran, les effets qui suivent le curseur sur l'écran où
//! il se trouve, un raccourci global par effet pour l'activer ou le couper, des profils de
//! réglages (un raccourci par profil, un pour le suivant), les clics lus dans la même boucle que
//! le curseur.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

use crate::coeur::activation::{Activations, ANNEAU, ONDULATION, PROJECTEUR, SON, TRAINEE};
use crate::coeur::clic::{Bouton, Detecteur};
use crate::coeur::config::SonClic;
use crate::coeur::profil::Profils;
use crate::coeur::son::{avec_volume, Son};
use crate::coeur::stockage::{self, Chargement};
use crate::coeur::{Config, Ecran, Point, Raccourci};
use crate::plateforme::{self, Icone, RaccourciGlobal};
use crate::rendu::{Gpu, Rendu};
use crate::sonde::{self, Compteurs};

/// Cadence d'interrogation du curseur (environ 120 Hz). Le dessin n'a lieu que si l'image change.
const PERIODE: Duration = Duration::from_millis(8);

/// Intervalle entre deux écritures du bilan en cours dans la sonde.
const PERIODE_BILAN: Duration = Duration::from_secs(10);

/// Effets activables au clavier, dans l'ordre des raccourcis de la configuration.
const ACTIVABLES: [(&str, &str); 5] = [
    (PROJECTEUR, "Projecteur"),
    (ANNEAU, "Anneau"),
    (TRAINEE, "Traînée"),
    (ONDULATION, "Ondulation"),
    (SON, "Son"),
];

/// Délai minimal entre deux bascules : une touche maintenue répète l'événement du raccourci.
const ANTI_REPETITION: Duration = Duration::from_millis(400);

/// Ce que déclenche un raccourci global, dans l'ordre où ils sont demandés au système.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cible {
    /// Active ou coupe l'effet `ACTIVABLES[i]`.
    Effet(usize),
    /// Passe au profil suivant.
    Suivant,
    /// Passe au profil `i` et réactive les effets coupés.
    Profil(usize),
}

/// Son de clic mis au volume réglé. La lecture asynchrone du système lit directement ces octets
/// après le retour de l'appel : ils sont gardés pour toute l'exécution (fuite volontaire, une par
/// couple son et volume rencontré, quelques dizaines de kilo-octets chacune).
fn son_pour(sons: &mut HashMap<(Son, u32), &'static [u8]>, reglage: SonClic) -> &'static [u8] {
    let r = reglage.borne();
    sons.entry((r.choix, r.volume.to_bits()))
        .or_insert_with(|| Box::leak(avec_volume(r.choix.wav(), r.volume).into_boxed_slice()))
}

struct Surcouche {
    fenetre: Arc<Window>,
    ecran: Ecran,
    rendu: Rendu,
    /// Position du curseur sur cet écran, `None` s'il est ailleurs.
    curseur: Option<Point>,
    /// Vrai tant que l'image affichée ne correspond pas à l'état courant.
    a_redessiner: bool,
    /// Images réellement présentées sur cet écran, pour le bilan de la sonde.
    images: u64,
}

pub struct App {
    /// Raccourcis des effets et réglages du profil actif.
    config: Config,
    profils: Profils,
    /// Fichier des profils ; `None` si le système ne donne pas de dossier de données.
    chemin_profils: Option<PathBuf>,
    cibles: Vec<Cible>,
    sons: HashMap<(Son, u32), &'static [u8]>,
    derniere_bascule_profil: Option<Instant>,
    changements_profil: u64,
    activations: Activations,
    surcouches: Vec<Surcouche>,
    icone: Option<Icone>,
    raccourci: Option<RaccourciGlobal>,
    /// Dernière bascule de chaque effet de `ACTIVABLES`.
    dernieres_bascules: [Option<Instant>; ACTIVABLES.len()],
    detecteur: Detecteur,
    /// Son de clic du profil actif.
    son: &'static [u8],
    pub echec: Option<String>,
    debut: Instant,
    dernier_bilan: Instant,
    reveils: u64,
    lectures_curseur: u64,
    clics_gauche: u64,
    clics_droit: u64,
    sons_joues: u64,
}

/// Position du curseur sur cet écran, ou `None` s'il est sur un autre.
fn curseur_sur(ecran: &Ecran, lectures: &mut u64) -> Option<Point> {
    *lectures += 1;
    plateforme::position_curseur(ecran).filter(|p| ecran.contient(*p))
}

impl App {
    /// `debut` est l'instant de lancement, point de départ de la durée du bilan.
    pub fn new(debut: Instant) -> Self {
        let chemin_profils =
            plateforme::dossier_donnees().map(|dossier| dossier.join(stockage::NOM_FICHIER));
        let chargement = match &chemin_profils {
            Some(chemin) => stockage::charger(chemin),
            None => Chargement {
                profils: Profils::default(),
                a_enregistrer: false,
                messages: vec![
                    "dossier de données introuvable : profils d'exemple, rien ne sera enregistré"
                        .into(),
                ],
            },
        };
        for message in &chargement.messages {
            crate::sonde::ecrire(&format!("Sonde : profils : {message}"));
        }
        let profils = chargement.profils;
        let config = Config {
            effets: profils.profil_actif().effets.borne(),
            ..Config::default()
        };
        let mut sons = HashMap::new();
        let son = son_pour(&mut sons, config.effets.son);
        let app = App {
            config,
            profils,
            chemin_profils,
            cibles: Vec::new(),
            sons,
            derniere_bascule_profil: None,
            changements_profil: 0,
            activations: Activations::default(),
            surcouches: Vec::new(),
            icone: None,
            raccourci: None,
            dernieres_bascules: [None; ACTIVABLES.len()],
            detecteur: Detecteur::default(),
            son,
            echec: None,
            debut,
            dernier_bilan: debut,
            reveils: 0,
            lectures_curseur: 0,
            clics_gauche: 0,
            clics_droit: 0,
            sons_joues: 0,
        };
        if chargement.a_enregistrer {
            app.enregistrer_profils();
        }
        crate::sonde::ecrire(&format!(
            "Sonde : profil actif « {} » ({} sur {}), adaptateur demandé {:?}, fichier {}",
            app.profils.profil_actif().nom,
            app.profils.actif + 1,
            app.profils.profils.len(),
            app.profils.adaptateur,
            app.chemin_profils
                .as_ref()
                .map_or("aucun".to_string(), |c| c.display().to_string())
        ));
        app
    }

    /// Écrit le fichier des profils. Un échec va dans la sonde : l'application continue avec les
    /// profils en mémoire.
    fn enregistrer_profils(&self) {
        let Some(chemin) = &self.chemin_profils else {
            return;
        };
        if let Err(e) = stockage::enregistrer(chemin, &self.profils) {
            crate::sonde::ecrire(&format!(
                "Sonde : enregistrement des profils impossible ({}) : {e}",
                chemin.display()
            ));
        }
    }

    /// Ligne de bilan : durée, temps processeur et ce que la boucle a fait depuis le lancement.
    pub fn bilan(&self) -> String {
        let statistiques: Vec<_> = self
            .surcouches
            .iter()
            .map(|s| s.rendu.statistiques())
            .collect();
        let compteurs = Compteurs {
            reveils: self.reveils,
            lectures_curseur: self.lectures_curseur,
            images_par_ecran: self.surcouches.iter().map(|s| s.images).collect(),
            echecs_acquisition: statistiques.iter().map(|s| s.echecs_acquisition).sum(),
            reconfigurations: statistiques.iter().map(|s| s.reconfigurations).sum(),
            clics_gauche: self.clics_gauche,
            clics_droit: self.clics_droit,
            sons_joues: self.sons_joues,
            profil_actif: self.profils.profil_actif().nom.clone(),
            changements_profil: self.changements_profil,
        };
        sonde::bilan(
            self.debut.elapsed(),
            plateforme::temps_processeur(),
            &compteurs,
        )
    }

    fn demarrer(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let ecrans = plateforme::ecrans(event_loop)?;
        let mut fenetres = Vec::new();
        for ecran in &ecrans {
            fenetres.push(plateforme::creer_surcouche(event_loop, ecran)?);
        }

        let cartes = plateforme::cartes_ecrans(&ecrans);
        let (gpu, premiere) = Gpu::nouveau(
            event_loop.owned_display_handle(),
            &fenetres[0],
            &cartes,
            self.profils.adaptateur,
        )?;
        let mut premiere = Some(premiere);
        for (indice, (ecran, fenetre)) in ecrans.iter().zip(fenetres).enumerate() {
            let surface = match premiere.take() {
                Some(surface) => Ok(surface),
                None => gpu.surface(&fenetre),
            };
            let rendu = surface
                .and_then(|s| Rendu::nouveau(&gpu, s, &fenetre, ecran, indice, &self.config));
            match rendu {
                Ok(rendu) => self.surcouches.push(Surcouche {
                    fenetre,
                    ecran: *ecran,
                    rendu,
                    curseur: curseur_sur(ecran, &mut self.lectures_curseur),
                    a_redessiner: true,
                    images: 0,
                }),
                // L'écran principal est indispensable ; un écran supplémentaire qui échoue
                // reste simplement sans voile.
                Err(e) if indice == 0 => return Err(e),
                Err(e) => crate::sonde::ecrire(&format!("Sonde : écran {indice} ignoré : {e}")),
            }
        }

        self.icone = Some(plateforme::creer_icone()?);
        // Dans l'ordre : les effets, le profil suivant, puis chaque profil qui a un raccourci.
        let mut demandes: Vec<Raccourci> = self.config.raccourcis_effets().to_vec();
        self.cibles = (0..demandes.len()).map(Cible::Effet).collect();
        if let Some(r) = self.profils.raccourci_suivant {
            demandes.push(r);
            self.cibles.push(Cible::Suivant);
        }
        for (indice, profil) in self.profils.profils.iter().enumerate() {
            if let Some(r) = profil.raccourci {
                demandes.push(r);
                self.cibles.push(Cible::Profil(indice));
            }
        }
        match plateforme::creer_raccourcis(&demandes) {
            Ok(raccourci) => {
                for (indice, e) in &raccourci.refus {
                    let qui = match self.cibles[*indice] {
                        Cible::Effet(i) => ACTIVABLES[i].1.to_string(),
                        Cible::Suivant => "Profil suivant".to_string(),
                        Cible::Profil(i) => format!("Profil « {} »", self.profils.profils[i].nom),
                    };
                    crate::sonde::ecrire(&format!(
                        "Sonde : raccourci global « {qui} » indisponible : {e}"
                    ));
                }
                self.raccourci = Some(raccourci);
            }
            // Sans raccourci, les effets restent actifs en permanence : pas une raison de quitter.
            Err(e) => crate::sonde::ecrire(&format!("Sonde : raccourci global indisponible : {e}")),
        }

        for s in &mut self.surcouches {
            let curseur = s.curseur;
            if s.rendu.dessiner(&s.ecran, || curseur, &self.activations) {
                s.images += 1;
                s.a_redessiner = false;
            }
            s.fenetre.set_visible(true);
        }
        Ok(())
    }

    /// Active ou coupe l'effet `ACTIVABLES[indice]`. Les effets se basculent indépendamment.
    fn basculer(&mut self, indice: usize) {
        let maintenant = Instant::now();
        if self.dernieres_bascules[indice]
            .is_some_and(|t| maintenant.duration_since(t) < ANTI_REPETITION)
        {
            return;
        }
        self.dernieres_bascules[indice] = Some(maintenant);
        let (nom, libelle) = ACTIVABLES[indice];
        let actif = self.activations.basculer(nom);
        crate::sonde::ecrire(&format!(
            "{libelle} {}",
            if actif { "activé" } else { "coupé" }
        ));
        for s in &mut self.surcouches {
            s.a_redessiner = true;
        }
    }

    /// Passe au profil `indice`. `par_raccourci_du_profil` : le raccourci propre au profil, qui
    /// réactive aussi les effets coupés (celui du profil suivant ne le fait pas). Le profil est
    /// appliqué aux effets en place, sans recréer de pipeline ni de présentation, et le dernier
    /// profil utilisé est enregistré.
    fn passer_au_profil(&mut self, indice: usize, origine: &str, par_raccourci_du_profil: bool) {
        let maintenant = Instant::now();
        if self
            .derniere_bascule_profil
            .is_some_and(|t| maintenant.duration_since(t) < ANTI_REPETITION)
        {
            return;
        }
        self.derniere_bascule_profil = Some(maintenant);
        let ancien = self.profils.actif;
        if self.profils.activer(indice).is_err() {
            return;
        }
        if par_raccourci_du_profil {
            self.activations.tout_activer();
        }
        let nom = self.profils.profil_actif().nom.clone();
        if indice == ancien {
            crate::sonde::ecrire(&format!("Sonde : profil « {nom} » conservé ({origine})"));
        } else {
            self.config.effets = self.profils.profil_actif().effets.borne();
            self.son = son_pour(&mut self.sons, self.config.effets.son);
            for s in &mut self.surcouches {
                s.rendu.appliquer(&self.config.effets);
            }
            self.changements_profil += 1;
            crate::sonde::ecrire(&format!(
                "Sonde : profil « {} » → « {nom} » ({origine})",
                self.profils.profils[ancien].nom
            ));
            self.enregistrer_profils();
        }
        for s in &mut self.surcouches {
            s.a_redessiner = true;
        }
    }

    /// Lit les boutons une fois, puis transmet chaque appui neuf à l'écran où se trouve le curseur
    /// et au son. Un appui compté l'est même si son effet est coupé : la sonde vérifie la détection.
    fn traiter_clics(&mut self) {
        let appuis = self.detecteur.appuis(plateforme::boutons());
        for (present, bouton) in [
            (appuis.gauche, Bouton::Gauche),
            (appuis.droit, Bouton::Droit),
        ] {
            if !present {
                continue;
            }
            match bouton {
                Bouton::Gauche => self.clics_gauche += 1,
                Bouton::Droit => self.clics_droit += 1,
            }
            for s in &mut self.surcouches {
                if let Some(position) = s.curseur {
                    // Pas de redessin demandé ici : si un effet a une animation à faire, la
                    // boucle le voit juste après avec `animation_en_cours`.
                    s.rendu.clic(bouton, position, &self.activations);
                }
            }
            if self.activations.est_actif(SON) && plateforme::jouer_son(self.son) {
                self.sons_joues += 1;
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.surcouches.is_empty() {
            return;
        }
        if let Err(erreur) = self.demarrer(event_loop) {
            self.echec = Some(erreur);
            event_loop.exit();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, evenement: WindowEvent) {
        let Some(s) = self.surcouches.iter_mut().find(|s| s.fenetre.id() == id) else {
            return;
        };
        match evenement {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(taille) => {
                s.rendu.redimensionner(taille.width, taille.height);
                s.a_redessiner = true;
            }
            WindowEvent::RedrawRequested => {
                let lectures = &mut self.lectures_curseur;
                let curseur = &mut s.curseur;
                let ecran = &s.ecran;
                let presentee = s.rendu.dessiner(
                    ecran,
                    || {
                        *curseur = curseur_sur(ecran, lectures);
                        *curseur
                    },
                    &self.activations,
                );
                s.a_redessiner = !presentee;
                s.images += u64::from(presentee);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.reveils += 1;
        if self.icone.as_ref().is_some_and(Icone::quitter_demande) {
            event_loop.exit();
            return;
        }
        for indice in self
            .raccourci
            .as_ref()
            .map(RaccourciGlobal::activations_demandees)
            .unwrap_or_default()
        {
            match self.cibles.get(indice).copied() {
                Some(Cible::Effet(i)) => self.basculer(i),
                Some(Cible::Suivant) => {
                    let suivant = self.profils.indice_suivant();
                    self.passer_au_profil(suivant, "raccourci du profil suivant", false);
                }
                Some(Cible::Profil(i)) => {
                    self.passer_au_profil(i, "raccourci du profil", true);
                }
                None => {}
            }
        }
        if self.dernier_bilan.elapsed() >= PERIODE_BILAN {
            self.dernier_bilan = Instant::now();
            sonde::bilan_en_cours(&self.bilan());
        }
        for s in &mut self.surcouches {
            let position = curseur_sur(&s.ecran, &mut self.lectures_curseur);
            if position != s.curseur {
                s.curseur = position;
                s.a_redessiner = true;
            }
        }
        // Après la lecture des curseurs : un clic va à l'écran où le curseur est maintenant.
        self.traiter_clics();
        for s in &mut self.surcouches {
            // Une animation en cours (ondulation, fondu de la traînée) demande des images, et
            // seulement elle : sans animation ni changement, aucune image n'est demandée.
            if s.a_redessiner || s.rendu.animation_en_cours(&self.activations) {
                s.fenetre.request_redraw();
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + PERIODE));
    }
}
