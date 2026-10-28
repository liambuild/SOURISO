//! Interface commune des effets. Ajouter un effet = écrire un fichier qui implémente `Effet` et
//! le déclarer dans `creer_effets` : ni la boucle, ni la surcouche, ni la plateforme ne changent.

use std::time::Instant;

use crate::coeur::clic::Bouton;
use crate::coeur::config::Effets;
use crate::coeur::{Config, Point};

use super::anneau::Anneau;
use super::ondulation::Ondulation;
use super::projecteur::Projecteur;
use super::trainee::Trainee;

/// Ce que le rendu sait au moment de dessiner une image d'un écran.
pub struct Contexte {
    /// Position du curseur sur cet écran, en pixels logiques. `None` s'il est sur un autre écran.
    pub curseur: Option<Point>,
    /// Pixels physiques par pixel logique de cet écran.
    pub echelle: f32,
    /// Taille de la surface en pixels physiques.
    pub taille: (f32, f32),
    /// Instant de l'image : tous les effets d'un écran partagent la même horloge.
    pub maintenant: Instant,
    /// Rectangle (gauche, haut, droite, bas) en pixels physiques de la surface où le voile du
    /// projecteur ne s'applique pas : l'emplacement de la fenêtre de réglages, qui doit rester
    /// lisible. Les autres effets le dessinent normalement. `None` : aucune fenêtre à ménager.
    pub exemption: Option<[f32; 4]>,
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

    /// Remplace les réglages de l'effet (changement de profil) sans toucher à son état : la
    /// traînée et les ondulations en cours continuent avec les nouvelles valeurs.
    fn appliquer(&mut self, effets: &Effets);

    /// Met à jour les données envoyées au GPU pour l'image à venir.
    fn preparer(&mut self, queue: &wgpu::Queue, contexte: &Contexte);

    /// Enregistre les commandes de dessin dans la passe, déjà vidée en transparent.
    fn dessiner(&self, passe: &mut wgpu::RenderPass<'_>);

    /// Un clic vient d'avoir lieu à `position` (pixels logiques de cet écran). Appelé même si
    /// l'image ne peut pas être dessinée tout de suite : le clic ne se perd pas.
    fn clic(&mut self, _bouton: Bouton, _position: Point, _maintenant: Instant) {}

    /// Vrai tant que l'effet a une animation à faire avancer ou à effacer : le rendu produit des
    /// images jusque-là, puis s'arrête. Un effet sans animation garde la valeur par défaut.
    fn animation_en_cours(&self) -> bool {
        false
    }
}

/// Les effets d'un écran, dans l'ordre de superposition (le voile en dessous).
pub fn creer_effets(
    device: &wgpu::Device,
    sortie: &Sortie,
    config: &Config,
) -> Vec<Box<dyn Effet>> {
    vec![
        Box::new(Projecteur::nouveau(
            device,
            sortie,
            config.effets.projecteur,
        )),
        Box::new(Trainee::nouveau(device, sortie, config.effets.trainee)),
        Box::new(Anneau::nouveau(device, sortie, config.effets.anneau)),
        Box::new(Ondulation::nouveau(
            device,
            sortie,
            config.effets.ondulation,
        )),
    ]
}
