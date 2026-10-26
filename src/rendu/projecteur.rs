//! Effet projecteur : un voile sur tout l'écran, percé d'une zone éclairée aux bords adoucis
//! autour du curseur.

use bytemuck::{Pod, Zeroable};

use crate::coeur::activation::PROJECTEUR;
use crate::coeur::config;

use super::effet::{Contexte, Effet, Sortie};

/// Position hors de tout écran : sans curseur sur cet écran, le voile est plein.
const LOIN: f32 = -1.0e6;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct Uniforme {
    curseur: [f32; 2],
    rayon: f32,
    adoucissement: f32,
    couleur: [f32; 3],
    intensite: f32,
    non_premultiplie: f32,
    _remplissage: [f32; 3],
}

pub struct Projecteur {
    reglages: config::Projecteur,
    non_premultiplie: bool,
    pipeline: wgpu::RenderPipeline,
    uniforme: wgpu::Buffer,
    groupe: wgpu::BindGroup,
}

impl Projecteur {
    pub fn nouveau(device: &wgpu::Device, sortie: &Sortie, reglages: config::Projecteur) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("projecteur.wgsl"));
        let uniforme = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("projecteur"),
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
            label: Some("projecteur"),
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
                    format: sortie.format,
                    blend: sortie.melange(),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Projecteur {
            reglages: reglages.borne(),
            non_premultiplie: sortie.non_premultiplie,
            pipeline,
            uniforme,
            groupe,
        }
    }

    fn donnees(&self, contexte: &Contexte) -> Uniforme {
        let e = contexte.echelle;
        let (cx, cy) = contexte
            .curseur
            .map_or((LOIN, LOIN), |p| (p.x * e, p.y * e));
        let r = &self.reglages;
        Uniforme {
            curseur: [cx, cy],
            rayon: r.rayon * e,
            // Une bande d'au moins un pixel : le shader divise par sa largeur.
            adoucissement: (r.adoucissement * e).max(1.0),
            couleur: r.couleur,
            intensite: r.intensite,
            non_premultiplie: if self.non_premultiplie { 1.0 } else { 0.0 },
            _remplissage: [0.0; 3],
        }
    }
}

impl Effet for Projecteur {
    fn nom(&self) -> &'static str {
        PROJECTEUR
    }

    fn preparer(&mut self, queue: &wgpu::Queue, contexte: &Contexte) {
        queue.write_buffer(
            &self.uniforme,
            0,
            bytemuck::bytes_of(&self.donnees(contexte)),
        );
    }

    fn dessiner(&self, passe: &mut wgpu::RenderPass<'_>) {
        passe.set_pipeline(&self.pipeline);
        passe.set_bind_group(0, &self.groupe, &[]);
        passe.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_shader_est_valide() {
        let module = wgpu::naga::front::wgsl::parse_str(include_str!("projecteur.wgsl")).unwrap();
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap();
    }

    #[test]
    fn la_structure_envoyee_au_gpu_fait_48_octets() {
        assert_eq!(std::mem::size_of::<Uniforme>(), 48);
    }
}
