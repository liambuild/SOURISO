// Sous Windows, un programme « console » ouvre une fenêtre de terminal qu'on ne peut pas fermer sans
// l'arrêter. SOURISO n'en a pas besoin : la sonde écrit dans `souriso-sonde.txt`. L'attribut est
// sans effet hors de Windows. C'est la seule ligne dépendante d'un système hors de `src/plateforme`
// (un attribut de racine de crate ne peut pas vivre ailleurs) : exception justifiée dans
// `docs/ARCHITECTURE.md`, section 4.1.
#![windows_subsystem = "windows"]

mod app;
mod coeur;
mod plateforme;
mod rendu;
mod sonde;

use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let debut = Instant::now();
    sonde::initialiser();
    let boucle = match plateforme::creer_boucle() {
        Ok(b) => b,
        Err(e) => {
            sonde::ecrire(&format!("SOURISO : {e}"));
            return ExitCode::from(2);
        }
    };
    let mut app = app::App::new(debut);
    let resultat = boucle.run_app(&mut app);
    sonde::bilan_final(&app.bilan());
    if let Err(e) = resultat {
        sonde::ecrire(&format!("SOURISO : {e}"));
        return ExitCode::from(2);
    }
    match &app.echec {
        Some(e) => {
            sonde::ecrire(&format!("SOURISO : {e}"));
            ExitCode::from(2)
        }
        None => ExitCode::SUCCESS,
    }
}
