//! Générateur des sons de clic : écrit `assets/sons/*.wav` (PCM 16 bits, mono, 44 100 Hz).
//! Les sons sont synthétisés ici, sans échantillon ni source externe, et le calcul est déterministe :
//! relancer le générateur redonne les mêmes octets. Origine et licence : `docs/SONS.md`.
//!
//! Usage : `cargo run --example generer_sons` depuis la racine du dépôt.

use std::f64::consts::TAU;
use std::fs;
use std::path::Path;

const FREQUENCE: u32 = 44_100;
/// Crête des sons, en fraction de la pleine échelle : le volume réel se règle dans la configuration.
const CRETE: f64 = 0.8;

/// Un son : durée en secondes et fonction qui donne l'échantillon (de -1 à 1) à l'instant `t`.
struct Son {
    nom: &'static str,
    duree: f64,
    echantillon: fn(f64) -> f64,
}

/// Tic sec : une sinusoïde aiguë et sa première harmonique, éteintes en quelques millisecondes.
fn clic(t: f64) -> f64 {
    let enveloppe = (-t / 0.006).exp();
    enveloppe * ((TAU * 1800.0 * t).sin() + 0.4 * (TAU * 3600.0 * t).sin())
}

/// Bulle : une sinusoïde dont la hauteur monte de 500 à 1 100 Hz en 80 ms.
fn bulle(t: f64) -> f64 {
    let montee = (t / 0.08).min(1.0);
    let frequence = 500.0 + 600.0 * montee;
    // Phase de la fréquence variable : intégrale de la fréquence.
    let phase = if t < 0.08 {
        500.0 * t + 600.0 * t * t / (2.0 * 0.08)
    } else {
        500.0 * 0.08 + 300.0 * 0.08 + frequence * (t - 0.08)
    };
    (-t / 0.05).exp() * (TAU * phase).sin()
}

/// Clochette : un son fondamental et un partiel inharmonique, éteints en un quart de seconde.
fn cloche(t: f64) -> f64 {
    (-t / 0.12).exp() * ((TAU * 880.0 * t).sin() + 0.5 * (TAU * 2425.0 * t).sin())
}

/// Fichier WAV : en-tête de 44 octets (RIFF, `fmt `, `data`) puis les échantillons.
fn wav(son: &Son) -> Vec<u8> {
    let nombre = (son.duree * f64::from(FREQUENCE)) as usize;
    let brut: Vec<f64> = (0..nombre)
        .map(|i| {
            let t = i as f64 / f64::from(FREQUENCE);
            // Entrée sur 1 ms et sortie sur les 5 dernières ms : aucun claquement aux extrémités.
            let entree = (t / 0.001).min(1.0);
            let sortie = ((son.duree - t) / 0.005).min(1.0);
            (son.echantillon)(t) * entree * sortie
        })
        .collect();
    let maximum = brut.iter().fold(f64::MIN_POSITIVE, |m, x| m.max(x.abs()));
    let octets_donnees = (nombre * 2) as u32;

    let mut sortie = Vec::with_capacity(44 + nombre * 2);
    sortie.extend_from_slice(b"RIFF");
    sortie.extend_from_slice(&(36 + octets_donnees).to_le_bytes());
    sortie.extend_from_slice(b"WAVEfmt ");
    sortie.extend_from_slice(&16u32.to_le_bytes());
    sortie.extend_from_slice(&1u16.to_le_bytes()); // PCM
    sortie.extend_from_slice(&1u16.to_le_bytes()); // mono
    sortie.extend_from_slice(&FREQUENCE.to_le_bytes());
    sortie.extend_from_slice(&(FREQUENCE * 2).to_le_bytes());
    sortie.extend_from_slice(&2u16.to_le_bytes());
    sortie.extend_from_slice(&16u16.to_le_bytes());
    sortie.extend_from_slice(b"data");
    sortie.extend_from_slice(&octets_donnees.to_le_bytes());
    for x in brut {
        let valeur = (x / maximum * CRETE * f64::from(i16::MAX)).round() as i16;
        sortie.extend_from_slice(&valeur.to_le_bytes());
    }
    sortie
}

fn main() -> std::io::Result<()> {
    let dossier = Path::new("assets/sons");
    fs::create_dir_all(dossier)?;
    let sons = [
        Son {
            nom: "clic",
            duree: 0.060,
            echantillon: clic,
        },
        Son {
            nom: "bulle",
            duree: 0.140,
            echantillon: bulle,
        },
        Son {
            nom: "cloche",
            duree: 0.450,
            echantillon: cloche,
        },
    ];
    for son in &sons {
        let chemin = dossier.join(format!("{}.wav", son.nom));
        let octets = wav(son);
        fs::write(&chemin, &octets)?;
        println!("{} : {} octets", chemin.display(), octets.len());
    }
    Ok(())
}
