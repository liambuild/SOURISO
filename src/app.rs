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
use crate::coeur::reglages::{self, ActionProfil, Base, CibleRaccourci, Demande, Faute, Reglage};
use crate::coeur::son::{avec_volume, Son};
use crate::coeur::stockage::{self, Chargement};
use crate::coeur::{Config, Ecran, Point, Raccourci};
use crate::plateforme::{self, FenetreReglages, Icone, RaccourciGlobal};
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

/// Délai entre le dernier réglage de la fenêtre et l'écriture du fichier des profils : un curseur
/// qu'on fait glisser ne réécrit pas le fichier à chaque image.
const DELAI_ENREGISTREMENT: Duration = Duration::from_millis(300);

/// Tant que la fenêtre de réglages bouge, la zone sans voile déborde de cette marge (pixels
/// logiques), pour que le retard de la position ne laisse pas un liseré sombre.
const MARGE_DEPLACEMENT: f32 = 8.0;

/// Une fenêtre qui n'a rien signalé depuis ce délai ne bouge plus.
const FIN_DEPLACEMENT: Duration = Duration::from_millis(250);

/// Après la fin du processus des réglages, temps laissé aux dernières lignes pour arriver.
const DELAI_DERNIERES_LIGNES: Duration = Duration::from_millis(50);

/// Délai minimal entre deux bascules : une touche maintenue répète l'événement du raccourci.
const ANTI_REPETITION: Duration = Duration::from_millis(400);

/// Ce que déclenche un raccourci global, dans l'ordre où ils sont demandés au système.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cible {
    /// Active ou coupe l'effet `ACTIVABLES[i]`.
    Effet(usize),
    /// Passe au profil suivant.
    Suivant,
    /// Passe au profil `i` et applique l'activation des effets enregistrée dans ce profil.
    Profil(usize),
}

/// Son de clic mis au volume réglé, au centième près. La lecture asynchrone du système lit
/// directement ces octets après le retour de l'appel : ils sont gardés pour toute l'exécution
/// (fuite volontaire, une par couple son et volume en centièmes rencontré, au plus 101 par son,
/// soit moins de 6 Mo en tout si l'on balaie tout le curseur de volume de chaque son).
fn son_pour(sons: &mut HashMap<(Son, u32), &'static [u8]>, reglage: SonClic) -> &'static [u8] {
    let r = reglage.borne();
    let centiemes = (r.volume * 100.0).round() as u32;
    sons.entry((r.choix, centiemes)).or_insert_with(|| {
        let volume = centiemes as f32 / 100.0;
        Box::leak(avec_volume(r.choix.wav(), volume).into_boxed_slice())
    })
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
    /// Fois où la taille de la fenêtre a dû être rétablie après un redimensionnement imprévu.
    rectifications: u8,
}

/// La fenêtre de réglages quand elle est ouverte.
struct Reglages {
    fenetre: FenetreReglages,
    /// Ce que « Revenir aux valeurs du profil » rétablit : le profil à l'ouverture ou au dernier
    /// changement de profil.
    base: Base,
    /// Dernière position et taille reçues (pixels physiques du bureau), si elle est visible.
    zone: Option<(i32, i32, i32, i32)>,
    /// Dernière fois que la fenêtre a signalé un déplacement ou un redimensionnement.
    dernier_mouvement: Option<Instant>,
    /// Vrai pendant que la zone sans voile porte la marge de déplacement.
    marge_active: bool,
    /// Vrai pendant la saisie d'un raccourci : les raccourcis globaux sont suspendus.
    capture: bool,
    /// Depuis quand le processus est terminé, pour laisser arriver ses dernières lignes.
    terminee_depuis: Option<Instant>,
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
    reglages: Option<Reglages>,
    ouvertures_reglages: u64,
    /// Instant à partir duquel le fichier des profils doit être réécrit (réglages en attente).
    enregistrement_prevu: Option<Instant>,
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
            reglages: None,
            ouvertures_reglages: 0,
            enregistrement_prevu: None,
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
            ouvertures_reglages: self.ouvertures_reglages,
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

        for (indice, (ecran, fenetre)) in ecrans.iter().zip(&fenetres).enumerate() {
            crate::sonde::ecrire(&format!(
                "Sonde : écran {indice} : taille physique {}x{}, échelle {}, position {:?} ; \
                 fenêtre après création : taille {:?}, position {:?}",
                ecran.taille.0,
                ecran.taille.1,
                ecran.echelle,
                ecran.origine,
                fenetre.inner_size(),
                fenetre.outer_position().ok()
            ));
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
                    rectifications: 0,
                }),
                // L'écran principal est indispensable ; un écran supplémentaire qui échoue
                // reste simplement sans voile.
                Err(e) if indice == 0 => return Err(e),
                Err(e) => crate::sonde::ecrire(&format!("Sonde : écran {indice} ignoré : {e}")),
            }
        }

        self.icone = Some(plateforme::creer_icone()?);
        self.enregistrer_raccourcis();

        for s in &mut self.surcouches {
            let curseur = s.curseur;
            if s.rendu.dessiner(&s.ecran, || curseur, &self.activations) {
                s.images += 1;
                s.a_redessiner = false;
            }
            s.fenetre.set_visible(true);
            // L'affichage peut déclencher un changement d'échelle qui redimensionne la fenêtre :
            // la taille est remise et la surface reconfigurée tout de suite, sans attendre
            // l'événement de redimensionnement (qui peut ne jamais venir).
            Self::rectifier(s, "après affichage");
        }
        Ok(())
    }

    /// Si la fenêtre n'a pas la taille de son écran, la remet et reconfigure la surface. Trois
    /// essais au plus : un système qui refuse ne doit pas faire boucler les événements.
    fn rectifier(s: &mut Surcouche, quand: &str) {
        let voulue = s.ecran.taille;
        let actuelle = s.fenetre.inner_size();
        if (actuelle.width, actuelle.height) == voulue {
            return;
        }
        if s.rectifications >= 3 {
            return;
        }
        s.rectifications += 1;
        let obtenue = plateforme::imposer_geometrie(&s.fenetre, &s.ecran);
        s.rendu.redimensionner(obtenue.width, obtenue.height);
        s.a_redessiner = true;
        crate::sonde::ecrire(&format!(
            "Sonde : surcouche {quand} : taille {}x{} au lieu de {}x{}, rétablie ({}x{} obtenu, essai {})",
            actuelle.width,
            actuelle.height,
            voulue.0,
            voulue.1,
            obtenue.width,
            obtenue.height,
            s.rectifications
        ));
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
        // Un raccourci ne change que la session ; la fenêtre ouverte montre l'état réel.
        if let Some(r) = &mut self.reglages {
            r.fenetre
                .envoyer(&reglages::message_activation(nom, actif).to_string());
        }
    }

    /// Enregistre les raccourcis globaux : les effets, le profil suivant, puis chaque profil qui
    /// en a un. Refait à chaque changement des profils ou des raccourcis, car les cibles
    /// désignent les profils par leur rang. Renvoie les cibles que le système a refusées (déjà
    /// prises) ; chacune est écrite dans la sonde.
    fn enregistrer_raccourcis(&mut self) -> Vec<Cible> {
        // Le système n'accepte pas deux fois une combinaison : l'ancien gestionnaire part d'abord.
        plateforme::suspendre_raccourcis(&mut self.raccourci);
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
        let mut refusees = Vec::new();
        match plateforme::creer_raccourcis(&demandes) {
            Ok(raccourci) => {
                for (indice, e) in &raccourci.refus {
                    let cible = self.cibles[*indice];
                    let qui = match cible {
                        Cible::Effet(i) => ACTIVABLES[i].1.to_string(),
                        Cible::Suivant => "Profil suivant".to_string(),
                        Cible::Profil(i) => format!("Profil « {} »", self.profils.profils[i].nom),
                    };
                    crate::sonde::ecrire(&format!(
                        "Sonde : raccourci global « {qui} » indisponible : {e}"
                    ));
                    refusees.push(cible);
                }
                self.raccourci = Some(raccourci);
            }
            // Sans raccourci, les effets restent actifs en permanence : pas une raison de quitter.
            Err(e) => crate::sonde::ecrire(&format!("Sonde : raccourci global indisponible : {e}")),
        }
        refusees
    }

    /// Met les effets, leur activation et la base de « Revenir » à l'état du profil actif, sans
    /// recréer de pipeline ni de présentation. `nouvelle_base` : le profil vient de changer.
    fn synchroniser_profil_actif(&mut self, nouvelle_base: bool) {
        let profil = self.profils.profil_actif();
        let (effets, actifs) = (profil.effets.borne(), profil.actifs);
        self.config.effets = effets;
        self.activations.appliquer(&actifs);
        self.son = son_pour(&mut self.sons, effets.son);
        for s in &mut self.surcouches {
            s.rendu.appliquer(&effets);
            s.a_redessiner = true;
        }
        if let (true, Some(r)) = (nouvelle_base, &mut self.reglages) {
            r.base = Base::du_profil(self.profils.profil_actif());
        }
    }

    /// Passe au profil `indice` et applique l'activation des effets qu'il enregistre. Les réglages
    /// sont remplacés dans les effets en place, sans recréer de pipeline ni de présentation, et le
    /// dernier profil utilisé est enregistré. `avec_delai` : les raccourcis sont protégés contre
    /// la répétition d'une touche maintenue ; la fenêtre de réglages ne l'est pas.
    fn passer_au_profil(&mut self, indice: usize, origine: &str, avec_delai: bool) {
        let maintenant = Instant::now();
        if avec_delai {
            if self
                .derniere_bascule_profil
                .is_some_and(|t| maintenant.duration_since(t) < ANTI_REPETITION)
            {
                return;
            }
            self.derniere_bascule_profil = Some(maintenant);
        }
        let ancien = self.profils.actif;
        if self.profils.activer(indice).is_err() {
            return;
        }
        let nom = self.profils.profil_actif().nom.clone();
        if indice == ancien {
            crate::sonde::ecrire(&format!("Sonde : profil « {nom} » conservé ({origine})"));
            self.synchroniser_profil_actif(false);
        } else {
            self.synchroniser_profil_actif(true);
            self.changements_profil += 1;
            crate::sonde::ecrire(&format!(
                "Sonde : profil « {} » → « {nom} » ({origine})",
                self.profils.profils[ancien].nom
            ));
            self.enregistrer_profils();
        }
        self.envoyer_etat(None, false);
    }

    /// Prévoit l'écriture du fichier des profils après un réglage de la fenêtre.
    fn prevoir_enregistrement(&mut self) {
        self.enregistrement_prevu
            .get_or_insert_with(|| Instant::now() + DELAI_ENREGISTREMENT);
    }

    /// Écrit tout de suite ce qui attend d'être écrit.
    fn enregistrer_si_prevu(&mut self) {
        if self.enregistrement_prevu.take().is_some() {
            self.enregistrer_profils();
        }
    }

    /// Ouvre la fenêtre de réglages, ou la ramène devant si elle est déjà ouverte.
    fn ouvrir_reglages(&mut self) {
        if let Some(r) = &mut self.reglages {
            r.fenetre.envoyer(r#"{"type":"montrer"}"#);
            return;
        }
        match FenetreReglages::ouvrir() {
            Ok(fenetre) => {
                self.ouvertures_reglages += 1;
                crate::sonde::ecrire("Sonde : fenêtre de réglages ouverte");
                self.reglages = Some(Reglages {
                    fenetre,
                    base: Base::du_profil(self.profils.profil_actif()),
                    zone: None,
                    dernier_mouvement: None,
                    marge_active: false,
                    capture: false,
                    terminee_depuis: None,
                });
            }
            Err(e) => {
                crate::sonde::ecrire(&format!("Sonde : fenêtre de réglages non ouverte : {e}"))
            }
        }
    }

    /// Ferme la fenêtre (si elle ne l'est pas déjà) : le voile reprend toute la place, les
    /// raccourcis suspendus sont rétablis, les réglages en attente sont écrits.
    fn fermer_reglages(&mut self, raison: &str) {
        let Some(r) = self.reglages.take() else {
            return;
        };
        let capture = r.capture;
        // Le processus s'arrête avec son entrée (ou est forcé), ses processus web avec lui.
        drop(r);
        crate::sonde::ecrire(&format!("Sonde : fenêtre de réglages fermée ({raison})"));
        if capture || self.raccourci.is_none() {
            self.enregistrer_raccourcis();
        }
        self.enregistrer_si_prevu();
        self.mettre_a_jour_exemption();
    }

    /// Envoie l'état complet à la fenêtre ouverte. `complet` : avec la description des réglages.
    fn envoyer_etat(&mut self, faute: Option<Faute>, complet: bool) {
        let Some(r) = &mut self.reglages else {
            return;
        };
        let etat = reglages::etat(
            &self.profils,
            &self.activations.actifs(),
            &r.base,
            faute.as_ref(),
            complet,
        );
        r.fenetre.envoyer(&etat.to_string());
    }

    fn envoyer_erreur(&mut self, message: &str) {
        crate::sonde::ecrire(&format!("Sonde : réglages : {message}"));
        if let Some(r) = &mut self.reglages {
            r.fenetre
                .envoyer(&reglages::message_erreur(message).to_string());
        }
    }

    /// Calcule, pour chaque écran, la zone que couvre la fenêtre de réglages (pixels physiques de
    /// la surface) et la transmet au rendu : le voile n'y est pas dessiné.
    fn mettre_a_jour_exemption(&mut self) {
        let (zone, en_mouvement) = match &self.reglages {
            Some(r) => (r.zone, r.marge_active),
            None => (None, false),
        };
        for s in &mut self.surcouches {
            let marge = if en_mouvement {
                MARGE_DEPLACEMENT * s.ecran.echelle as f32
            } else {
                0.0
            };
            let rect = zone.and_then(|(x, y, l, h)| {
                let (ox, oy) = s.ecran.origine;
                let (g, t) = ((x - ox) as f32 - marge, (y - oy) as f32 - marge);
                let (d, b) = (g + l as f32 + 2.0 * marge, t + h as f32 + 2.0 * marge);
                let (largeur, hauteur) = (s.ecran.taille.0 as f32, s.ecran.taille.1 as f32);
                // Hors de cet écran : rien à ménager ici.
                (l > 0 && h > 0 && d > 0.0 && b > 0.0 && g < largeur && t < hauteur)
                    .then_some([g, t, d, b])
            });
            if s.rendu.exempter(rect) {
                s.a_redessiner = true;
            }
        }
    }

    /// Lit ce que la fenêtre a écrit, constate sa fin, passe la main au traitement des demandes.
    fn traiter_reglages(&mut self) {
        let Some(r) = &mut self.reglages else {
            return;
        };
        let terminee = r.fenetre.terminee();
        let lignes = r.fenetre.lignes();
        for ligne in lignes {
            match reglages::lire_demande(&ligne) {
                Ok(demande) => self.traiter_demande(demande),
                Err(e) => self.envoyer_erreur(&e),
            }
            if self.reglages.is_none() {
                return;
            }
        }
        let Some(r) = &mut self.reglages else {
            return;
        };
        if terminee {
            let depuis = *r.terminee_depuis.get_or_insert_with(Instant::now);
            if depuis.elapsed() >= DELAI_DERNIERES_LIGNES {
                self.fermer_reglages("processus terminé");
            }
            return;
        }
        // Fin du déplacement : la marge disparaît.
        let en_mouvement = r
            .dernier_mouvement
            .is_some_and(|t| t.elapsed() < FIN_DEPLACEMENT);
        if en_mouvement != r.marge_active {
            r.marge_active = en_mouvement;
            self.mettre_a_jour_exemption();
        }
    }

    fn traiter_demande(&mut self, demande: Demande) {
        match demande {
            Demande::Pret => self.envoyer_etat(None, true),
            Demande::Regler { chemin, valeur } => self.regler(&chemin, &valeur),
            Demande::Revenir => {
                let Some(r) = &self.reglages else {
                    return;
                };
                let base = r.base.clone();
                let actif = self.profils.actif;
                base.retablir(&mut self.profils.profils[actif]);
                self.synchroniser_profil_actif(false);
                self.prevoir_enregistrement();
                self.envoyer_etat(None, false);
            }
            Demande::Profil {
                action,
                indice,
                nom,
            } => self.gerer_profil(action, indice, &nom),
            Demande::Raccourci { cible, raccourci } => self.definir_raccourci(cible, raccourci),
            Demande::Capture { debut } => {
                if let Some(r) = &mut self.reglages {
                    r.capture = debut;
                }
                if debut {
                    plateforme::suspendre_raccourcis(&mut self.raccourci);
                } else if self.raccourci.is_none() {
                    self.enregistrer_raccourcis();
                }
            }
            Demande::Adaptateur { valeur } => {
                self.profils.adaptateur = valeur;
                self.prevoir_enregistrement();
            }
            Demande::Fenetre {
                x,
                y,
                l,
                h,
                visible,
            } => {
                let Some(r) = &mut self.reglages else {
                    return;
                };
                let zone = (visible && l > 0 && h > 0).then_some((x, y, l, h));
                if zone != r.zone {
                    r.zone = zone;
                    r.dernier_mouvement = Some(Instant::now());
                    r.marge_active = true;
                    self.mettre_a_jour_exemption();
                }
            }
            Demande::Fermee => self.fermer_reglages("fermée par l'utilisateur"),
        }
    }

    /// Un réglage d'effet ou l'interrupteur d'un effet, appliqué tout de suite aux effets réels.
    fn regler(&mut self, chemin: &str, valeur: &serde_json::Value) {
        let actif = self.profils.actif;
        match reglages::regler(&mut self.profils.profils[actif], chemin, valeur) {
            Ok(Reglage::Effets) => {
                self.synchroniser_effets();
                // Entendre le son qu'on vient de régler, s'il est actif.
                if chemin.starts_with("son.") && self.activations.est_actif(SON) {
                    plateforme::jouer_son(self.son);
                }
            }
            Ok(Reglage::Activation(nom, actif)) => {
                self.activations.definir(nom, actif);
                for s in &mut self.surcouches {
                    s.a_redessiner = true;
                }
            }
            Err(e) => {
                self.envoyer_erreur(&e);
                return;
            }
        }
        self.prevoir_enregistrement();
    }

    /// Applique les réglages du profil actif aux effets, sans toucher à l'activation de session.
    fn synchroniser_effets(&mut self) {
        let effets = self.profils.profil_actif().effets.borne();
        self.config.effets = effets;
        self.son = son_pour(&mut self.sons, effets.son);
        for s in &mut self.surcouches {
            s.rendu.appliquer(&effets);
            s.a_redessiner = true;
        }
    }

    /// Créer, dupliquer, renommer, supprimer, activer un profil. Un profil créé ou dupliqué devient
    /// le profil actif : c'est celui qu'on vient régler.
    fn gerer_profil(&mut self, action: ActionProfil, indice: usize, nom: &str) {
        let faute = |cible: String, message: String| Some(Faute { cible, message });
        let resultat: Result<Option<usize>, Faute> = match action {
            ActionProfil::Activer => {
                self.passer_au_profil(indice, "fenêtre de réglages", false);
                return;
            }
            ActionProfil::Creer => self
                .profils
                .creer(nom)
                .map(Some)
                .map_err(|m| faute("nouveau".into(), m).unwrap()),
            ActionProfil::Dupliquer => self
                .profils
                .dupliquer(indice)
                .map(Some)
                .map_err(|m| faute(format!("profil:{indice}"), m).unwrap()),
            ActionProfil::Renommer => self
                .profils
                .renommer(indice, nom)
                .map(|()| None)
                .map_err(|m| faute(format!("renommer:{indice}"), m).unwrap()),
            ActionProfil::Supprimer => {
                let actif_avant = self.profils.actif;
                self.profils
                    .supprimer(indice)
                    .map(|()| {
                        // Les rangs ont changé : les raccourcis sont à refaire, et si le profil
                        // actif a disparu, le profil par défaut prend sa place.
                        self.enregistrer_raccourcis();
                        // Seul le départ du profil actif change les effets ; supprimer un autre
                        // profil ne touche ni aux réglages ni à l'activation en cours.
                        if actif_avant == indice {
                            self.synchroniser_profil_actif(true);
                            self.changements_profil += 1;
                        }
                        None
                    })
                    .map_err(|m| faute(format!("profil:{indice}"), m).unwrap())
            }
        };
        match resultat {
            Ok(nouveau) => {
                self.enregistrement_prevu = None;
                if let Some(i) = nouveau {
                    // Les profils créés ou copiés n'ont pas de raccourci : aucun rang à refaire.
                    // Le passage au profil écrit le fichier et renvoie l'état.
                    self.passer_au_profil(i, "fenêtre de réglages", false);
                } else {
                    self.enregistrer_profils();
                    self.envoyer_etat(None, false);
                }
            }
            Err(f) => self.envoyer_etat(Some(f), false),
        }
    }

    /// Fixe le raccourci d'un profil ou du profil suivant. Le cœur refuse ce que ce programme
    /// détient déjà ; le système, ce qu'une autre application détient : dans les deux cas
    /// l'ancien raccourci est rétabli et la fenêtre l'explique.
    fn definir_raccourci(&mut self, cible: CibleRaccourci, nouveau: Option<Raccourci>) {
        let (ancien, resultat, cible_interne, lieu) = match cible {
            CibleRaccourci::Profil(i) => {
                let Some(p) = self.profils.profils.get(i) else {
                    self.envoyer_erreur(&format!("le profil {i} n'existe pas"));
                    return;
                };
                let ancien = p.raccourci;
                (
                    ancien,
                    self.profils.definir_raccourci(i, nouveau),
                    Cible::Profil(i),
                    format!("profil:{i}"),
                )
            }
            CibleRaccourci::Suivant(_) => {
                let ancien = self.profils.raccourci_suivant;
                (
                    ancien,
                    self.profils.definir_raccourci_suivant(nouveau),
                    Cible::Suivant,
                    "suivant".to_string(),
                )
            }
        };
        let mut faute = resultat.err();
        // Dans tous les cas, les raccourcis suspendus par la saisie sont rétablis.
        let refusees = self.enregistrer_raccourcis();
        if faute.is_none() && refusees.contains(&cible_interne) {
            match cible {
                CibleRaccourci::Profil(i) => self.profils.profils[i].raccourci = ancien,
                CibleRaccourci::Suivant(_) => self.profils.raccourci_suivant = ancien,
            }
            self.enregistrer_raccourcis();
            faute = Some("indisponible : déjà utilisé par une autre application".to_string());
        }
        if let Some(r) = &mut self.reglages {
            r.capture = false;
        }
        if faute.is_none() {
            self.enregistrer_profils();
            self.enregistrement_prevu = None;
        }
        let faute = faute.map(|message| Faute {
            cible: lieu,
            message,
        });
        self.envoyer_etat(faute, false);
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
                Self::rectifier(s, "après redimensionnement");
            }
            // Windows envoie ce changement quand la surcouche passe sur un écran d'une autre échelle
            // que celui de référence, avec une taille suggérée dérivée de l'autre écran. La
            // surcouche doit couvrir exactement son écran : la taille physique de celui-ci est
            // imposée et la surface reconfigurée aussitôt.
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                mut inner_size_writer,
            } => {
                let (largeur, hauteur) = s.ecran.taille;
                let accepte = inner_size_writer
                    .request_inner_size(winit::dpi::PhysicalSize::new(largeur, hauteur))
                    .is_ok();
                s.rendu.redimensionner(largeur, hauteur);
                s.a_redessiner = true;
                crate::sonde::ecrire(&format!(
                    "Sonde : changement d'échelle reçu : {scale_factor} (écran à {}), taille \
                     imposée {largeur}x{hauteur} {}",
                    s.ecran.echelle,
                    if accepte { "acceptée" } else { "refusée" }
                ));
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
        let choix = self
            .icone
            .as_ref()
            .map(Icone::evenements)
            .unwrap_or_default();
        if choix.quitter {
            event_loop.exit();
            return;
        }
        if choix.reglages {
            self.ouvrir_reglages();
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
                    self.passer_au_profil(suivant, "raccourci du profil suivant", true);
                }
                Some(Cible::Profil(i)) => {
                    self.passer_au_profil(i, "raccourci du profil", true);
                }
                None => {}
            }
        }
        if self.reglages.is_some() {
            self.traiter_reglages();
        }
        if self
            .enregistrement_prevu
            .is_some_and(|t| t <= Instant::now())
        {
            self.enregistrer_si_prevu();
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

impl Drop for App {
    /// Les réglages encore en attente sont écrits avant de quitter.
    fn drop(&mut self) {
        self.enregistrer_si_prevu();
    }
}
