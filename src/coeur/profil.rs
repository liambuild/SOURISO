//! Profils nommés. Un profil regroupe les réglages de tous les effets et son raccourci pour y
//! passer. Le premier de la liste est le profil par défaut : il ne se supprime pas. Modèle pur,
//! testable sous Linux ; la lecture et l'écriture du fichier sont dans `stockage`.

use serde::{Deserialize, Serialize};

use super::activation::ActifsProfil;
use super::config::{raccourci, Adaptateur, Anneau, Config, Effets, Ondulation, Projecteur};
use super::config::{Raccourci, SonClic, Trainee};
use super::son::Son;

/// Version du format du fichier des profils. À augmenter, avec une migration, à chaque changement
/// incompatible. La 2 ajoute l'activation de chaque effet dans le profil (`actifs`).
pub const VERSION_FORMAT: u32 = 2;

/// Plus ancienne version relue : un fichier de cette version est migré (tous les effets actifs).
pub const VERSION_MINIMALE: u32 = 1;

/// Longueur maximale d'un nom de profil, en caractères.
pub const LONGUEUR_NOM_MAX: usize = 40;

const NOM_DEFAUT: &str = "Par défaut";
const NOM_PRESENTATION: &str = "Présentation";
const NOM_ENREGISTREMENT: &str = "Enregistrement";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profil {
    pub nom: String,
    /// Passe à ce profil et active les effets s'ils étaient coupés. `None` : pas de raccourci.
    pub raccourci: Option<Raccourci>,
    pub effets: Effets,
    /// Effets que le profil active : appliqués quand on passe à lui, modifiés par l'interrupteur
    /// de la fenêtre de réglages (pas par les raccourcis d'effet, qui ne durent que la session).
    pub actifs: ActifsProfil,
}

impl Default for Profil {
    fn default() -> Self {
        Profil {
            nom: NOM_DEFAUT.into(),
            raccourci: Some(raccourci('D')),
            effets: Effets::default(),
            actifs: ActifsProfil::default(),
        }
    }
}

impl Profil {
    /// Gros repères très visibles, pour une salle ou un partage d'écran.
    fn presentation() -> Profil {
        Profil {
            nom: NOM_PRESENTATION.into(),
            raccourci: Some(raccourci('E')),
            effets: Effets {
                projecteur: Projecteur {
                    rayon: 260.0,
                    adoucissement: 140.0,
                    intensite: 0.75,
                    couleur: [0.0, 0.0, 0.0],
                },
                anneau: Anneau {
                    rayon: 46.0,
                    epaisseur: 8.0,
                    couleur: [1.0, 0.25, 0.2],
                    opacite: 1.0,
                },
                trainee: Trainee {
                    longueur: 280.0,
                    duree_fondu: 0.7,
                    rayon: 11.0,
                    couleur: [1.0, 0.6, 0.1],
                    opacite: 0.8,
                },
                ondulation: Ondulation {
                    rayon_max: 130.0,
                    duree: 0.9,
                    epaisseur: 6.0,
                    couleur_gauche: [1.0, 0.9, 0.2],
                    couleur_droit: [0.9, 0.2, 0.8],
                    opacite: 1.0,
                },
                son: SonClic {
                    volume: 0.8,
                    choix: Son::Cloche,
                },
            },
            actifs: ActifsProfil::default(),
        }
    }

    /// Repères fins et discrets, pour ne pas encombrer une vidéo.
    fn enregistrement() -> Profil {
        Profil {
            nom: NOM_ENREGISTREMENT.into(),
            raccourci: Some(raccourci('R')),
            effets: Effets {
                projecteur: Projecteur {
                    rayon: 100.0,
                    adoucissement: 70.0,
                    intensite: 0.25,
                    couleur: [0.0, 0.05, 0.2],
                },
                anneau: Anneau {
                    rayon: 20.0,
                    epaisseur: 2.0,
                    couleur: [0.2, 1.0, 0.6],
                    opacite: 0.8,
                },
                trainee: Trainee {
                    longueur: 90.0,
                    duree_fondu: 0.3,
                    rayon: 4.0,
                    couleur: [0.6, 0.4, 1.0],
                    opacite: 0.4,
                },
                ondulation: Ondulation {
                    rayon_max: 45.0,
                    duree: 0.45,
                    epaisseur: 2.0,
                    couleur_gauche: [0.4, 1.0, 0.8],
                    couleur_droit: [1.0, 0.7, 0.4],
                    opacite: 0.7,
                },
                son: SonClic {
                    volume: 0.2,
                    choix: Son::Bulle,
                },
            },
            actifs: ActifsProfil::default(),
        }
    }
}

/// Contenu du fichier des profils.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profils {
    pub version: u32,
    /// Indice du dernier profil utilisé, restauré au démarrage.
    pub actif: usize,
    /// Carte graphique voulue, prise en compte au prochain lancement.
    pub adaptateur: Adaptateur,
    /// Passe au profil suivant, en revenant au premier après le dernier.
    pub raccourci_suivant: Option<Raccourci>,
    /// Le premier est le profil par défaut.
    pub profils: Vec<Profil>,
}

impl Default for Profils {
    /// Contenu du premier lancement : le profil par défaut et deux exemples.
    fn default() -> Self {
        Profils {
            version: VERSION_FORMAT,
            actif: 0,
            adaptateur: Adaptateur::default(),
            raccourci_suivant: Some(raccourci('N')),
            profils: vec![
                Profil::default(),
                Profil::presentation(),
                Profil::enregistrement(),
            ],
        }
    }
}

/// Une lettre de A à Z, avec Ctrl ou Alt : les seules combinaisons que la plateforme sait
/// enregistrer, et qui ne prennent pas à une application une lettre qu'on tape.
fn touche_valide(r: &Raccourci) -> bool {
    r.touche.is_ascii_alphabetic() && (r.ctrl || r.alt)
}

/// Même combinaison, quelle que soit la casse de la lettre.
fn meme_raccourci(a: &Raccourci, b: &Raccourci) -> bool {
    (a.ctrl, a.alt, a.maj, a.touche.to_ascii_uppercase())
        == (b.ctrl, b.alt, b.maj, b.touche.to_ascii_uppercase())
}

impl Profils {
    pub fn profil_actif(&self) -> &Profil {
        &self.profils[self.actif]
    }

    /// Remet le contenu dans un état cohérent après lecture : ce qu'un fichier modifié à la main
    /// ou d'une autre version peut avoir cassé. Renvoie ce qui a été corrigé, une phrase par
    /// correction. Ne plante jamais.
    pub fn normaliser(&mut self) -> Vec<String> {
        let mut corrections = Vec::new();
        self.version = VERSION_FORMAT;
        if self.profils.is_empty() {
            corrections.push("aucun profil : profil par défaut recréé".to_string());
            self.profils.push(Profil::default());
        }
        if self.actif >= self.profils.len() {
            corrections.push(format!(
                "profil actif {} inexistant : retour au profil par défaut",
                self.actif
            ));
            self.actif = 0;
        }
        for i in 0..self.profils.len() {
            let brut = std::mem::take(&mut self.profils[i].nom);
            let mut nom: String = brut.trim().chars().take(LONGUEUR_NOM_MAX).collect();
            if nom.is_empty() {
                nom = format!("Profil {}", i + 1);
            }
            let base = nom.clone();
            let mut n = 2;
            while self.profils[..i]
                .iter()
                .any(|p| p.nom.to_lowercase() == nom.to_lowercase())
            {
                nom = format!("{base} ({n})");
                n += 1;
            }
            if nom != brut {
                corrections.push(format!("nom de profil « {brut} » remplacé par « {nom} »"));
            }
            self.profils[i].nom = nom;
            self.profils[i].effets = self.profils[i].effets.borne();
        }
        // Un raccourci inutilisable (touche inconnue, ou déjà pris par un effet, le raccourci
        // suivant ou un profil plus haut dans la liste) est retiré : le premier arrivé le garde.
        let mut pris: Vec<Raccourci> = Config::default().raccourcis_effets().to_vec();
        if let Some(r) = self.raccourci_suivant {
            if retenir(&r, &mut pris) {
                corrections.push("raccourci du profil suivant inutilisable : retiré".into());
                self.raccourci_suivant = None;
            }
        }
        for p in &mut self.profils {
            if let Some(r) = p.raccourci {
                if retenir(&r, &mut pris) {
                    corrections.push(format!(
                        "raccourci du profil « {} » inutilisable : retiré",
                        p.nom
                    ));
                    p.raccourci = None;
                }
            }
        }
        corrections
    }

    pub fn activer(&mut self, indice: usize) -> Result<(), String> {
        self.verifier_indice(indice)?;
        self.actif = indice;
        Ok(())
    }

    /// Indice du profil suivant, en revenant au premier après le dernier.
    pub fn indice_suivant(&self) -> usize {
        (self.actif + 1) % self.profils.len()
    }
}

/// Gestion des profils par l'utilisateur, depuis la fenêtre de réglages : créer, dupliquer,
/// renommer, supprimer, fixer un raccourci.
impl Profils {
    /// Nom prêt à servir pour le profil `sauf` (ou un nouveau si `None`) : espaces retirés, ni
    /// vide, ni trop long, ni déjà pris sans tenir compte de la casse.
    fn nom_valide(&self, nom: &str, sauf: Option<usize>) -> Result<String, String> {
        let nom = nom.trim();
        if nom.is_empty() {
            return Err("le nom d'un profil ne peut pas être vide".into());
        }
        if nom.chars().count() > LONGUEUR_NOM_MAX {
            return Err(format!(
                "le nom d'un profil est limité à {LONGUEUR_NOM_MAX} caractères"
            ));
        }
        let pris = self
            .profils
            .iter()
            .enumerate()
            .any(|(i, p)| Some(i) != sauf && p.nom.to_lowercase() == nom.to_lowercase());
        if pris {
            return Err(format!("un profil s'appelle déjà « {nom} »"));
        }
        Ok(nom.to_string())
    }

    fn verifier_indice(&self, indice: usize) -> Result<(), String> {
        if indice < self.profils.len() {
            Ok(())
        } else {
            Err(format!("le profil {indice} n'existe pas"))
        }
    }

    /// Crée un profil aux réglages par défaut, sans raccourci. Renvoie son indice.
    pub fn creer(&mut self, nom: &str) -> Result<usize, String> {
        let nom = self.nom_valide(nom, None)?;
        self.profils.push(Profil {
            nom,
            raccourci: None,
            effets: Effets::default(),
            actifs: ActifsProfil::default(),
        });
        Ok(self.profils.len() - 1)
    }

    /// Copie le profil `indice` sous « <nom> (copie) », sans raccourci : deux profils ne partagent
    /// pas une combinaison. Renvoie l'indice de la copie.
    pub fn dupliquer(&mut self, indice: usize) -> Result<usize, String> {
        self.verifier_indice(indice)?;
        // Le suffixe prend la place qu'il lui faut : c'est le nom d'origine qui est raccourci.
        let mut n = 1;
        let nom = loop {
            let fin = if n == 1 {
                " (copie)".to_string()
            } else {
                format!(" (copie) {n}")
            };
            let place = LONGUEUR_NOM_MAX - fin.chars().count();
            let debut: String = self.profils[indice].nom.chars().take(place).collect();
            let candidat = format!("{debut}{fin}");
            if self.nom_valide(&candidat, None).is_ok() {
                break candidat;
            }
            n += 1;
        };
        self.profils.push(Profil {
            nom,
            raccourci: None,
            effets: self.profils[indice].effets,
            actifs: self.profils[indice].actifs,
        });
        Ok(self.profils.len() - 1)
    }

    pub fn renommer(&mut self, indice: usize, nom: &str) -> Result<(), String> {
        self.verifier_indice(indice)?;
        self.profils[indice].nom = self.nom_valide(nom, Some(indice))?;
        Ok(())
    }

    /// Supprime un profil. Le profil par défaut (le premier) est refusé. Le profil actif suit : si
    /// c'est lui qui part, retour au profil par défaut.
    pub fn supprimer(&mut self, indice: usize) -> Result<(), String> {
        self.verifier_indice(indice)?;
        if indice == 0 {
            return Err("le profil par défaut ne peut pas être supprimé".into());
        }
        self.profils.remove(indice);
        if self.actif == indice {
            self.actif = 0;
        } else if self.actif > indice {
            self.actif -= 1;
        }
        Ok(())
    }

    /// Qui détient déjà cette combinaison, hors le profil `sauf` : un effet, le profil suivant ou
    /// un autre profil.
    pub fn detenteur(&self, r: &Raccourci, sauf: Option<usize>) -> Option<String> {
        if Config::default()
            .raccourcis_effets()
            .iter()
            .any(|e| meme_raccourci(e, r))
        {
            return Some("un raccourci d'effet".into());
        }
        if self
            .raccourci_suivant
            .as_ref()
            .is_some_and(|s| meme_raccourci(s, r))
        {
            return Some("le raccourci du profil suivant".into());
        }
        self.profils
            .iter()
            .enumerate()
            .find(|(i, p)| {
                Some(*i) != sauf && p.raccourci.as_ref().is_some_and(|x| meme_raccourci(x, r))
            })
            .map(|(_, p)| format!("le profil « {} »", p.nom))
    }

    /// Fixe (ou retire) le raccourci qui passe au profil suivant. Mêmes règles que pour un profil.
    pub fn definir_raccourci_suivant(&mut self, nouveau: Option<Raccourci>) -> Result<(), String> {
        let ancien = self.raccourci_suivant.take();
        let verdict = match &nouveau {
            Some(r) if !touche_valide(r) => {
                Err("une lettre de A à Z avec Ctrl ou Alt est nécessaire".to_string())
            }
            Some(r) => match self.detenteur(r, None) {
                Some(qui) => Err(format!("indisponible : déjà utilisé par {qui}")),
                None => Ok(()),
            },
            None => Ok(()),
        };
        self.raccourci_suivant = if verdict.is_ok() { nouveau } else { ancien };
        verdict
    }

    /// Fixe (ou retire) le raccourci d'un profil. Refuse une touche non prise en charge et une
    /// combinaison déjà détenue par ce programme ; un refus du système est un autre cas, traité à
    /// l'enregistrement.
    pub fn definir_raccourci(
        &mut self,
        indice: usize,
        nouveau: Option<Raccourci>,
    ) -> Result<(), String> {
        self.verifier_indice(indice)?;
        if let Some(r) = &nouveau {
            if !touche_valide(r) {
                return Err("une lettre de A à Z avec Ctrl ou Alt est nécessaire".into());
            }
            if let Some(qui) = self.detenteur(r, Some(indice)) {
                return Err(format!("indisponible : déjà utilisé par {qui}"));
            }
        }
        self.profils[indice].raccourci = nouveau;
        Ok(())
    }
}

/// Vrai si `r` est inutilisable (touche non prise en charge ou combinaison déjà dans `pris`) ;
/// sinon l'ajoute à `pris` et renvoie faux.
fn retenir(r: &Raccourci, pris: &mut Vec<Raccourci>) -> bool {
    if !touche_valide(r) || pris.iter().any(|p| meme_raccourci(p, r)) {
        return true;
    }
    pris.push(*r);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trois() -> Profils {
        Profils::default()
    }

    #[test]
    fn le_premier_lancement_donne_le_defaut_et_deux_exemples_coherents() {
        let mut p = trois();
        let noms: Vec<_> = p.profils.iter().map(|x| x.nom.as_str()).collect();
        assert_eq!(noms, ["Par défaut", "Présentation", "Enregistrement"]);
        assert_eq!((p.actif, p.version), (0, VERSION_FORMAT));
        // Déjà cohérent : la normalisation ne corrige rien.
        assert_eq!(p.normaliser(), Vec::<String>::new());
        assert_eq!(p, trois());
    }

    #[test]
    fn les_exemples_different_nettement_du_defaut_et_entre_eux() {
        let p = trois();
        let (d, pr, en) = (
            &p.profils[0].effets,
            &p.profils[1].effets,
            &p.profils[2].effets,
        );
        assert!(pr.anneau.rayon > 1.5 * d.anneau.rayon && d.anneau.rayon > 1.2 * en.anneau.rayon);
        assert!(pr.projecteur.rayon > d.projecteur.rayon);
        assert!(d.projecteur.rayon > en.projecteur.rayon);
        for (a, b) in [(d, pr), (d, en), (pr, en)] {
            assert_ne!(a.anneau.couleur, b.anneau.couleur);
            assert_ne!(a.trainee.couleur, b.trainee.couleur);
            assert_ne!(a.son.choix, b.son.choix);
        }
    }

    #[test]
    fn les_raccourcis_par_defaut_sont_tous_distincts_et_hors_des_effets() {
        let p = trois();
        let mut vus: Vec<Raccourci> = Config::default().raccourcis_effets().to_vec();
        vus.extend(p.raccourci_suivant);
        vus.extend(p.profils.iter().filter_map(|x| x.raccourci));
        assert_eq!(vus.len(), 5 + 1 + 3);
        for (i, a) in vus.iter().enumerate() {
            assert!(vus[i + 1..].iter().all(|b| !meme_raccourci(a, b)));
        }
    }

    #[test]
    fn le_defaut_ne_se_supprime_pas_les_autres_oui() {
        let mut p = trois();
        assert!(p.supprimer(0).is_err());
        assert_eq!(p.profils.len(), 3);
        p.supprimer(1).unwrap();
        assert_eq!(p.profils[1].nom, "Enregistrement");
        assert!(p.supprimer(5).is_err());
    }

    #[test]
    fn supprimer_garde_le_profil_actif_ou_revient_au_defaut() {
        let mut p = trois();
        p.activer(2).unwrap();
        p.supprimer(1).unwrap();
        assert_eq!(p.profil_actif().nom, "Enregistrement");
        p.supprimer(1).unwrap();
        assert_eq!((p.actif, p.profil_actif().nom.as_str()), (0, "Par défaut"));
    }

    #[test]
    fn creer_dupliquer_renommer_verifient_les_noms() {
        let mut p = trois();
        let i = p.creer("  Cours  ").unwrap();
        assert_eq!(p.profils[i].nom, "Cours");
        assert_eq!(p.profils[i].raccourci, None);
        assert!(p.creer("cours").is_err(), "casse ignorée");
        assert!(p.creer("   ").is_err());
        assert!(p.creer(&"x".repeat(LONGUEUR_NOM_MAX + 1)).is_err());
        let copie = p.dupliquer(1).unwrap();
        assert_eq!(p.profils[copie].nom, "Présentation (copie)");
        assert_eq!(p.profils[copie].effets, p.profils[1].effets);
        assert_eq!(p.profils[copie].raccourci, None);
        let autre = p.dupliquer(1).unwrap();
        assert_eq!(p.profils[autre].nom, "Présentation (copie) 2");
        p.renommer(i, "Atelier").unwrap();
        assert!(p.renommer(i, "Présentation").is_err());
        p.renommer(i, "atelier").unwrap();
        assert!(p.renommer(9, "x").is_err());
    }

    #[test]
    fn dupliquer_un_nom_de_longueur_maximale_reste_dans_la_limite() {
        let mut p = trois();
        let long = "a".repeat(LONGUEUR_NOM_MAX);
        let i = p.creer(&long).unwrap();
        let copie = p.dupliquer(i).unwrap();
        assert!(p.profils[copie].nom.chars().count() <= LONGUEUR_NOM_MAX);
    }

    #[test]
    fn suivant_fait_le_tour() {
        let mut p = trois();
        let mut tour = Vec::new();
        for _ in 0..3 {
            let i = p.indice_suivant();
            p.activer(i).unwrap();
            tour.push(i);
        }
        assert_eq!(tour, [1, 2, 0]);
    }

    #[test]
    fn un_raccourci_pris_est_refuse_avec_son_detenteur() {
        let mut p = trois();
        let err = p.definir_raccourci(2, Some(raccourci('P'))).unwrap_err();
        assert!(err.contains("raccourci d'effet"), "{err}");
        let err = p.definir_raccourci(2, Some(raccourci('e'))).unwrap_err();
        assert!(err.contains("Présentation"), "{err}");
        let err = p.definir_raccourci(2, Some(raccourci('N'))).unwrap_err();
        assert!(err.contains("suivant"), "{err}");
        assert!(p.definir_raccourci(2, Some(raccourci('1'))).is_err());
        // Garder sa propre combinaison est permis ; une combinaison libre aussi ; la retirer aussi.
        p.definir_raccourci(2, Some(raccourci('R'))).unwrap();
        p.definir_raccourci(2, Some(raccourci('K'))).unwrap();
        p.definir_raccourci(2, None).unwrap();
        assert_eq!(p.profils[2].raccourci, None);
    }

    #[test]
    fn normaliser_repare_un_fichier_modifie_a_la_main() {
        let mut p = trois();
        p.actif = 9;
        p.profils[1].nom = "  ".into();
        p.profils[2].nom = "par défaut".into();
        p.profils[1].raccourci = Some(raccourci('P'));
        p.profils[2].raccourci = Some(raccourci('D'));
        p.profils[0].effets.anneau.opacite = 7.0;
        let corrections = p.normaliser();
        assert_eq!(p.actif, 0);
        assert_eq!(p.profils[1].nom, "Profil 2");
        assert_eq!(p.profils[2].nom, "par défaut (2)");
        assert_eq!(p.profils[1].raccourci, None, "pris par un effet");
        assert_eq!(
            p.profils[2].raccourci, None,
            "pris par le profil par défaut"
        );
        assert_eq!(p.profils[0].effets.anneau.opacite, 1.0);
        assert_eq!(corrections.len(), 5, "{corrections:?}");
    }

    #[test]
    fn un_raccourci_sans_ctrl_ni_alt_est_refuse() {
        let mut p = trois();
        let nu = Raccourci {
            ctrl: false,
            alt: false,
            maj: true,
            touche: 'K',
        };
        assert!(p.definir_raccourci(2, Some(nu)).is_err());
        assert!(p.definir_raccourci_suivant(Some(nu)).is_err());
        let alt = Raccourci { alt: true, ..nu };
        p.definir_raccourci(2, Some(alt)).unwrap();
        p.profils[1].raccourci = Some(nu);
        p.normaliser();
        assert_eq!(p.profils[1].raccourci, None, "retiré à la lecture");
    }

    #[test]
    fn le_raccourci_du_profil_suivant_se_change_avec_les_memes_regles() {
        let mut p = trois();
        let err = p
            .definir_raccourci_suivant(Some(raccourci('E')))
            .unwrap_err();
        assert!(err.contains("indisponible : déjà utilisé par"), "{err}");
        assert_eq!(
            p.raccourci_suivant,
            Some(raccourci('N')),
            "inchangé après refus"
        );
        p.definir_raccourci_suivant(Some(raccourci('N'))).unwrap();
        p.definir_raccourci_suivant(Some(raccourci('K'))).unwrap();
        assert_eq!(p.raccourci_suivant, Some(raccourci('K')));
        p.definir_raccourci_suivant(None).unwrap();
        assert_eq!(p.raccourci_suivant, None);
    }

    #[test]
    fn l_activation_des_effets_est_dans_le_profil_et_se_duplique() {
        let mut p = trois();
        assert!(p
            .profils
            .iter()
            .all(|x| x.actifs == ActifsProfil::default()));
        p.profils[2].actifs.son = false;
        let copie = p.dupliquer(2).unwrap();
        assert!(!p.profils[copie].actifs.son);
        let neuf = p.creer("Neuf").unwrap();
        assert!(p.profils[neuf].actifs.son, "un profil neuf active tout");
    }

    #[test]
    fn normaliser_recree_le_defaut_si_la_liste_est_vide() {
        let mut p = trois();
        p.profils.clear();
        p.normaliser();
        assert_eq!(p.profils, vec![Profil::default()]);
        assert!(p.supprimer(0).is_err());
    }

    #[test]
    fn normaliser_retire_les_touches_non_prises_en_charge() {
        let mut p = trois();
        p.raccourci_suivant = Some(raccourci('é'));
        p.profils[0].raccourci = Some(raccourci('7'));
        p.normaliser();
        assert_eq!(p.raccourci_suivant, None);
        assert_eq!(p.profils[0].raccourci, None);
    }
}
