//! Effet anneau : un trait coloré circulaire centré sur le curseur. Seul un carré autour du
//! curseur est rempli.

use bytemuck::{Pod, Zeroable};

use crate::coeur::activation::ANNEAU;
use crate::coeur::config;

use super::commun::Passe;
use super::effet::{Contexte, Effet, Sortie};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct Uniforme {
    curseur: [f32; 2],
    taille: [f32; 2],
    couleur: [f32; 3],
    opacite: f32,
    rayon: f32,
    epaisseur: f32,
    demi_cote: f32,
    non_premultiplie: f32,
}

pub struct Anneau {
    reglages: config::Anneau,
    non_premultiplie: bool,
    passe: Passe,
    /// Vrai si le curseur est sur cet écran : sinon rien n'est dessiné.
    visible: bool,
}

impl Anneau {
    pub fn nouveau(device: &wgpu::Device, sortie: &Sortie, reglages: config::Anneau) -> Self {
        Anneau {
            reglages: reglages.borne(),
            non_premultiplie: sortie.non_premultiplie,
            passe: Passe::nouveau(
                device,
                sortie,
                "anneau",
                wgpu::include_wgsl!("anneau.wgsl"),
                std::mem::size_of::<Uniforme>() as u64,
            ),
            visible: false,
        }
    }

    fn donnees(&self, contexte: &Contexte, curseur: crate::coeur::Point) -> Uniforme {
        let e = contexte.echelle;
        let r = &self.reglages;
        Uniforme {
            curseur: [curseur.x * e, curseur.y * e],
            taille: [contexte.taille.0, contexte.taille.1],
            couleur: r.couleur,
            opacite: r.opacite,
            rayon: r.rayon * e,
            epaisseur: r.epaisseur * e,
            // Le trait déborde de la moitié de son épaisseur ; deux pixels de marge pour le bord.
            demi_cote: (r.rayon + r.epaisseur * 0.5) * e + 2.0,
            non_premultiplie: if self.non_premultiplie { 1.0 } else { 0.0 },
        }
    }
}

impl Effet for Anneau {
    fn nom(&self) -> &'static str {
        ANNEAU
    }

    fn preparer(&mut self, queue: &wgpu::Queue, contexte: &Contexte) {
        self.visible = contexte.curseur.is_some();
        if let Some(curseur) = contexte.curseur {
            queue.write_buffer(
                &self.passe.uniforme,
                0,
                bytemuck::bytes_of(&self.donnees(contexte, curseur)),
            );
        }
    }

    fn dessiner(&self, passe: &mut wgpu::RenderPass<'_>) {
        self.passe.dessiner(passe, u32::from(self.visible));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_shader_est_valide() {
        super::super::commun::valider_shader(include_str!("anneau.wgsl"));
    }

    #[test]
    fn la_structure_envoyee_au_gpu_fait_48_octets() {
        assert_eq!(std::mem::size_of::<Uniforme>(), 48);
    }
}
