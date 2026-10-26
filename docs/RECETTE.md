# Recette

Ce document recense les vérifications du produit, avec le résultat par système (macOS, Windows) et la mention « non testé » le cas échéant.

## chore/init-methode

- [ ] Le README présente le produit, son périmètre et l'organisation du dépôt.
- [ ] CLAUDE.md contient la présentation, le périmètre, la méthode de travail et la règle sur les fonctions dépendant du système.
- [ ] CLAUDE.md contient les sections vides « Commandes de vérification » et « Points de contrôle au démarrage ».

## docs/cadrage-technique

- [ ] `docs/ARCHITECTURE.md` compare au moins trois familles d'approches sur tous les critères demandés.
- [ ] Il contient une recommandation, une frontière système, des risques classés avec leur test le moins coûteux et des critères de succès mesurables.
- [ ] La section « Décision » indique la famille retenue, sa condition (sonde comparative sur Windows) et ses replis.
- [ ] Les contraintes sont consignées : Windows et macOS, aucun droit administrateur, exigence d'esthétique, Mac indisponible pour l'instant.
- [ ] Les affirmations propres à un système sont marquées « à vérifier » : aucune n'est testée à ce stade (Windows : non testé ; macOS : non testé).

## chore/socle-multi-os

Socle minimal : surcouche plein écran transparente et traversable, repère qui suit le curseur, icône pour quitter. Rendu `wgpu` (famille D). Aucun effet de clic, aucun son, aucun réglage, écran principal seulement.

**État au moment de la livraison** : la CI compile et teste sur Windows et sur macOS, mais personne n'a encore lancé l'application sur un vrai système. Tout ce qui suit est « non testé » jusqu'à ce que tu coches la case et indiques le résultat.

**Ce que fait la sonde (R13).** Au lancement, l'application écrit sur la sortie d'erreur une ligne `Sonde : adaptateur …, backend …, modes alpha […], modes de présentation […]`. Si aucun mode alpha transparent n'est disponible (`PreMultiplied` ou `PostMultiplied`), elle **ne montre pas la fenêtre**, affiche le constat et quitte avec le code 2 : une surface opaque cacherait l'écran. C'est le signal pour décider du repli (`docs/ARCHITECTURE.md`, section 4). Lancer depuis un terminal pour lire la ligne : `cargo run --release` (Windows : PowerShell).

Variable utile à la sonde : `WGPU_BACKEND=vulkan` ou `dx12` (Windows), `metal` (macOS) force le backend. À noter dans le résultat.

**Tester sans rien installer (exécutable issu de la CI).** Chaque exécution de la CI conserve l'exécutable compilé en mode release, 7 jours, sans release ni étiquette. Artefacts : `souriso-Windows` (contient `souriso.exe`) et `souriso-macOS` (contient `souriso`).

1. GitHub, onglet *Actions*, ouvrir l'exécution voulue de « CI » (le bouton *Run workflow* en lance une sur la branche), descendre à la section *Artifacts*.
2. Télécharger `souriso-Windows` (ou `souriso-macOS`), puis extraire l'archive zip dans un dossier où l'on peut écrire (le fichier texte y sera créé).
3. Windows : double-clic sur `souriso.exe`. SmartScreen peut afficher « Windows a protégé votre ordinateur » (exécutable non signé) : *Informations complémentaires*, puis *Exécuter quand même*. À noter dans le scénario 6.
4. macOS : l'extraction perd le droit d'exécution et le fichier est mis en quarantaine. Dans un terminal : `chmod +x souriso && xattr -d com.apple.quarantine souriso && ./souriso`. À noter dans le scénario 6.
5. Lire `souriso-sonde.txt`, créé à côté de l'exécutable et remis à zéro à chaque lancement : la ligne `Sonde : adaptateur …, modes alpha […]`, puis, si l'application s'est arrêtée avec le code 2, la ligne `SOURISO : …` qui en donne le motif. Le même texte va sur la sortie d'erreur si un terminal est présent.

### Scénarios à tester par toi

| # | Scénario | Attendu | Windows | macOS |
|---|---|---|---|---|
| 1 | Lancement : `cargo run --release` depuis un terminal | Aucune invite d'élévation (UAC). Ligne « Sonde » lisible. Pas d'entrée dans la barre des tâches (Windows) ni dans le Dock (macOS) | non testé | non testé |
| 2 | Transparence | Le bureau reste visible, aucun fond noir ni blanc ni cadre ni ombre | non testé | non testé |
| 3 | Repère qui suit le curseur | Un anneau jaune centré sur le curseur, qui le suit sans à-coup visible | non testé | non testé |
| 4 | Clics qui traversent | Cliquer, double-cliquer, glisser, faire défiler et cliquer droit à travers l'anneau agit sur l'application en dessous | non testé | non testé |
| 5 | Fermeture par l'icône | L'icône apparaît en zone de notification (Windows) ou en barre de menus (macOS). Menu « Quitter SOURISO » : l'anneau disparaît et le processus se termine (vérifier dans le Gestionnaire des tâches ou le Moniteur d'activité) | non testé | non testé |
| 6 | Permissions demandées par le système | Noter toute invite au premier lancement (macOS : accessibilité, surveillance de l'entrée, enregistrement d'écran ; Windows : pare-feu, SmartScreen). Attendu : aucune, car aucun clic n'est écouté à ce stade | non testé | non testé |
| 7 | Mise à l'échelle | Écran à 100 % puis à 150 % ou Retina : l'anneau reste centré sur le curseur (écran principal seulement) | non testé | non testé |
| 8 | Aucun droit administrateur (Windows) | Compte standard : lancement sans élévation | non testé | sans objet |
| 9 | Processeur au repos | Curseur immobile : usage proche de zéro | non testé | non testé |

### Fonctions de la couche plateforme

| Fonction | Windows | macOS |
|---|---|---|
| Création de la surcouche (transparente, sans cadre, au-dessus) | non testé | non testé |
| Traversée des clics (`set_cursor_hittest(false)`) | non testé | non testé |
| Position du curseur (`GetCursorPos` / CoreGraphics) et conversion en pixels logiques | non testé | non testé |
| Icône de zone de notification ou de barre de menus | non testé | non testé |
| Absence d'icône dans le Dock | sans objet | non testé |
| Visible au-dessus d'une app en plein écran (R2) | hors périmètre du socle | hors périmètre du socle |
| Plusieurs écrans | hors périmètre du socle | hors périmètre du socle |

### Vérifications automatiques

- [ ] La CI est verte sur Windows et sur macOS (format, lint, tests, compilation, frontière système).
- [ ] `./scripts/verifier-frontiere.sh` échoue si une API système apparaît hors de `src/plateforme` (vérifié à la main avec un fichier fautif).
- [ ] `CLAUDE.md` renseigne « Commandes de vérification » et « Points de contrôle au démarrage ».

## chore/repli-surcouche-windows

Surcouche Windows : DX12 avec présentation par composition (DirectComposition), choisie dans `plateforme::options_instance`. macOS inchangé. Voir `docs/ARCHITECTURE.md`, section 9.

**État au moment de la livraison** : la CI compile et vérifie sur Windows et macOS, mais rien de ce qui suit n'a été exécuté par l'assistant. Tout est « non testé » jusqu'à ce que tu coches la case et indiques le résultat. Plus besoin de poser `WGPU_BACKEND` ni `WGPU_DX12_PRESENTATION_SYSTEM` : l'exécutable issu de la CI fait ce choix seul. Un lancement sans ces variables doit donc donner le même résultat que ton test.

| # | Scénario (Windows) | Attendu | Windows |
|---|---|---|---|
| W1 | Lancement de `souriso.exe` sans variable d'environnement | `souriso-sonde.txt` : backend `Dx12`, modes alpha contenant `PreMultiplied` ou `PostMultiplied`. Pas de code de sortie 2 | non testé |
| W2 | Transparence | Bureau visible, aucun fond noir ni blanc, aucun cadre, aucune ombre, y compris au premier affichage et après 5 minutes | non testé |
| W3 | Clics qui traversent | Cliquer, glisser, faire défiler et cliquer droit à travers l'anneau agit sur la fenêtre dessous | non testé |
| W4 | Droits | Compte standard : aucune invite UAC, aucune invite pare-feu | non testé |
| W5 | Forçage du backend | `WGPU_BACKEND=vulkan` : le constat de la sonde est lisible, l'application quitte avec le code 2 sans afficher d'écran opaque (comportement attendu, Vulkan reste opaque) | non testé |
| W6 | Fluidité (S1, S2) | Anneau sans à-coup. Décalage à noter : vidéo à 240 images/s, images d'écart entre l'anneau et le curseur | non testé |
| W7 | Processeur (S3, S4) | Compteur de performance du processus : en mouvement continu 60 s, noter le % d'un cœur logique (seuil < 3 %). Curseur immobile 60 s : noter (seuil < 1 %) | non testé |
| W8 | Applications en plein écran | Anneau visible au-dessus d'un diaporama, d'un navigateur en plein écran et d'un jeu en fenêtre sans bordure. Noter aussi si une vidéo plein écran en dessous change d'aspect ou saccade | non testé |
| W9 | Mise à l'échelle (S10) | Écran à 100 % puis à 150 % : anneau centré à ± 1 px logique (écran principal) | non testé |
| W10 | Enregistrement (S11) | OBS en capture d'écran entière : l'anneau est visible dans la vidéo. Noter la capture de fenêtre et la Barre de jeu | non testé |
| W11 | Autre carte graphique | Si une machine à carte intégrée est disponible : mêmes scénarios W1 à W3 | non testé |

Automatique : `cargo clippy --target x86_64-pc-windows-msvc` et `--target aarch64-apple-darwin` passent depuis Linux (vérification croisée, qui ne remplace pas W1 à W11).
