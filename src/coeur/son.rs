//! Sons de clic : les trois sons embarqués et leur mise au volume. Les fichiers sont produits par
//! `examples/generer_sons.rs` ; origine et licence dans `docs/SONS.md`.

use serde::{Deserialize, Serialize};

/// Son joué au clic, au choix du profil.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Son {
    /// Tic sec.
    Clic,
    /// Bulle qui monte.
    Bulle,
    /// Clochette.
    Cloche,
}

/// Taille de l'en-tête des fichiers WAV du générateur : RIFF, `fmt ` et début de `data`.
const ENTETE: usize = 44;

impl Son {
    /// Fichier WAV embarqué (PCM 16 bits, mono).
    pub fn wav(self) -> &'static [u8] {
        match self {
            Son::Clic => include_bytes!("../../assets/sons/clic.wav"),
            Son::Bulle => include_bytes!("../../assets/sons/bulle.wav"),
            Son::Cloche => include_bytes!("../../assets/sons/cloche.wav"),
        }
    }
}

/// Copie du WAV dont les échantillons sont multipliés par `volume` (de 0 à 1). Le volume se règle
/// ici car la lecture à la demande de la plateforme n'en offre pas.
pub fn avec_volume(wav: &[u8], volume: f32) -> Vec<u8> {
    let volume = volume.clamp(0.0, 1.0);
    let mut sortie = wav.to_vec();
    for paire in sortie[ENTETE.min(wav.len())..].as_chunks_mut::<2>().0 {
        let echantillon = i16::from_le_bytes(*paire);
        let regle = (f32::from(echantillon) * volume).round() as i16;
        *paire = regle.to_le_bytes();
    }
    sortie
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOUS: [Son; 3] = [Son::Clic, Son::Bulle, Son::Cloche];

    #[test]
    fn les_sons_embarques_sont_des_wav_pcm_16_bits_mono() {
        for son in TOUS {
            let wav = son.wav();
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(&wav[8..16], b"WAVEfmt ");
            assert_eq!(u16::from_le_bytes([wav[20], wav[21]]), 1, "PCM");
            assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1, "mono");
            assert_eq!(u16::from_le_bytes([wav[34], wav[35]]), 16, "16 bits");
            let donnees = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]) as usize;
            assert_eq!(wav.len(), ENTETE + donnees);
        }
    }

    #[test]
    fn le_volume_un_ne_change_rien() {
        let wav = Son::Clic.wav();
        assert_eq!(avec_volume(wav, 1.0), wav);
    }

    #[test]
    fn le_volume_zero_rend_le_silence_sans_toucher_l_entete() {
        let wav = Son::Bulle.wav();
        let muet = avec_volume(wav, 0.0);
        assert_eq!(&muet[..ENTETE], &wav[..ENTETE]);
        assert!(muet[ENTETE..].iter().all(|o| *o == 0));
    }

    #[test]
    fn le_volume_est_borne_et_reduit_la_crete() {
        let wav = Son::Cloche.wav();
        let crete = |w: &[u8]| {
            w[ENTETE..]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| i16::from_le_bytes(*p).unsigned_abs())
                .max()
                .unwrap()
        };
        assert_eq!(crete(&avec_volume(wav, 5.0)), crete(wav));
        let moitie = crete(&avec_volume(wav, 0.5));
        assert!((i32::from(moitie) - i32::from(crete(wav)) / 2).abs() <= 1);
    }
}
