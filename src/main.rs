mod app;
mod coeur;
mod plateforme;
mod rendu;

use std::process::ExitCode;

fn main() -> ExitCode {
    let boucle = match plateforme::creer_boucle() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("SOURISO : {e}");
            return ExitCode::from(2);
        }
    };
    let mut app = app::App::default();
    if let Err(e) = boucle.run_app(&mut app) {
        eprintln!("SOURISO : {e}");
        return ExitCode::from(2);
    }
    match app.echec {
        Some(e) => {
            eprintln!("SOURISO : {e}");
            ExitCode::from(2)
        }
        None => ExitCode::SUCCESS,
    }
}
