//! Rendu GPU du repère. Ne connaît pas le système : reçoit une fenêtre déjà créée.

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use crate::coeur::{Ecran, Point, REPERE};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforme {
    curseur: [f32; 2],
    rayon: f32,
    epaisseur: f32,
    couleur: [f32; 3],
    non_premultiplie: f32,
}

pub struct Rendu {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    uniforme: wgpu::Buffer,
    groupe: wgpu::BindGroup,
    non_premultiplie: bool,
}

impl Rendu {
    /// Prépare le rendu. Échoue, avec le constat des capacités de la surface, si aucun mode
    /// de transparence n'est disponible : afficher une surface opaque cacherait l'écran.
    pub fn nouveau(fenetre: Arc<Window>, affichage: OwnedDisplayHandle) -> Result<Self, String> {
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(affichage)),
        );
        let surface = instance
            .create_surface(fenetre.clone())
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
        let capacites = surface.get_capabilities(&adapter);
        crate::sonde::ecrire(&format!(
            "Sonde : adaptateur « {} », backend {:?}, modes alpha {:?}, modes de présentation {:?}",
            infos.name, infos.backend, capacites.alpha_modes, capacites.present_modes
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
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("repere.wgsl"));
        let uniforme = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("repere"),
            size: std::mem::size_of::<Uniforme>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let disposition = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let groupe = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &disposition,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforme.as_entire_binding(),
            }],
        });
        let gabarit = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&disposition)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("repere"),
            layout: Some(&gabarit),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Rendu {
            surface,
            config,
            device,
            queue,
            pipeline,
            uniforme,
            groupe,
            non_premultiplie,
        })
    }

    pub fn redimensionner(&mut self, largeur: u32, hauteur: u32) {
        if largeur == 0 || hauteur == 0 {
            return;
        }
        self.config.width = largeur;
        self.config.height = hauteur;
        self.surface.configure(&self.device, &self.config);
    }

    /// Dessine le repère autour du curseur. Sans curseur connu, l'image reste transparente.
    pub fn dessiner(&mut self, ecran: &Ecran, curseur: Option<Point>) {
        let e = ecran.echelle as f32;
        let (cx, cy) = curseur.map_or((-1000.0, -1000.0), |p| ecran.vers_physique(p));
        let donnees = Uniforme {
            curseur: [cx, cy],
            rayon: REPERE.rayon * e,
            epaisseur: REPERE.epaisseur * e,
            couleur: REPERE.couleur,
            non_premultiplie: if self.non_premultiplie { 1.0 } else { 0.0 },
        };
        self.queue
            .write_buffer(&self.uniforme, 0, bytemuck::bytes_of(&donnees));

        let texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            _ => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };
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
            passe.set_pipeline(&self.pipeline);
            passe.set_bind_group(0, &self.groupe, &[]);
            passe.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encodeur.finish()));
        self.queue.present(texture);
    }
}
