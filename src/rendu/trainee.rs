//! Effet traînée : des disques doux le long du chemin récent du curseur, qui s'effacent. Un
//! disque est un carré instancié ; la carte ne remplit que ces carrés.

use std::time::Duration;

use bytemuck::{Pod, Zeroable};

use crate::coeur::activation::TRAINEE;
use crate::coeur::config;
use crate::coeur::trainee::{Trainee as Etat, MAX_POINTS};

use super::commun::Passe;
use super::effet::{Contexte, Effet, Sortie};

/// Part de l'opacité réglée que porte chaque disque : ils se recouvrent, leurs opacités s'ajoutent.
const PART_PAR_DISQUE: f32 = 0.35;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct Uniforme {
    couleur: [f32; 3],
    non_premultiplie: f32,
    taille: [f32; 2],
    _remplissage: [f32; 2],
    /// x, y (pixels), rayon (pixels), opacité.
    points: [[f32; 4]; MAX_POINTS],
}

pub struct Trainee {
    reglages: config::Trainee,
    non_premultiplie: bool,
    etat: Etat,
    passe: Passe,
    disques: u32,
}

impl Trainee {
    pub fn nouveau(device: &wgpu::Device, sortie: &Sortie, reglages: config::Trainee) -> Self {
        Trainee {
            reglages: reglages.borne(),
            non_premultiplie: sortie.non_premultiplie,
            etat: Etat::default(),
            passe: Passe::nouveau(
                device,
                sortie,
                "trainee",
                wgpu::include_wgsl!("trainee.wgsl"),
                std::mem::size_of::<Uniforme>() as u64,
            ),
            disques: 0,
        }
    }

    fn duree(&self) -> Duration {
        Duration::from_secs_f32(self.reglages.duree_fondu)
    }
}

impl Effet for Trainee {
    fn nom(&self) -> &'static str {
        TRAINEE
    }

    fn preparer(&mut self, queue: &wgpu::Queue, contexte: &Contexte) {
        if let Some(curseur) = contexte.curseur {
            self.etat
                .ajouter(curseur, contexte.maintenant, &self.reglages);
        }
        let duree = self.duree();
        self.etat.retenir(contexte.maintenant, duree);

        let e = contexte.echelle;
        let r = &self.reglages;
        let mut donnees = Uniforme {
            couleur: r.couleur,
            non_premultiplie: if self.non_premultiplie { 1.0 } else { 0.0 },
            taille: [contexte.taille.0, contexte.taille.1],
            _remplissage: [0.0; 2],
            points: [[0.0; 4]; MAX_POINTS],
        };
        let mut n = 0;
        for (p, age) in self.etat.ages(contexte.maintenant, duree).take(MAX_POINTS) {
            donnees.points[n] = [
                p.x * e,
                p.y * e,
                r.rayon * (1.0 - 0.5 * age) * e,
                r.opacite * (1.0 - age) * PART_PAR_DISQUE,
            ];
            n += 1;
        }
        self.disques = n as u32;
        if n > 0 {
            queue.write_buffer(&self.passe.uniforme, 0, bytemuck::bytes_of(&donnees));
        }
    }

    fn dessiner(&self, passe: &mut wgpu::RenderPass<'_>) {
        self.passe.dessiner(passe, self.disques);
    }

    fn animation_en_cours(&self) -> bool {
        self.etat.en_cours()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_shader_est_valide() {
        super::super::commun::valider_shader(include_str!("trainee.wgsl"));
    }

    #[test]
    fn la_structure_envoyee_au_gpu_a_la_taille_du_shader() {
        // 32 octets d'en-tête, puis 128 entrées de 16 octets.
        assert_eq!(std::mem::size_of::<Uniforme>(), 32 + 128 * 16);
    }
}
