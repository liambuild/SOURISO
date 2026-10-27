//! Essai de bout en bout sur un adaptateur graphique réel ou logiciel (Vulkan « lavapipe » sous
//! Linux) : les pipelines se créent, les quatre effets se dessinent dans une même passe et les
//! pixels attendus apparaissent. L'essai est ignoré, avec un message, s'il n'y a aucun adaptateur.

use std::time::{Duration, Instant};

use crate::coeur::clic::Bouton;
use crate::coeur::{Config, Point};

use super::effet::{creer_effets, Contexte, Sortie};

const L: u32 = 256;
const H: u32 = 256;

struct Banc {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn banc() -> Option<Banc> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        force_fallback_adapter: false,
        ..Default::default()
    }))
    .ok()?;
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
    Some(Banc { device, queue })
}

/// Dessine tous les effets actifs dans une texture hors écran et renvoie les pixels RGBA.
fn image(
    banc: &Banc,
    effets: &mut [Box<dyn super::effet::Effet>],
    contexte: &Contexte,
) -> Vec<[u8; 4]> {
    for e in effets.iter_mut() {
        e.preparer(&banc.queue, contexte);
    }
    let texture = banc.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: L,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let vue = texture.create_view(&Default::default());
    let tampon = banc.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(L * H * 4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encodeur = banc.device.create_command_encoder(&Default::default());
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
        for e in effets.iter() {
            e.dessiner(&mut passe);
        }
    }
    encodeur.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &tampon,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(L * 4),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: L,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    banc.queue.submit(Some(encodeur.finish()));
    tampon
        .slice(..)
        .map_async(wgpu::MapMode::Read, |r| r.unwrap());
    banc.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    let donnees = tampon.slice(..).get_mapped_range().unwrap();
    donnees.as_chunks::<4>().0.to_vec()
}

fn pixel(image: &[[u8; 4]], x: u32, y: u32) -> [u8; 4] {
    image[(y * L + x) as usize]
}

#[test]
fn les_quatre_effets_se_dessinent_dans_une_meme_passe() {
    let Some(banc) = banc() else {
        eprintln!("essai ignoré : aucun adaptateur graphique");
        return;
    };
    let config = Config::default();
    let sortie = Sortie {
        format: wgpu::TextureFormat::Rgba8Unorm,
        non_premultiplie: false,
    };
    let mut effets = creer_effets(&banc.device, &sortie, &config);
    let t0 = Instant::now();
    let curseur = Point { x: 128.0, y: 128.0 };
    let contexte = |t: Instant, curseur: Option<Point>| Contexte {
        curseur,
        echelle: 1.0,
        taille: (L as f32, H as f32),
        maintenant: t,
    };

    // Sans curseur ni clic : seul le voile du projecteur est là, partout.
    let vide = image(&banc, &mut effets, &contexte(t0, None));
    assert!(vide.iter().all(|p| p[3] > 0), "le voile couvre l'écran");

    // Curseur au centre : l'anneau (rayon 28, trait 4) est jaune à 28 px du centre.
    let avec_curseur = image(&banc, &mut effets, &contexte(t0, Some(curseur)));
    let sur_anneau = pixel(&avec_curseur, 128 + 28, 128);
    assert!(sur_anneau[0] > 150 && sur_anneau[1] > 100 && sur_anneau[2] < 80);
    // Au centre, le projecteur est transparent : il ne reste que la tête de la traînée (cyan, peu
    // opaque), ni le voile ni l'anneau.
    let centre = pixel(&avec_curseur, 128, 128);
    assert!(
        centre[3] > 0 && centre[3] < 100,
        "tête de traînée seule ({centre:?})"
    );
    assert!(centre[2] > centre[0], "cyan");

    // Clic droit au même endroit, à mi-parcours : de l'orange, et le centre reste transparent.
    for e in &mut effets {
        e.clic(Bouton::Droit, curseur, t0);
    }
    assert!(effets.iter().any(|e| e.animation_en_cours()));
    let mi = t0 + Duration::from_millis(300);
    let ondule = image(&banc, &mut effets, &contexte(mi, Some(curseur)));
    let change = ondule
        .iter()
        .zip(&avec_curseur)
        .filter(|(a, b)| a != b)
        .count();
    assert!(change > 100, "l'ondulation dessine des pixels ({change})");

    // Bien après la fin de toutes les animations : plus rien à animer, anneau et voile seuls au repos.
    let fin = t0 + Duration::from_secs(10);
    let repos = image(&banc, &mut effets, &contexte(fin, Some(curseur)));
    assert!(
        effets.iter().all(|e| !e.animation_en_cours()),
        "plus aucune animation"
    );
    // Le projecteur et l'anneau restent ; la tête de la traînée et l'ondulation ont disparu.
    assert_eq!(pixel(&repos, 128, 128)[3], 0, "traînée effacée");
    assert_eq!(pixel(&repos, 128 + 28, 128), sur_anneau);
}
