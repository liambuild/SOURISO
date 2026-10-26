//! Interface commune des effets. Ajouter un effet = écrire un fichier qui implémente `Effet` et
//! le déclarer dans `creer_effets` : ni la boucle, ni la surcouche, ni la plateforme ne changent.

use crate::coeur::{Config, Point};

use super::projecteur::Projecteur;

/// Ce que le rendu sait au moment de dessiner une image d'un écran.
pub struct Contexte {
    /// Position du curseur sur cet écran, en pixels logiques. `None` s'il est sur un autre écran.
    pub curseur: Option<Point>,
    /// Pixels physiques par pixel logique de cet écran.
    pub echelle: f32,
}

/// Format de la surface : les effets construisent leur pipeline pour elle.
pub struct Sortie {
    pub format: wgpu::TextureFormat,
    /// Vrai si la surface attend de l'alpha non prémultiplié (repli : un seul effet y est correct,
    /// car les effets n'y sont pas mélangés).
    pub non_premultiplie: bool,
}

impl Sortie {
    /// Mélange à utiliser : les effets écrivent de l'alpha prémultiplié et se superposent.
    pub fn melange(&self) -> Option<wgpu::BlendState> {
        (!self.non_premultiplie).then_some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING)
    }
}

/// Un effet dessiné par-dessus le bureau, pour un écran. Il possède ses ressources graphiques.
pub trait Effet {
    /// Nom stable, clé de `coeur::activation::Activations`.
    fn nom(&self) -> &'static str;

    /// Met à jour les données envoyées au GPU pour l'image à venir.
    fn preparer(&mut self, queue: &wgpu::Queue, contexte: &Contexte);

    /// Enregistre les commandes de dessin dans la passe, déjà vidée en transparent.
    fn dessiner(&self, passe: &mut wgpu::RenderPass<'_>);
}

/// Les effets d'un écran, dans l'ordre de superposition.
pub fn creer_effets(
    device: &wgpu::Device,
    sortie: &Sortie,
    config: &Config,
) -> Vec<Box<dyn Effet>> {
    vec![Box::new(Projecteur::nouveau(
        device,
        sortie,
        config.projecteur,
    ))]
}
