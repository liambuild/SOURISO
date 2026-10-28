//! Contrôles de la page embarquée (`ui/`) qui ne demandent ni écran ni moteur web : contraste des
//! couleurs, absence de toute ressource distante. Les parcours au clavier et à l'écran sont dans
//! la recette (`docs/RECETTE.md`).

use std::collections::HashMap;

const INDEX: &str = include_str!("../ui/index.html");
const STYLE: &str = include_str!("../ui/style.css");
const SCRIPT: &str = include_str!("../ui/app.js");

type Couleur = [f64; 3];
type Jetons = HashMap<String, Couleur>;

/// Les jetons `--nom: #rrggbb;` d'un morceau de feuille de style.
fn jetons(css: &str) -> Jetons {
    let mut trouves = HashMap::new();
    for (debut, _) in css.match_indices("--") {
        let reste = &css[debut + 2..];
        let fin_nom = reste
            .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
            .unwrap_or(0);
        let (nom, suite) = reste.split_at(fin_nom);
        let Some(valeur) = suite.strip_prefix(':').map(str::trim_start) else {
            continue;
        };
        let Some(hex) = valeur.strip_prefix('#').and_then(|v| v.get(..6)) else {
            continue;
        };
        if let Ok(n) = u32::from_str_radix(hex, 16) {
            let c = |decalage: u32| f64::from((n >> decalage) & 0xff);
            trouves.insert(nom.to_string(), [c(16), c(8), c(0)]);
        }
    }
    trouves
}

/// Jetons du thème clair, puis du thème sombre (les jetons clairs, remplacés par ceux du bloc
/// `prefers-color-scheme: dark`).
fn themes() -> (Jetons, Jetons) {
    let (clair, sombre) = STYLE
        .split_once("prefers-color-scheme: dark")
        .expect("bloc du thème sombre");
    let clair = jetons(clair);
    let mut sombre_complet = clair.clone();
    sombre_complet.extend(jetons(sombre));
    (clair, sombre_complet)
}

fn luminance(c: Couleur) -> f64 {
    let lin = |x: f64| {
        let x = x / 255.0;
        if x <= 0.03928 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

/// Rapport de contraste WCAG entre deux couleurs.
fn contraste(a: Couleur, b: Couleur) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Texte sur son fond : au moins 4,5:1 (WCAG 1.4.3).
const TEXTE: &[(&str, &str)] = &[
    ("texte", "fond"),
    ("texte", "surface"),
    ("texte", "accent-doux"),
    ("texte-2", "fond"),
    ("texte-2", "surface"),
    ("texte-2", "accent-doux"),
    ("accent", "surface"),
    ("accent", "fond"),
    ("sur-accent", "accent"),
    ("danger", "surface"),
    ("danger", "fond"),
];

/// Contour des contrôles, bascule, anneau de focus : au moins 3:1 (WCAG 1.4.11).
const COMPOSANTS: &[(&str, &str)] = &[
    ("contour", "surface"),
    ("contour", "fond"),
    ("accent", "surface"),
    ("accent", "fond"),
    ("accent", "accent-doux"),
];

fn verifier(paires: &[(&str, &str)], minimum: f64) {
    let (clair, sombre) = themes();
    for (nom, theme) in [("clair", clair), ("sombre", sombre)] {
        for (a, b) in paires {
            let (ca, cb) = (theme[*a], theme[*b]);
            let rapport = contraste(ca, cb);
            assert!(
                rapport >= minimum,
                "thème {nom} : --{a} sur --{b} = {rapport:.2}:1, attendu au moins {minimum}:1"
            );
        }
    }
}

#[test]
fn les_deux_themes_definissent_les_memes_jetons() {
    let (clair, sombre) = themes();
    let mut a: Vec<_> = clair.keys().collect();
    let mut b: Vec<_> = sombre.keys().collect();
    a.sort();
    b.sort();
    assert_eq!(a, b);
    // Chaque jeton du thème clair est redéfini pour le sombre (sinon une couleur claire resterait
    // sur un fond sombre).
    let (avant, apres) = STYLE.split_once("prefers-color-scheme: dark").unwrap();
    let redefinis = jetons(apres);
    for nom in jetons(avant).keys() {
        assert!(
            redefinis.contains_key(nom),
            "--{nom} n'est pas redéfini en sombre"
        );
    }
}

#[test]
fn le_texte_a_un_contraste_d_au_moins_4_5_dans_les_deux_themes() {
    verifier(TEXTE, 4.5);
}

#[test]
fn les_contours_et_le_focus_ont_un_contraste_d_au_moins_3_dans_les_deux_themes() {
    verifier(COMPOSANTS, 3.0);
}

#[test]
fn le_calcul_de_contraste_donne_les_valeurs_connues() {
    assert!((contraste([0.0; 3], [255.0; 3]) - 21.0).abs() < 1e-9);
    assert!((contraste([255.0; 3], [255.0; 3]) - 1.0).abs() < 1e-9);
    // Gris #767676 sur blanc : la limite classique de 4,5:1.
    assert!((contraste([118.0; 3], [255.0; 3]) - 4.54).abs() < 0.01);
}

#[test]
fn la_page_ne_demande_aucune_ressource_distante() {
    for (nom, contenu) in [
        ("index.html", INDEX),
        ("style.css", STYLE),
        ("app.js", SCRIPT),
    ] {
        for interdit in [
            "http://",
            "https://",
            "//cdn",
            "@import",
            "url(",
            "fetch(",
            "XMLHttpRequest",
            "WebSocket",
            "eval(",
        ] {
            assert!(!contenu.contains(interdit), "{nom} contient « {interdit} »");
        }
    }
    // Les seuls fichiers chargés sont ceux du dossier.
    assert!(INDEX.contains(r#"href="style.css""#) && INDEX.contains(r#"src="app.js""#));
    assert_eq!(
        INDEX.matches("src=").count() + INDEX.matches("href=\"style").count(),
        2
    );
}

#[test]
fn la_page_n_utilise_ni_style_ni_script_en_ligne() {
    assert!(!INDEX.contains("<style") && !INDEX.contains("style="));
    assert!(!INDEX.contains("onclick") && !INDEX.contains("javascript:"));
    // La politique de sécurité du crate l'interdit de toute façon.
    let conf = include_str!("../tauri.conf.json");
    assert!(conf.contains("default-src 'self'") && !conf.contains("-inline"));
}
