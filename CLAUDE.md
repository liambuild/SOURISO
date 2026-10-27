# SOURISO

## Présentation

- SOURISO est une application de bureau installable sur macOS et Windows.
- Elle met en évidence le curseur de la souris pour les créateurs de contenu et les formateurs.
- Effets : projecteur, anneau, traînée, ondulation au clic et son.
- Le dépôt GitHub liambuild/SOURISO est la source de vérité de bout en bout.
- Il contient le code, la documentation, les décisions, la recette et les installeurs publiés en releases.

## Périmètre

- Systèmes : macOS et Windows. Linux est hors périmètre.
- Cibles : créateurs de contenu et formateurs.
- Aucun compte, aucun réseau, aucune télémétrie.

## Méthode de travail

- Une branche par changement, préfixée `feat`, `fix`, `docs` ou `chore`, créée depuis `main` à jour.
- Jamais de commit sur `main`.
- Commits en français.
- Toute modification se termine par `/cloturer` et une merge request vers `main`.
- Tu ne merges jamais.
- Tu ne lis ni n'affiches jamais un secret, un certificat ou un jeton.
- Si une consigne contredit le code, tu t'arrêtes et tu me le signales.
- Avant de modifier tout ce qui touche aux permissions système, à la signature ou à la publication, tu me rapportes ton constat.

## Architecture

- La base technique et la frontière système sont décidées dans `docs/ARCHITECTURE.md`.
- Tout ce qui dépend d'un système vit dans la couche plateforme, nulle part ailleurs (y compris le raccourci global et la lecture du temps processeur).
- Une surcouche par écran. Les effets implémentent `rendu::effet::Effet` et se déclarent dans `creer_effets` ; tous leurs paramètres sont dans `coeur::config::Config`.
- La sonde écrit une ligne « Bilan » dans `souriso-sonde.txt` à la fermeture, et la réécrit toutes les 10 secondes (elle reste la dernière ligne) : durée, temps processeur (noyau et utilisateur), images par écran, réveils, lectures du curseur, échecs d'acquisition, reconfigurations.
- La sonde écrit aussi, au lancement, la liste des adaptateurs graphiques que wgpu énumère (nom, type, backend, présentation possible sur la surface), l'adaptateur retenu et, par écran, son taux de rafraîchissement lu par le système.
- L'adaptateur se choisit par `WGPU_ADAPTER_NAME` (fragment du nom) puis `WGPU_POWER_PREF` (`low`, `high`, `none`) ; sans elles, le choix par défaut de wgpu s'applique.
- Aucun droit administrateur ne doit être requis sous Windows.

## Fonctions dépendant du système

Toute fonction qui dépend du système est soit testée sur le système concerné, soit marquée « non testé » dans `docs/RECETTE.md`.

## Commandes de vérification

Chaîne Rust épinglée par `rust-toolchain.toml` (installation par `rustup`). Mêmes commandes en local et en CI (`.github/workflows/ci.yml`, Windows et macOS) :

- `./scripts/verifier-frontiere.sh` : aucune API ni condition système hors de `src/plateforme`.
- `cargo fmt --check`
- `cargo clippy --all-targets --locked -- -D warnings`
- `cargo test --locked`
- `cargo build --release --locked`

Sous Linux, l'adaptateur factice permet de compiler et de tester le cœur. Vérification croisée possible avec `cargo check --target x86_64-pc-windows-msvc` et `--target aarch64-apple-darwin` (cibles à ajouter par `rustup target add`). Elle ne remplace pas la recette sur le vrai système.

## Points de contrôle au démarrage

Lecture seule :

- `cargo --version` doit correspondre au canal de `rust-toolchain.toml`.
- `./scripts/verifier-frontiere.sh` doit réussir.
- `cargo fmt --check` doit réussir.
