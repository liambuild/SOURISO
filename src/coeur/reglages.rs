//! Langage entre le processus principal et la fenêtre de réglages : une ligne de JSON par
//! message, dans les deux sens. Ici, rien de système : la table des paramètres (libellés, bornes,
//! unités), la lecture des demandes de la fenêtre, la modification d'un profil et l'état renvoyé.
//! Les bornes de la table sont la seule source : la page les reçoit, elle ne les copie pas.

use serde::Deserialize;
use serde_json::{json, Map, Value};

use super::activation::{self, nom_connu, ActifsProfil, NOMS};
use super::config::{Adaptateur, Effets, Raccourci};
use super::profil::{Profil, Profils};
use super::son::Son;

/// Version du langage : la page affiche une erreur si elle reçoit une autre version.
pub const VERSION_PROTOCOLE: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Genre {
    /// Nombre borné, réglé par un curseur.
    Nombre,
    /// Couleur « #rrggbb ».
    Couleur,
    /// Choix d'un son parmi `Son::TOUS`.
    Son,
}

/// Un réglage d'effet tel que la fenêtre le présente.
pub struct Parametre {
    /// Clé stable : `effet.champ`, par exemple `anneau.rayon`.
    pub chemin: &'static str,
    pub libelle: &'static str,
    pub aide: &'static str,
    pub genre: Genre,
    /// Bornes et pas, dans l'unité affichée.
    pub min: f64,
    pub max: f64,
    pub pas: f64,
    pub unite: &'static str,
    /// Valeur affichée = valeur du cœur × facteur (100 pour un pourcentage).
    pub facteur: f64,
}

const fn nombre(
    chemin: &'static str,
    libelle: &'static str,
    aide: &'static str,
    (min, max, pas): (f64, f64, f64),
    unite: &'static str,
    facteur: f64,
) -> Parametre {
    Parametre {
        chemin,
        libelle,
        aide,
        genre: Genre::Nombre,
        min,
        max,
        pas,
        unite,
        facteur,
    }
}

const fn couleur(chemin: &'static str, libelle: &'static str, aide: &'static str) -> Parametre {
    Parametre {
        chemin,
        libelle,
        aide,
        genre: Genre::Couleur,
        min: 0.0,
        max: 0.0,
        pas: 0.0,
        unite: "",
        facteur: 1.0,
    }
}

/// Un effet activable et sa présentation.
pub struct InfoEffet {
    pub id: &'static str,
    pub titre: &'static str,
    pub description: &'static str,
}

/// Dans l'ordre de `activation::NOMS`.
pub const EFFETS: [InfoEffet; 5] = [
    InfoEffet {
        id: activation::PROJECTEUR,
        titre: "Projecteur",
        description: "Assombrit l'écran, sauf autour du curseur.",
    },
    InfoEffet {
        id: activation::ANNEAU,
        titre: "Anneau",
        description: "Un anneau de couleur autour du curseur.",
    },
    InfoEffet {
        id: activation::TRAINEE,
        titre: "Traînée",
        description: "Une traînée qui s'efface derrière le curseur.",
    },
    InfoEffet {
        id: activation::ONDULATION,
        titre: "Ondulation",
        description: "Une onde à chaque clic : un anneau pour le clic gauche, deux pour le droit.",
    },
    InfoEffet {
        id: activation::SON,
        titre: "Son",
        description: "Un son à chaque clic, gauche ou droit.",
    },
];

/// Tous les réglages, groupés par effet, dans l'ordre de la page.
pub const PARAMETRES: &[Parametre] = &[
    nombre(
        "projecteur.rayon",
        "Rayon",
        "Taille de la zone entièrement éclairée autour du curseur.",
        (0.0, 600.0, 1.0),
        "px",
        1.0,
    ),
    nombre(
        "projecteur.adoucissement",
        "Adoucissement",
        "Largeur de la transition entre la zone éclairée et le voile.",
        (0.0, 400.0, 1.0),
        "px",
        1.0,
    ),
    nombre(
        "projecteur.intensite",
        "Opacité du voile",
        "Force de l'assombrissement loin du curseur.",
        (0.0, 100.0, 1.0),
        "%",
        100.0,
    ),
    couleur(
        "projecteur.couleur",
        "Couleur du voile",
        "Noir pour assombrir, une teinte pour colorer l'écran.",
    ),
    nombre(
        "anneau.rayon",
        "Rayon",
        "Taille de l'anneau, mesurée au milieu de son trait.",
        (1.0, 200.0, 1.0),
        "px",
        1.0,
    ),
    nombre(
        "anneau.epaisseur",
        "Épaisseur",
        "Épaisseur du trait.",
        (1.0, 40.0, 1.0),
        "px",
        1.0,
    ),
    nombre(
        "anneau.opacite",
        "Opacité",
        "De transparent à plein.",
        (0.0, 100.0, 1.0),
        "%",
        100.0,
    ),
    couleur("anneau.couleur", "Couleur", "Couleur de l'anneau."),
    nombre(
        "trainee.longueur",
        "Longueur",
        "Longueur maximale de la traînée derrière le curseur.",
        (1.0, 800.0, 1.0),
        "px",
        1.0,
    ),
    nombre(
        "trainee.duree_fondu",
        "Durée du fondu",
        "Temps que met un point à disparaître : plus il est court, plus la traînée est courte.",
        (0.05, 5.0, 0.05),
        "s",
        1.0,
    ),
    nombre(
        "trainee.rayon",
        "Taille des points",
        "Rayon des points au départ.",
        (1.0, 60.0, 1.0),
        "px",
        1.0,
    ),
    nombre(
        "trainee.opacite",
        "Opacité",
        "De transparent à plein.",
        (0.0, 100.0, 1.0),
        "%",
        100.0,
    ),
    couleur("trainee.couleur", "Couleur", "Couleur de la traînée."),
    nombre(
        "ondulation.rayon_max",
        "Rayon final",
        "Taille que l'onde atteint à la fin.",
        (1.0, 300.0, 1.0),
        "px",
        1.0,
    ),
    nombre(
        "ondulation.duree",
        "Durée",
        "Temps que dure l'onde.",
        (0.05, 3.0, 0.05),
        "s",
        1.0,
    ),
    nombre(
        "ondulation.epaisseur",
        "Épaisseur",
        "Épaisseur du trait de l'onde.",
        (1.0, 30.0, 1.0),
        "px",
        1.0,
    ),
    nombre(
        "ondulation.opacite",
        "Opacité",
        "De transparent à plein.",
        (0.0, 100.0, 1.0),
        "%",
        100.0,
    ),
    couleur(
        "ondulation.couleur_gauche",
        "Couleur du clic gauche",
        "Couleur de l'onde au clic gauche.",
    ),
    couleur(
        "ondulation.couleur_droit",
        "Couleur du clic droit",
        "Couleur des deux ondes au clic droit.",
    ),
    nombre(
        "son.volume",
        "Volume",
        "Niveau du son de clic.",
        (0.0, 100.0, 1.0),
        "%",
        100.0,
    ),
    Parametre {
        chemin: "son.choix",
        libelle: "Son",
        aide: "Le son joué à chaque clic.",
        genre: Genre::Son,
        min: 0.0,
        max: 0.0,
        pas: 0.0,
        unite: "",
        facteur: 1.0,
    },
];

/// Accès à un champ des effets par son chemin.
enum Champ<'a> {
    Nombre(&'a mut f32),
    Couleur(&'a mut [f32; 3]),
    Son(&'a mut Son),
}

fn champ<'a>(e: &'a mut Effets, chemin: &str) -> Option<Champ<'a>> {
    use Champ::{Couleur, Nombre};
    Some(match chemin {
        "projecteur.rayon" => Nombre(&mut e.projecteur.rayon),
        "projecteur.adoucissement" => Nombre(&mut e.projecteur.adoucissement),
        "projecteur.intensite" => Nombre(&mut e.projecteur.intensite),
        "projecteur.couleur" => Couleur(&mut e.projecteur.couleur),
        "anneau.rayon" => Nombre(&mut e.anneau.rayon),
        "anneau.epaisseur" => Nombre(&mut e.anneau.epaisseur),
        "anneau.opacite" => Nombre(&mut e.anneau.opacite),
        "anneau.couleur" => Couleur(&mut e.anneau.couleur),
        "trainee.longueur" => Nombre(&mut e.trainee.longueur),
        "trainee.duree_fondu" => Nombre(&mut e.trainee.duree_fondu),
        "trainee.rayon" => Nombre(&mut e.trainee.rayon),
        "trainee.opacite" => Nombre(&mut e.trainee.opacite),
        "trainee.couleur" => Couleur(&mut e.trainee.couleur),
        "ondulation.rayon_max" => Nombre(&mut e.ondulation.rayon_max),
        "ondulation.duree" => Nombre(&mut e.ondulation.duree),
        "ondulation.epaisseur" => Nombre(&mut e.ondulation.epaisseur),
        "ondulation.opacite" => Nombre(&mut e.ondulation.opacite),
        "ondulation.couleur_gauche" => Couleur(&mut e.ondulation.couleur_gauche),
        "ondulation.couleur_droit" => Couleur(&mut e.ondulation.couleur_droit),
        "son.volume" => Nombre(&mut e.son.volume),
        "son.choix" => Champ::Son(&mut e.son.choix),
        _ => return None,
    })
}

fn arrondi(x: f64) -> f64 {
    (x * 1e4).round() / 1e4
}

fn hex(c: [f32; 3]) -> String {
    let [r, v, b] = c.map(|x| (x.clamp(0.0, 1.0) * 255.0).round() as u8);
    format!("#{r:02x}{v:02x}{b:02x}")
}

fn depuis_hex(texte: &str) -> Result<[f32; 3], String> {
    let chiffres = texte
        .strip_prefix('#')
        .filter(|c| c.len() == 6 && c.is_ascii());
    let octet = |i: usize| {
        chiffres
            .and_then(|c| u8::from_str_radix(&c[i..i + 2], 16).ok())
            .map(|o| f32::from(o) / 255.0)
    };
    match (octet(0), octet(2), octet(4)) {
        (Some(r), Some(v), Some(b)) => Ok([r, v, b]),
        _ => Err(format!("couleur « {texte} » invalide (attendu #rrggbb)")),
    }
}

/// Valeur d'un réglage dans l'unité affichée (couleur en « #rrggbb », son par identifiant).
fn valeur(effets: &Effets, p: &Parametre) -> Value {
    let mut copie = *effets;
    match champ(&mut copie, p.chemin) {
        Some(Champ::Nombre(x)) => json!(arrondi(f64::from(*x) * p.facteur)),
        Some(Champ::Couleur(c)) => json!(hex(*c)),
        Some(Champ::Son(s)) => json!(s.id()),
        None => Value::Null,
    }
}

/// Ce qu'un réglage a changé, pour que l'application sache quoi mettre à jour.
#[derive(Debug, PartialEq)]
pub enum Reglage {
    /// Les réglages des effets du profil ont changé.
    Effets,
    /// L'activation d'un effet a changé dans le profil.
    Activation(&'static str, bool),
}

/// Applique un réglage reçu de la fenêtre au profil. `actif.<effet>` fixe l'activation de
/// l'effet dans le profil ; les autres chemins sont ceux de `PARAMETRES`. La valeur est ramenée
/// dans les bornes de la table puis dans celles du cœur.
pub fn regler(profil: &mut Profil, chemin: &str, valeur: &Value) -> Result<Reglage, String> {
    if let Some(effet) = chemin.strip_prefix("actif.") {
        let nom = nom_connu(effet).ok_or_else(|| format!("effet « {effet} » inconnu"))?;
        let actif = valeur
            .as_bool()
            .ok_or_else(|| format!("« {chemin} » attend vrai ou faux"))?;
        profil.actifs.definir(nom, actif);
        return Ok(Reglage::Activation(nom, actif));
    }
    let p = PARAMETRES
        .iter()
        .find(|p| p.chemin == chemin)
        .ok_or_else(|| format!("réglage « {chemin} » inconnu"))?;
    let champ = champ(&mut profil.effets, chemin).ok_or("réglage inconnu")?;
    match (champ, p.genre) {
        (Champ::Nombre(x), Genre::Nombre) => {
            let v = valeur
                .as_f64()
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("« {chemin} » attend un nombre"))?;
            *x = (v.clamp(p.min, p.max) / p.facteur) as f32;
        }
        (Champ::Couleur(c), Genre::Couleur) => {
            let texte = valeur
                .as_str()
                .ok_or_else(|| format!("« {chemin} » attend une couleur"))?;
            *c = depuis_hex(texte)?;
        }
        (Champ::Son(s), Genre::Son) => {
            *s = serde_json::from_value(valeur.clone())
                .map_err(|_| format!("son {valeur} inconnu"))?;
        }
        _ => return Err(format!("valeur de « {chemin} » de mauvais type")),
    }
    profil.effets = profil.effets.borne();
    Ok(Reglage::Effets)
}

/// Les réglages d'un profil au moment où la fenêtre s'ouvre ou où l'on change de profil : ce que
/// « Revenir aux valeurs du profil » rétablit.
#[derive(Clone, Debug, PartialEq)]
pub struct Base {
    pub effets: Effets,
    pub actifs: ActifsProfil,
}

impl Base {
    pub fn du_profil(profil: &Profil) -> Self {
        Base {
            effets: profil.effets,
            actifs: profil.actifs,
        }
    }

    /// Remet le profil dans l'état de la base. Renvoie faux s'il y était déjà.
    pub fn retablir(&self, profil: &mut Profil) -> bool {
        let change = profil.effets != self.effets || profil.actifs != self.actifs;
        profil.effets = self.effets;
        profil.actifs = self.actifs;
        change
    }
}

/// Sur quoi porte un raccourci à fixer.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CibleRaccourci {
    Profil(usize),
    /// Le texte « suivant » : le raccourci du profil suivant.
    Suivant(Suivant),
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Suivant {
    Suivant,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionProfil {
    Activer,
    Creer,
    Dupliquer,
    Renommer,
    Supprimer,
}

/// Ce que la fenêtre demande.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Demande {
    /// La page est prête à recevoir l'état.
    Pret,
    Regler {
        chemin: String,
        valeur: Value,
    },
    Revenir,
    Profil {
        action: ActionProfil,
        #[serde(default)]
        indice: usize,
        #[serde(default)]
        nom: String,
    },
    Raccourci {
        cible: CibleRaccourci,
        raccourci: Option<Raccourci>,
    },
    /// Début ou fin de saisie d'un raccourci : les raccourcis globaux sont suspendus entre les deux.
    Capture {
        debut: bool,
    },
    Adaptateur {
        valeur: Adaptateur,
    },
    /// Position et taille de la fenêtre, en pixels physiques du bureau.
    Fenetre {
        x: i32,
        y: i32,
        l: i32,
        h: i32,
        visible: bool,
    },
    /// La fenêtre se ferme.
    Fermee,
}

pub fn lire_demande(ligne: &str) -> Result<Demande, String> {
    serde_json::from_str(ligne).map_err(|e| format!("message illisible ({e}) : {ligne}"))
}

/// « Ctrl + Alt + Maj + P », comme la fenêtre l'affiche.
pub fn texte_raccourci(r: &Raccourci) -> String {
    let mut parties = Vec::new();
    if r.ctrl {
        parties.push("Ctrl".to_string());
    }
    if r.alt {
        parties.push("Alt".to_string());
    }
    if r.maj {
        parties.push("Maj".to_string());
    }
    parties.push(r.touche.to_ascii_uppercase().to_string());
    parties.join(" + ")
}

fn json_raccourci(r: &Option<Raccourci>) -> Value {
    r.as_ref().map_or(Value::Null, |r| {
        json!({"ctrl": r.ctrl, "alt": r.alt, "maj": r.maj,
               "touche": r.touche.to_ascii_uppercase().to_string(), "texte": texte_raccourci(r)})
    })
}

fn json_valeurs(effets: &Effets) -> Value {
    let mut m = Map::new();
    for p in PARAMETRES {
        m.insert(p.chemin.into(), valeur(effets, p));
    }
    Value::Object(m)
}

fn json_actifs(actifs: &ActifsProfil) -> Value {
    let mut m = Map::new();
    for nom in NOMS {
        m.insert(nom.into(), json!(actifs.est_actif(nom)));
    }
    Value::Object(m)
}

/// Variable d'environnement qui impose la carte graphique, s'il y en a une : elle l'emporte sur
/// le réglage du fichier (`docs/ARCHITECTURE.md`, 4.2).
pub fn adaptateur_impose() -> Option<&'static str> {
    ["WGPU_ADAPTER_NAME", "WGPU_POWER_PREF"]
        .into_iter()
        .find(|v| std::env::var(v).is_ok_and(|x| !x.trim().is_empty()))
}

/// Faute à montrer à côté d'un élément de la page.
#[derive(Clone, Debug, PartialEq)]
pub struct Faute {
    /// Où l'afficher : `profil:<indice>`, `suivant`, `nouveau`, `renommer:<indice>`.
    pub cible: String,
    pub message: String,
}

/// État complet envoyé à la fenêtre. `live` est l'activation en cours (session) du profil actif ;
/// elle diffère de celle du profil quand un raccourci d'effet a été pressé. `complet` ajoute la
/// description des effets et des réglages, utile une seule fois.
pub fn etat(
    profils: &Profils,
    live: &ActifsProfil,
    base: &Base,
    faute: Option<&Faute>,
    complet: bool,
) -> Value {
    let mut v = json!({
        "type": "etat",
        "version": VERSION_PROTOCOLE,
        "actif": profils.actif,
        "live": json_actifs(live),
        "base": {"valeurs": json_valeurs(&base.effets), "actifs": json_actifs(&base.actifs)},
        "suivant": json_raccourci(&profils.raccourci_suivant),
        "adaptateur": profils.adaptateur,
        "adaptateur_impose": adaptateur_impose(),
        "profils": profils.profils.iter().map(|p| json!({
            "nom": p.nom,
            "raccourci": json_raccourci(&p.raccourci),
            "valeurs": json_valeurs(&p.effets),
            "actifs": json_actifs(&p.actifs),
        })).collect::<Vec<_>>(),
        "faute": faute.map(|f| json!({"cible": f.cible, "message": f.message})),
    });
    if complet {
        v["effets"] = EFFETS
            .iter()
            .zip(crate::coeur::Config::default().raccourcis_effets())
            .map(|(e, r)| {
                json!({"id": e.id, "titre": e.titre, "description": e.description,
                       "raccourci": texte_raccourci(&r)})
            })
            .collect();
        v["parametres"] = PARAMETRES
            .iter()
            .map(|p| {
                let effet = p.chemin.split('.').next().unwrap_or_default();
                let mut j = json!({
                    "chemin": p.chemin, "effet": effet, "libelle": p.libelle, "aide": p.aide,
                    "genre": match p.genre {
                        Genre::Nombre => "nombre",
                        Genre::Couleur => "couleur",
                        Genre::Son => "son",
                    },
                });
                if p.genre == Genre::Nombre {
                    j["min"] = json!(p.min);
                    j["max"] = json!(p.max);
                    j["pas"] = json!(p.pas);
                    j["unite"] = json!(p.unite);
                }
                if p.genre == Genre::Son {
                    j["choix"] = Son::TOUS
                        .iter()
                        .map(|s| json!({"id": s.id(), "nom": s.libelle()}))
                        .collect();
                }
                j
            })
            .collect();
    }
    v
}

/// Message d'activation d'un effet (raccourci pressé pendant que la fenêtre est ouverte).
pub fn message_activation(effet: &str, actif: bool) -> Value {
    json!({"type": "activation", "effet": effet, "actif": actif})
}

/// Erreur sans rapport avec un élément précis (message illisible, processus qui échoue).
pub fn message_erreur(message: &str) -> Value {
    json!({"type": "erreur", "message": message})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coeur::config::raccourci;

    fn profil() -> Profil {
        Profil::default()
    }

    #[test]
    fn chaque_parametre_a_un_champ_et_chaque_champ_un_parametre() {
        let mut e = Effets::default();
        for p in PARAMETRES {
            assert!(champ(&mut e, p.chemin).is_some(), "{}", p.chemin);
        }
        let mut chemins: Vec<_> = PARAMETRES.iter().map(|p| p.chemin).collect();
        chemins.sort_unstable();
        chemins.dedup();
        assert_eq!(chemins.len(), PARAMETRES.len(), "chemins en double");
        assert_eq!(PARAMETRES.len(), 21);
        assert_eq!(EFFETS.map(|e| e.id), NOMS);
    }

    #[test]
    fn les_bornes_de_la_table_sont_dans_celles_du_coeur() {
        // Régler au minimum puis au maximum de la table ne doit rien changer à ce que le cœur
        // garde : sinon le curseur annoncerait une valeur que le rendu ne prend pas.
        for p in PARAMETRES.iter().filter(|p| p.genre == Genre::Nombre) {
            for v in [p.min, p.max] {
                let mut pr = profil();
                regler(&mut pr, p.chemin, &json!(v)).unwrap();
                let relu = valeur(&pr.effets, p).as_f64().unwrap();
                assert!((relu - v).abs() < 1e-6, "{} : {v} → {relu}", p.chemin);
            }
        }
    }

    #[test]
    fn les_profils_d_exemple_sont_dans_les_bornes_de_la_table() {
        for pr in &Profils::default().profils {
            for p in PARAMETRES.iter().filter(|p| p.genre == Genre::Nombre) {
                let v = valeur(&pr.effets, p).as_f64().unwrap();
                assert!(
                    (p.min..=p.max).contains(&v),
                    "{} / {} : {v}",
                    pr.nom,
                    p.chemin
                );
            }
        }
    }

    #[test]
    fn regler_convertit_les_pourcentages_et_borne() {
        let mut pr = profil();
        assert_eq!(
            regler(&mut pr, "anneau.opacite", &json!(40)),
            Ok(Reglage::Effets)
        );
        assert!((pr.effets.anneau.opacite - 0.4).abs() < 1e-6);
        regler(&mut pr, "anneau.rayon", &json!(99999)).unwrap();
        assert_eq!(pr.effets.anneau.rayon, 200.0, "borne de la table");
        regler(&mut pr, "trainee.duree_fondu", &json!(-4)).unwrap();
        assert_eq!(pr.effets.trainee.duree_fondu, 0.05);
        regler(&mut pr, "son.volume", &json!(100)).unwrap();
        assert_eq!(pr.effets.son.volume, 1.0);
    }

    #[test]
    fn regler_refuse_ce_qui_n_a_pas_de_sens_sans_rien_changer() {
        let mut pr = profil();
        let avant = pr.clone();
        for (chemin, v) in [
            ("inconnu.rayon", json!(3)),
            ("anneau.rayon", json!("grand")),
            ("anneau.rayon", Value::Null),
            ("anneau.couleur", json!("rouge")),
            ("anneau.couleur", json!("#12345")),
            ("anneau.couleur", json!("#12é456")),
            ("anneau.couleur", json!(5)),
            ("son.choix", json!("trompette")),
            ("son.volume", json!(true)),
            ("actif.inconnu", json!(true)),
            ("actif.son", json!("oui")),
        ] {
            assert!(regler(&mut pr, chemin, &v).is_err(), "{chemin} {v}");
        }
        assert_eq!(pr, avant);
    }

    #[test]
    fn les_couleurs_vont_et_viennent_en_hexadecimal() {
        let mut pr = profil();
        regler(&mut pr, "projecteur.couleur", &json!("#ff8000")).unwrap();
        assert_eq!(hex(pr.effets.projecteur.couleur), "#ff8000");
        let p = PARAMETRES
            .iter()
            .find(|p| p.chemin == "projecteur.couleur")
            .unwrap();
        assert_eq!(valeur(&pr.effets, p), json!("#ff8000"));
        regler(&mut pr, "son.choix", &json!("cloche")).unwrap();
        assert_eq!(pr.effets.son.choix, Son::Cloche);
    }

    #[test]
    fn l_interrupteur_ecrit_dans_le_profil() {
        let mut pr = profil();
        assert_eq!(
            regler(&mut pr, "actif.son", &json!(false)),
            Ok(Reglage::Activation(activation::SON, false))
        );
        assert!(!pr.actifs.son && pr.actifs.anneau);
        regler(&mut pr, "actif.son", &json!(true)).unwrap();
        assert!(pr.actifs.son);
    }

    #[test]
    fn revenir_rétablit_la_base_et_dit_si_quelque_chose_a_change() {
        let mut pr = profil();
        let base = Base::du_profil(&pr);
        assert!(!base.retablir(&mut pr), "rien n'a changé");
        regler(&mut pr, "anneau.rayon", &json!(90)).unwrap();
        regler(&mut pr, "actif.trainee", &json!(false)).unwrap();
        assert!(base.retablir(&mut pr));
        assert_eq!(pr, profil());
    }

    #[test]
    fn les_demandes_se_lisent() {
        let ok = |l: &str| lire_demande(l).unwrap();
        assert_eq!(ok(r#"{"type":"pret"}"#), Demande::Pret);
        assert_eq!(
            ok(r#"{"type":"regler","chemin":"anneau.rayon","valeur":30}"#),
            Demande::Regler {
                chemin: "anneau.rayon".into(),
                valeur: json!(30)
            }
        );
        assert_eq!(
            ok(r#"{"type":"profil","action":"renommer","indice":2,"nom":"Cours"}"#),
            Demande::Profil {
                action: ActionProfil::Renommer,
                indice: 2,
                nom: "Cours".into()
            }
        );
        assert_eq!(
            ok(r#"{"type":"profil","action":"creer","nom":"Neuf"}"#),
            Demande::Profil {
                action: ActionProfil::Creer,
                indice: 0,
                nom: "Neuf".into()
            }
        );
        assert_eq!(
            ok(r#"{"type":"raccourci","cible":"suivant","raccourci":null}"#),
            Demande::Raccourci {
                cible: CibleRaccourci::Suivant(Suivant::Suivant),
                raccourci: None
            }
        );
        assert_eq!(
            ok(r#"{"type":"raccourci","cible":1,"raccourci":
                {"ctrl":true,"alt":false,"maj":false,"touche":"K"}}"#),
            Demande::Raccourci {
                cible: CibleRaccourci::Profil(1),
                raccourci: Some(Raccourci {
                    ctrl: true,
                    alt: false,
                    maj: false,
                    touche: 'K'
                })
            }
        );
        assert_eq!(
            ok(r#"{"type":"adaptateur","valeur":"performant"}"#),
            Demande::Adaptateur {
                valeur: Adaptateur::Performant
            }
        );
        assert_eq!(
            ok(r#"{"type":"fenetre","x":-10,"y":5,"l":880,"h":600,"visible":true}"#),
            Demande::Fenetre {
                x: -10,
                y: 5,
                l: 880,
                h: 600,
                visible: true
            }
        );
        assert_eq!(
            ok(r#"{"type":"capture","debut":true}"#),
            Demande::Capture { debut: true }
        );
        assert_eq!(ok(r#"{"type":"fermee"}"#), Demande::Fermee);
        for mauvais in [
            "",
            "pas du json",
            r#"{"type":"inconnu"}"#,
            r#"{"type":"regler"}"#,
            r#"{"type":"profil","action":"detruire"}"#,
            r#"{"type":"raccourci","cible":"autre","raccourci":null}"#,
        ] {
            assert!(lire_demande(mauvais).is_err(), "{mauvais}");
        }
    }

    #[test]
    fn les_raccourcis_s_ecrivent_comme_la_fenetre_les_affiche() {
        assert_eq!(texte_raccourci(&raccourci('p')), "Ctrl + Alt + Maj + P");
        let r = Raccourci {
            ctrl: false,
            alt: true,
            maj: false,
            touche: 'k',
        };
        assert_eq!(texte_raccourci(&r), "Alt + K");
    }

    #[test]
    fn l_etat_decrit_les_profils_et_une_seule_fois_les_reglages() {
        let mut p = Profils::default();
        p.profils[1].actifs.son = false;
        let base = Base::du_profil(p.profil_actif());
        let live = ActifsProfil {
            anneau: false,
            ..ActifsProfil::default()
        };
        let faute = Faute {
            cible: "profil:1".into(),
            message: "indisponible : déjà utilisé par le profil « X »".into(),
        };
        let v = etat(&p, &live, &base, Some(&faute), true);
        assert_eq!(v["type"], "etat");
        assert_eq!(v["profils"].as_array().unwrap().len(), 3);
        assert_eq!(v["profils"][1]["nom"], "Présentation");
        assert_eq!(v["profils"][1]["actifs"]["son"], false);
        assert_eq!(
            v["profils"][0]["raccourci"]["texte"],
            "Ctrl + Alt + Maj + D"
        );
        assert_eq!(v["live"]["anneau"], false);
        assert_eq!(
            v["base"]["actifs"]["anneau"], true,
            "la base n'est pas la session"
        );
        assert_eq!(v["suivant"]["texte"], "Ctrl + Alt + Maj + N");
        assert_eq!(v["adaptateur"], "automatique");
        assert_eq!(v["faute"]["cible"], "profil:1");
        assert_eq!(v["profils"][0]["valeurs"]["anneau.opacite"], 90.0);
        assert_eq!(v["profils"][0]["valeurs"]["anneau.couleur"], "#ffcc1a");
        assert_eq!(v["parametres"].as_array().unwrap().len(), PARAMETRES.len());
        assert_eq!(v["effets"][4]["raccourci"], "Ctrl + Alt + Maj + S");
        let son = v["parametres"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["chemin"] == "son.choix")
            .unwrap();
        assert_eq!(son["choix"].as_array().unwrap().len(), 3);
        let court = etat(&p, &live, &base, None, false);
        assert!(court.get("parametres").is_none() && court.get("effets").is_none());
        assert_eq!(court["faute"], Value::Null);
    }

    /// Outil de mise au point : `SOURISO_ETAT_EXEMPLE=chemin cargo test ecrire_etat_exemple`
    /// écrit l'état complet du premier lancement, pour essayer la page sans SOURISO.
    #[test]
    fn ecrire_etat_exemple_si_demande() {
        let Some(chemin) = std::env::var_os("SOURISO_ETAT_EXEMPLE") else {
            return;
        };
        let p = Profils::default();
        let base = Base::du_profil(p.profil_actif());
        let v = etat(&p, &ActifsProfil::default(), &base, None, true);
        std::fs::write(chemin, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
    }
}
