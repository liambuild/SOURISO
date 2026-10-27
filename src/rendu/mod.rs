//! Rendu GPU des effets. Ne connaît pas le système : reçoit des fenêtres déjà créées.
//! Un seul périphérique graphique sert toutes les surcouches ; chaque écran a sa surface et ses
//! effets.

mod anneau;
mod commun;
mod effet;
mod ondulation;
mod projecteur;
mod trainee;

#[cfg(test)]
mod essai_gpu;

use std::sync::Arc;

use std::time::Instant;

use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use crate::coeur::activation::Activations;
use crate::coeur::clic::Bouton;
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
    /// `cartes` donne, par écran (le principal en premier), la carte qui le porte si le système
    /// la connaît.
    pub fn nouveau(
        affichage: OwnedDisplayHandle,
        premiere: &Arc<Window>,
        cartes: &[Option<String>],
    ) -> Result<(Self, wgpu::Surface<'static>), String> {
        let instance =
            wgpu::Instance::new(crate::plateforme::options_instance(affichage).with_env());
        let surface = instance
            .create_surface(premiere.clone())
            .map_err(|e| format!("création de la surface : {e}"))?;
        let mut adaptateurs =
            pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
        let candidats: Vec<Candidat> = adaptateurs
            .iter()
            .map(|a| {
                let infos = a.get_info();
                Candidat {
                    nom: infos.name,
                    type_: infos.device_type,
                    presente: a.is_surface_supported(&surface),
                }
            })
            .collect();
        for (indice, (a, candidat)) in adaptateurs.iter().zip(&candidats).enumerate() {
            crate::sonde::ecrire(&format!(
                "Sonde : adaptateur disponible {indice} : « {} », type {}, backend {:?}, \
                 présentation sur la surface {}",
                candidat.nom,
                type_adaptateur(candidat.type_),
                a.get_info().backend,
                if candidat.presente {
                    "possible"
                } else {
                    "impossible"
                }
            ));
        }
        for (indice, carte) in cartes.iter().enumerate() {
            crate::sonde::ecrire(&format!(
                "Sonde : écran {indice} porté par la carte {}",
                match carte {
                    Some(nom) => format!("« {nom} »"),
                    None => "inconnue".to_string(),
                }
            ));
        }
        // Ordre de priorité : `WGPU_ADAPTER_NAME`, puis `WGPU_POWER_PREF`, puis la carte qui porte
        // l'écran principal, puis le choix de wgpu.
        let par_nom = std::env::var("WGPU_ADAPTER_NAME")
            .ok()
            .filter(|nom| !nom.is_empty())
            .and_then(|voulu| {
                let indice = indice_par_nom(&candidats, &voulu);
                if indice.is_none() {
                    crate::sonde::ecrire(&format!(
                        "Sonde : WGPU_ADAPTER_NAME « {voulu} » ne correspond à aucun adaptateur \
                         pouvant présenter, choix par défaut"
                    ));
                }
                indice
            });
        let preference = wgpu::PowerPreference::from_env();
        let (indice, mut critere) = if let Some(indice) = par_nom {
            (Some(indice), Critere::VariableNom)
        } else if preference.is_some() {
            (None, Critere::VariablePreference)
        } else if let Some(indice) = indice_carte_des_ecrans(&candidats, cartes) {
            (Some(indice), Critere::CarteDesEcrans)
        } else {
            (None, Critere::ChoixWgpu)
        };
        let adapter = match indice {
            Some(indice) => adaptateurs.swap_remove(indice),
            None => {
                let retenu =
                    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: preference.unwrap_or_default(),
                        compatible_surface: Some(&surface),
                        ..Default::default()
                    }))
                    .map_err(|e| format!("aucun adaptateur graphique : {e}"))?;
                // Le pilote logiciel (type processeur) ne sert que faute de mieux.
                let sans_processeur = match critere {
                    Critere::ChoixWgpu
                        if retenu.get_info().device_type == wgpu::DeviceType::Cpu =>
                    {
                        indice_hors_processeur(&candidats)
                    }
                    _ => None,
                };
                match sans_processeur {
                    Some(indice) => {
                        critere = Critere::ChoixWgpuSansProcesseur;
                        adaptateurs.swap_remove(indice)
                    }
                    None => retenu,
                }
            }
        };
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(|e| format!("périphérique graphique : {e}"))?;
        let infos = adapter.get_info();
        crate::sonde::ecrire(&format!(
            "Sonde : adaptateur « {} », type {}, backend {:?}, critère : {}",
            infos.name,
            type_adaptateur(infos.device_type),
            infos.backend,
            critere.libelle()
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

/// Adaptateur énuméré par wgpu, tel que la sonde et le choix le voient.
struct Candidat {
    nom: String,
    type_: wgpu::DeviceType,
    presente: bool,
}

/// Ce qui a décidé de l'adaptateur, pour la sonde.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Critere {
    VariableNom,
    VariablePreference,
    CarteDesEcrans,
    ChoixWgpu,
    ChoixWgpuSansProcesseur,
}

impl Critere {
    fn libelle(self) -> &'static str {
        match self {
            Critere::VariableNom => "variable WGPU_ADAPTER_NAME",
            Critere::VariablePreference => "variable WGPU_POWER_PREF",
            Critere::CarteDesEcrans => "carte qui porte les écrans",
            Critere::ChoixWgpu => "choix de wgpu",
            Critere::ChoixWgpuSansProcesseur => "choix de wgpu, pilote logiciel écarté",
        }
    }
}

/// Premier adaptateur pouvant présenter dont le nom contient `voulu`, sans tenir compte de la casse.
fn indice_par_nom(candidats: &[Candidat], voulu: &str) -> Option<usize> {
    let voulu = voulu.to_lowercase();
    candidats
        .iter()
        .position(|c| c.presente && c.nom.to_lowercase().contains(&voulu))
}

/// Premier adaptateur pouvant présenter qui n'est pas un pilote logiciel (type processeur).
fn indice_hors_processeur(candidats: &[Candidat]) -> Option<usize> {
    candidats
        .iter()
        .position(|c| c.presente && c.type_ != wgpu::DeviceType::Cpu)
}

/// Adaptateur qui correspond à la carte de l'écran principal (le premier de `cartes`). Si les
/// écrans sont répartis entre plusieurs cartes, celle du principal l'emporte. `None` si le
/// système n'a pas donné cette carte ou si aucun adaptateur qui peut présenter ne lui correspond.
fn indice_carte_des_ecrans(candidats: &[Candidat], cartes: &[Option<String>]) -> Option<usize> {
    let carte = cartes.first()?.as_ref()?.trim().to_lowercase();
    if carte.is_empty() {
        return None;
    }
    candidats.iter().position(|c| {
        let nom = c.nom.trim().to_lowercase();
        c.presente
            && c.type_ != wgpu::DeviceType::Cpu
            && (nom == carte || nom.contains(&carte) || carte.contains(&nom))
    })
}

/// Type d'adaptateur pour la sonde.
fn type_adaptateur(type_: wgpu::DeviceType) -> &'static str {
    match type_ {
        wgpu::DeviceType::IntegratedGpu => "intégré",
        wgpu::DeviceType::DiscreteGpu => "dédié",
        wgpu::DeviceType::VirtualGpu => "virtuel",
        wgpu::DeviceType::Cpu => "processeur",
        wgpu::DeviceType::Other => "autre",
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
        ecran: &Ecran,
        indice: usize,
        reglages: &Config,
    ) -> Result<Self, String> {
        let capacites = surface.get_capabilities(&gpu.adapter);
        crate::sonde::ecrire(&format!(
            "Sonde : écran {indice}, taux de rafraîchissement {}, modes alpha {:?}, \
             modes de présentation {:?}",
            crate::sonde::rafraichissement(ecran.rafraichissement_mhz),
            capacites.alpha_modes,
            capacites.present_modes
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

    /// Transmet un clic aux effets actifs de cet écran.
    pub fn clic(&mut self, bouton: Bouton, position: Point, activations: &Activations) {
        let maintenant = Instant::now();
        for effet in &mut self.effets {
            if activations.est_actif(effet.nom()) {
                effet.clic(bouton, position, maintenant);
            }
        }
    }

    /// Vrai si un effet actif a encore une animation à produire.
    pub fn animation_en_cours(&self, activations: &Activations) -> bool {
        self.effets
            .iter()
            .any(|e| activations.est_actif(e.nom()) && e.animation_en_cours())
    }

    /// Dessine les effets actifs, tous dans la même passe et la même présentation. Sans effet
    /// actif, l'image reste transparente.
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
            taille: (self.config.width as f32, self.config.height as f32),
            maintenant: Instant::now(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn candidat(nom: &str, type_: wgpu::DeviceType, presente: bool) -> Candidat {
        Candidat {
            nom: nom.to_string(),
            type_,
            presente,
        }
    }

    fn candidats() -> Vec<Candidat> {
        vec![
            candidat("NVIDIA Quadro P620", wgpu::DeviceType::DiscreteGpu, true),
            candidat(
                "Intel(R) UHD Graphics",
                wgpu::DeviceType::IntegratedGpu,
                true,
            ),
            candidat("Intel(R) Iris", wgpu::DeviceType::IntegratedGpu, false),
            candidat("Microsoft Basic Render Driver", wgpu::DeviceType::Cpu, true),
        ]
    }

    fn cartes(noms: &[Option<&str>]) -> Vec<Option<String>> {
        noms.iter().map(|n| n.map(str::to_string)).collect()
    }

    #[test]
    fn le_nom_voulu_est_cherche_sans_casse_parmi_les_adaptateurs_qui_presentent() {
        assert_eq!(indice_par_nom(&candidats(), "intel"), Some(1));
        assert_eq!(indice_par_nom(&candidats(), "QUADRO"), Some(0));
        assert_eq!(indice_par_nom(&candidats(), "iris"), None);
        assert_eq!(indice_par_nom(&candidats(), "amd"), None);
    }

    #[test]
    fn la_carte_de_l_ecran_principal_est_retenue() {
        let intel = cartes(&[Some("Intel(R) UHD Graphics"), Some("NVIDIA Quadro P620")]);
        let quadro = cartes(&[Some("NVIDIA Quadro P620"), Some("Intel(R) UHD Graphics")]);
        assert_eq!(indice_carte_des_ecrans(&candidats(), &intel), Some(1));
        assert_eq!(indice_carte_des_ecrans(&candidats(), &quadro), Some(0));
    }

    #[test]
    fn sans_avis_du_systeme_ou_sans_correspondance_le_choix_revient_a_wgpu() {
        assert_eq!(indice_carte_des_ecrans(&candidats(), &[]), None);
        assert_eq!(
            indice_carte_des_ecrans(&candidats(), &cartes(&[None])),
            None
        );
        assert_eq!(
            indice_carte_des_ecrans(&candidats(), &cartes(&[Some("AMD Radeon")])),
            None
        );
        // Une carte qui ne peut pas présenter n'est pas retenue.
        assert_eq!(
            indice_carte_des_ecrans(&candidats(), &cartes(&[Some("Intel(R) Iris")])),
            None
        );
    }

    #[test]
    fn le_pilote_logiciel_n_est_jamais_retenu_par_la_carte_des_ecrans() {
        let logiciel = cartes(&[Some("Microsoft Basic Render Driver")]);
        assert_eq!(indice_carte_des_ecrans(&candidats(), &logiciel), None);
    }

    #[test]
    fn le_premier_adaptateur_hors_processeur_qui_presente_est_cherche() {
        assert_eq!(indice_hors_processeur(&candidats()), Some(0));
        let seul = vec![candidat(
            "Microsoft Basic Render Driver",
            wgpu::DeviceType::Cpu,
            true,
        )];
        assert_eq!(indice_hors_processeur(&seul), None);
    }

    #[test]
    fn le_critere_se_dit_en_francais() {
        assert_eq!(
            Critere::CarteDesEcrans.libelle(),
            "carte qui porte les écrans"
        );
        assert_eq!(Critere::VariableNom.libelle(), "variable WGPU_ADAPTER_NAME");
    }

    #[test]
    fn le_type_d_adaptateur_est_dit_en_francais() {
        assert_eq!(type_adaptateur(wgpu::DeviceType::IntegratedGpu), "intégré");
        assert_eq!(type_adaptateur(wgpu::DeviceType::DiscreteGpu), "dédié");
    }
}
