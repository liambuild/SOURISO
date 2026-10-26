mod app;
mod coeur;
mod plateforme;
mod rendu;
mod sonde;

use std::process::ExitCode;

fn main() -> ExitCode {
    sonde::initialiser();
    let boucle = match plateforme::creer_boucle() {
        Ok(b) => b,
        Err(e) => {
            sonde::ecrire(&format!("SOURISO : {e}"));
            return ExitCode::from(2);
        }
    };
    let mut app = app::App::default();
    if let Err(e) = boucle.run_app(&mut app) {
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
