//! Temps processeur par `getrusage`, commun à macOS et à l'adaptateur factice.

use std::time::Duration;

use super::TempsProcesseur;

pub fn temps_processeur() -> Option<TempsProcesseur> {
    // SAFETY : `rusage` est une structure de nombres, valide une fois mise à zéro, et exclusive
    // pour la durée de l'appel.
    let usage = unsafe {
        let mut usage: libc::rusage = std::mem::zeroed();
        if libc::getrusage(libc::RUSAGE_SELF, &mut usage) != 0 {
            return None;
        }
        usage
    };
    let duree =
        |t: libc::timeval| Duration::new(t.tv_sec.max(0) as u64, (t.tv_usec.max(0) as u32) * 1000);
    Some(TempsProcesseur {
        noyau: duree(usage.ru_stime),
        utilisateur: duree(usage.ru_utime),
    })
}
