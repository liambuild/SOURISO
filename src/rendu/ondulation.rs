//! Effet ondulation au clic : un anneau qui s'élargit en s'effaçant à l'endroit du clic. Le clic
//! gauche et le clic droit n'ont ni la même couleur ni la même forme. Les ondulations d'un écran
//! se dessinent dans la même passe que les autres effets.

use std::time::{Duration, Instant};

use bytemuck::{Pod, Zeroable};

use crate::coeur::activation::ONDULATION;
use crate::coeur::clic::Bouton;
use crate::coeur::config::{self, Effets};
use crate::coeur::ondulation::{Ondulations, MAX};
use crate::coeur::Point;

use super::commun::Passe;
use super::effet::{Contexte, Effet, Sortie};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct Uniforme {
    /// x, y (pixels), avancement de 0 à 1, bouton (0 gauche, 1 droit).
    entrees: [[f32; 4]; MAX],
    /// Couleur RGB et opacité.
    couleur_gauche: [f32; 4],
    couleur_droit: [f32; 4],
    taille: [f32; 2],
    rayon_max: f32,
    epaisseur: f32,
    non_premultiplie: f32,
    _remplissage: [f32; 3],
}

pub struct Ondulation {
    reglages: config::Ondulation,
    non_premultiplie: bool,
    etat: Ondulations,
    passe: Passe,
    nombre: u32,
}

impl Ondulation {
    pub fn nouveau(device: &wgpu::Device, sortie: &Sortie, reglages: config::Ondulation) -> Self {
        Ondulation {
            reglages: reglages.borne(),
            non_premultiplie: sortie.non_premultiplie,
            etat: Ondulations::default(),
            passe: Passe::nouveau(
                device,
                sortie,
                "ondulation",
                wgpu::include_wgsl!("ondulation.wgsl"),
                std::mem::size_of::<Uniforme>() as u64,
            ),
            nombre: 0,
        }
    }

    fn duree(&self) -> Duration {
        Duration::from_secs_f32(self.reglages.duree)
    }
}

impl Effet for Ondulation {
    fn nom(&self) -> &'static str {
        ONDULATION
    }

    fn appliquer(&mut self, effets: &Effets) {
        self.reglages = effets.ondulation.borne();
    }

    fn clic(&mut self, bouton: Bouton, position: Point, maintenant: Instant) {
        self.etat.ajouter(position, bouton, maintenant);
    }

    fn preparer(&mut self, queue: &wgpu::Queue, contexte: &Contexte) {
        let duree = self.duree();
        self.etat.retenir(contexte.maintenant, duree);

        let e = contexte.echelle;
        let r = &self.reglages;
        let avec_opacite = |c: [f32; 3]| [c[0], c[1], c[2], r.opacite];
        let mut donnees = Uniforme {
            entrees: [[0.0; 4]; MAX],
            couleur_gauche: avec_opacite(r.couleur_gauche),
            couleur_droit: avec_opacite(r.couleur_droit),
            taille: [contexte.taille.0, contexte.taille.1],
            rayon_max: r.rayon_max * e,
            epaisseur: r.epaisseur * e,
            non_premultiplie: if self.non_premultiplie { 1.0 } else { 0.0 },
            _remplissage: [0.0; 3],
        };
        let mut n = 0;
        for (p, bouton, avancement) in self.etat.avancements(contexte.maintenant, duree).take(MAX) {
            donnees.entrees[n] = [
                p.x * e,
                p.y * e,
                avancement,
                match bouton {
                    Bouton::Gauche => 0.0,
                    Bouton::Droit => 1.0,
                },
            ];
            n += 1;
        }
        self.nombre = n as u32;
        if n > 0 {
            queue.write_buffer(&self.passe.uniforme, 0, bytemuck::bytes_of(&donnees));
        }
    }

    fn dessiner(&self, passe: &mut wgpu::RenderPass<'_>) {
        self.passe.dessiner(passe, self.nombre);
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
        super::super::commun::valider_shader(include_str!("ondulation.wgsl"));
    }

    #[test]
    fn la_structure_envoyee_au_gpu_a_la_taille_du_shader() {
        // 8 entrées de 16 octets, deux couleurs de 16 octets, puis 32 octets de réglages.
        assert_eq!(std::mem::size_of::<Uniforme>(), 8 * 16 + 2 * 16 + 32);
    }
}
