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
    let mut app = app::App::default();
    let resultat = boucle.run_app(&mut app);
    sonde::ecrire(&sonde::bilan(
        debut.elapsed(),
        plateforme::temps_processeur(),
        app.images,
    ));
    if let Err(e) = resultat {
        sonde::ecrire(&format!("SOURISO : {e}"));
        return ExitCode::from(2);
    }
    match app.echec {
        Some(e) => {
            sonde::ecrire(&format!("SOURISO : {e}"));
            ExitCode::from(2)
        }
        None => ExitCode::SUCCESS,
    }
}
