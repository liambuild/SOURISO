//! Ce que les effets à quads instanciés ont en commun : un pipeline, un tampon uniforme et son
//! groupe de liaison. Les effets partagent la passe et la présentation de l'écran ; ils ne
//! partagent que ce code de création.

use super::effet::Sortie;

pub struct Passe {
    pub pipeline: wgpu::RenderPipeline,
    pub uniforme: wgpu::Buffer,
    pub groupe: wgpu::BindGroup,
}

impl Passe {
    /// Pipeline sans tampon de sommets (les quads sont calculés par le shader) et un tampon
    /// uniforme de `taille` octets, lisible par les deux étages.
    pub fn nouveau(
        device: &wgpu::Device,
        sortie: &Sortie,
        nom: &str,
        shader: wgpu::ShaderModuleDescriptor<'_>,
        taille: u64,
    ) -> Self {
        let shader = device.create_shader_module(shader);
        let uniforme = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(nom),
            size: taille,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let disposition = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
            label: Some(nom),
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
        Passe {
            pipeline,
            uniforme,
            groupe,
        }
    }

    /// Dessine `instances` quads de six sommets. Rien n'est enregistré s'il n'y en a aucun.
    pub fn dessiner(&self, passe: &mut wgpu::RenderPass<'_>, instances: u32) {
        if instances == 0 {
            return;
        }
        passe.set_pipeline(&self.pipeline);
        passe.set_bind_group(0, &self.groupe, &[]);
        passe.draw(0..6, 0..instances);
    }
}

/// Valide un shader WGSL (utilisé par les tests de chaque effet).
#[cfg(test)]
pub fn valider_shader(source: &str) {
    let module = wgpu::naga::front::wgsl::parse_str(source).unwrap();
    wgpu::naga::valid::Validator::new(
        wgpu::naga::valid::ValidationFlags::all(),
        wgpu::naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .unwrap();
}
