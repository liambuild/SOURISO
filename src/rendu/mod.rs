//! Rendu GPU des effets. Ne connaît pas le système : reçoit des fenêtres déjà créées.
//! Un seul périphérique graphique sert toutes les surcouches ; chaque écran a sa surface et ses
//! effets.

mod effet;
mod projecteur;

use std::sync::Arc;

use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use crate::coeur::activation::Activations;
use crate::coeur::{Config, Ecran, Point};
use effet::{creer_effets, Contexte, Effet, Sortie};

/// Instance, adaptateur et périphérique partagés par tous les écrans.
pub struct Gpu {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Gpu {
    /// Choisit l'adaptateur compatible avec la surface de la première fenêtre et la renvoie.
    pub fn nouveau(
        affichage: OwnedDisplayHandle,
        premiere: &Arc<Window>,
    ) -> Result<(Self, wgpu::Surface<'static>), String> {
        let instance =
            wgpu::Instance::new(crate::plateforme::options_instance(affichage).with_env());
        let surface = instance
            .create_surface(premiere.clone())
            .map_err(|e| format!("création de la surface : {e}"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .map_err(|e| format!("aucun adaptateur graphique : {e}"))?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(|e| format!("périphérique graphique : {e}"))?;
        let infos = adapter.get_info();
        crate::sonde::ecrire(&format!(
            "Sonde : adaptateur « {} », backend {:?}",
            infos.name, infos.backend
        ));
        Ok((
            Gpu {
                instance,
                adapter,
                device,
                queue,
            },
            surface,
        ))
    }

    /// Surface d'une fenêtre supplémentaire. Échoue si l'adaptateur ne sait pas l'afficher.
    pub fn surface(&self, fenetre: &Arc<Window>) -> Result<wgpu::Surface<'static>, String> {
        let surface = self
            .instance
            .create_surface(fenetre.clone())
            .map_err(|e| format!("création de la surface : {e}"))?;
        if !self.adapter.is_surface_supported(&surface) {
            return Err("l'adaptateur graphique ne sait pas afficher sur cet écran".into());
        }
        Ok(surface)
    }
}

/// Acquisitions d'image qui ont échoué, pour la sonde.
#[derive(Clone, Copy, Debug, Default)]
pub struct Statistiques {
    pub echecs_acquisition: u64,
    pub reconfigurations: u64,
}

/// Rendu d'une surcouche : la surface d'un écran et les effets qui s'y dessinent.
pub struct Rendu {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    effets: Vec<Box<dyn Effet>>,
    statistiques: Statistiques,
}

impl Rendu {
    /// Prépare le rendu. Échoue, avec le constat des capacités de la surface, si aucun mode
    /// de transparence n'est disponible : afficher une surface opaque cacherait l'écran.
    pub fn nouveau(
        gpu: &Gpu,
        surface: wgpu::Surface<'static>,
        fenetre: &Window,
        indice: usize,
        reglages: &Config,
    ) -> Result<Self, String> {
        let capacites = surface.get_capabilities(&gpu.adapter);
        crate::sonde::ecrire(&format!(
            "Sonde : écran {indice}, modes alpha {:?}, modes de présentation {:?}",
            capacites.alpha_modes, capacites.present_modes
        ));

        let (alpha_mode, non_premultiplie) = if capacites
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            (wgpu::CompositeAlphaMode::PreMultiplied, false)
        } else if capacites
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
        {
            (wgpu::CompositeAlphaMode::PostMultiplied, true)
        } else {
            return Err(format!(
                "surface transparente indisponible (modes alpha : {:?}). Voir docs/ARCHITECTURE.md, risque R13 et replis.",
                capacites.alpha_modes
            ));
        };

        let format = capacites
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(capacites.formats[0]);
        let taille = fenetre.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: taille.width.max(1),
            height: taille.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            color_space: Default::default(),
            view_formats: vec![],
        };
        surface.configure(&gpu.device, &config);

        let sortie = Sortie {
            format,
            non_premultiplie,
        };
        Ok(Rendu {
            surface,
            config,
            device: gpu.device.clone(),
            queue: gpu.queue.clone(),
            effets: creer_effets(&gpu.device, &sortie, reglages),
            statistiques: Statistiques::default(),
        })
    }

    pub fn statistiques(&self) -> Statistiques {
        self.statistiques
    }

    pub fn redimensionner(&mut self, largeur: u32, hauteur: u32) {
        if largeur == 0 || hauteur == 0 {
            return;
        }
        self.config.width = largeur;
        self.config.height = hauteur;
        self.surface.configure(&self.device, &self.config);
    }

    /// Dessine les effets actifs. Sans effet actif, l'image reste transparente.
    /// `lire_curseur` donne la position sur cet écran, ou `None` s'il est ailleurs. Elle est
    /// appelée une fois l'image acquise, car l'acquisition peut attendre le rafraîchissement de
    /// l'écran : la position dessinée est ainsi la plus récente possible.
    /// Renvoie vrai si l'image a été présentée.
    pub fn dessiner(
        &mut self,
        ecran: &Ecran,
        lire_curseur: impl FnOnce() -> Option<Point>,
        activations: &Activations,
    ) -> bool {
        let texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            // Rien à reconfigurer : l'image est sautée et la boucle réessaie à son prochain passage.
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                self.statistiques.echecs_acquisition += 1;
                return false;
            }
            // Surface périmée ou perdue : seule une reconfiguration la remet en état.
            _ => {
                self.statistiques.echecs_acquisition += 1;
                self.statistiques.reconfigurations += 1;
                self.surface.configure(&self.device, &self.config);
                return false;
            }
        };

        let contexte = Contexte {
            curseur: lire_curseur(),
            echelle: ecran.echelle as f32,
        };
        for effet in &mut self.effets {
            if activations.est_actif(effet.nom()) {
                effet.preparer(&self.queue, &contexte);
            }
        }

        let vue = texture.texture.create_view(&Default::default());
        let mut encodeur = self.device.create_command_encoder(&Default::default());
        {
            let mut passe = encodeur.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &vue,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            for effet in &self.effets {
                if activations.est_actif(effet.nom()) {
                    effet.dessiner(&mut passe);
                }
            }
        }
        self.queue.submit(Some(encodeur.finish()));
        self.queue.present(texture);
        true
    }
}
