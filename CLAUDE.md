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
- Une surcouche par écran. Les effets implémentent `rendu::effet::Effet` et se déclarent dans `creer_effets` ; tous leurs paramètres sont dans `coeur::config::Config` (champ `effets`, copie de ceux du profil actif). Tous les effets d'un écran se dessinent dans la même passe et la même présentation, jamais une présentation par effet ; une animation ne demande des images que pendant sa durée (`Effet::animation_en_cours`).
- Les profils (`coeur::profil`, fichier `profils.json` lu et écrit par `coeur::stockage`, dossier donné par `plateforme::dossier_donnees`) regroupent les réglages de tous les effets, l'activation de chaque effet (`actifs`) et un raccourci chacun ; le premier est le profil par défaut, non supprimable. Le dernier profil utilisé est restauré au démarrage ; un fichier absent ou corrompu ramène aux valeurs par défaut sans planter (`docs/ARCHITECTURE.md`, section 9). Changer de profil remplace les réglages des effets en place (`Effet::appliquer`) et applique l'activation enregistrée dans le profil, sans nouvelle présentation. Les raccourcis P, A, T, O, S ne durent que la session et ne modifient pas le profil ; l'interrupteur de la fenêtre de réglages modifie le profil actif.
- La fenêtre de réglages est un second exécutable, `souriso-reglages` (crate `reglages/`, Tauri 2, page statique dans `reglages/ui/`, sans framework ni npm ni ressource distante), lancé à côté de `souriso` par `plateforme::FenetreReglages` et fermé avec sa fenêtre ; échanges par entrée et sortie standard, une ligne de JSON par message, dont tout le langage vit dans `coeur::reglages` (le crate des réglages ne l'interprète pas). Chaque réglage s'applique aux effets réels (aperçu en direct, aucun second rendu) ; le voile du projecteur ménage un trou à l'emplacement de la fenêtre (`Contexte::exemption`). Le crate des réglages ne contient aucune condition ni API système (contrôlé comme `src/`). Voir `docs/ARCHITECTURE.md`, section 9.
- Les clics se lisent par interrogation des boutons dans la boucle du curseur (`plateforme::boutons`), le son se joue à la demande (`plateforme::jouer_son`) ; les sons sont générés par `examples/generer_sons.rs` (`docs/SONS.md`).
- La sonde écrit une ligne « Bilan » dans `souriso-sonde.txt` à la fermeture, et la réécrit toutes les 10 secondes (elle reste la dernière ligne) : durée, temps processeur (noyau et utilisateur), images par écran, réveils, lectures du curseur, échecs d'acquisition, reconfigurations, clics gauche et droit détectés, sons joués, profil actif et changements de profil.
- La sonde écrit aussi, au lancement, la liste des adaptateurs graphiques que wgpu énumère (nom, type, backend, présentation possible sur la surface), l'adaptateur retenu et, par écran, son taux de rafraîchissement lu par le système.
- L'adaptateur se choisit par `WGPU_ADAPTER_NAME` (fragment du nom), puis `WGPU_POWER_PREF` (`low`, `high`, `none`), puis le réglage « économe » ou « performant » du fichier des profils, puis la carte qui porte l'écran principal (demandée à la couche plateforme, Windows seulement), puis le choix de wgpu. Le pilote logiciel n'est retenu que faute d'autre carte ; la sonde cite le critère décisif (`docs/ARCHITECTURE.md`, 4.2).
- Aucun droit administrateur ne doit être requis sous Windows.

## Fonctions dépendant du système

Toute fonction qui dépend du système est soit testée sur le système concerné, soit marquée « non testé » dans `docs/RECETTE.md`.

## Commandes de vérification

Chaîne Rust épinglée par `rust-toolchain.toml` (installation par `rustup`). Mêmes commandes en local et en CI (`.github/workflows/ci.yml`, Windows et macOS) :

- `./scripts/verifier-frontiere.sh` : aucune API ni condition système hors de `src/plateforme` (et aucune dans `reglages/`, dont la page n'appelle aucune ressource distante).
- `cargo fmt --all --check`
- `cargo clippy --all-targets --locked -- -D warnings`
- `cargo test --locked`
- `cargo build --release --locked`

Ces quatre dernières visent le crate principal (`default-members`). La CI ajoute `--workspace` (les deux crates) et publie, par système, un artefact avec `souriso` et `souriso-reglages` côte à côte. Le crate des réglages se vérifie aussi en local avec `-p souriso-reglages` (WebKitGTK requis sous Linux) ; sa compilation croisée vers Windows ou macOS depuis Linux est impossible, la CI s'en charge.

Sous Linux, l'adaptateur factice permet de compiler et de tester le cœur. Vérification croisée possible avec `cargo check --target x86_64-pc-windows-msvc` et `--target aarch64-apple-darwin` (cibles à ajouter par `rustup target add`). Elle ne remplace pas la recette sur le vrai système.

## Points de contrôle au démarrage

Lecture seule :

- `cargo --version` doit correspondre au canal de `rust-toolchain.toml`.
- `./scripts/verifier-frontiere.sh` doit réussir.
- `cargo fmt --all --check` doit réussir.
