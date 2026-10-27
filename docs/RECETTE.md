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

## feat/projecteur

Effet projecteur : un voile par écran, zone éclairée aux bords adoucis autour du curseur, raccourci global **Ctrl + Alt + Maj + P** pour activer ou couper. Configuration unique (`src/coeur/config.rs`, valeurs par défaut : rayon 150 px logiques, adoucissement 100 px, intensité 0,6, couleur noire), sans interface de réglage. Interface commune des effets. Aide de mesure du processeur écrite à la fermeture. Le repère en anneau du socle est remplacé par le projecteur.

**État au moment de la livraison** : la CI et la vérification croisée depuis Linux valident la compilation, les tests du cœur et la validité du shader. Rien de ce qui suit n'a été exécuté sur Windows ni sur macOS par l'assistant : tout est « non testé », y compris tout chiffre de processeur.

**Lire le bilan de fermeture.** À la fermeture par l'icône (« Quitter »), `souriso-sonde.txt` reçoit une ligne `Bilan : durée d'exécution … s, temps processeur … s, images dessinées … (… images/s en moyenne)`. Depuis `fix/performance-projecteur`, elle est aussi réécrite toutes les 10 secondes (voir plus bas) : un processus tué garde le dernier bilan.
Part d'un cœur logique (%) = temps processeur ÷ durée d'exécution × 100 : 3 s de processeur sur 60 s donnent 5 %. La durée inclut le lancement : pour S3, bouger la souris en continu 60 s juste après le lancement puis quitter ; pour S4, laisser le curseur immobile 60 s puis quitter.
Cadence : le chiffre d'images par seconde est une moyenne sur toute l'exécution, repos compris (aucune image n'est dessinée à l'arrêt) ; une moyenne basse n'est donc pas un défaut en soi.

| # | Scénario | Attendu | Windows | macOS |
|---|---|---|---|---|
| W12 | Projecteur sur un écran | Écran assombri, zone éclairée centrée sur le curseur, bords adoucis (sans anneau net), le curseur reste visible | non testé | non testé |
| W13 | Deux écrans ou plus | Un voile sur chaque écran. Le trou lumineux suit le curseur sur l'écran où il se trouve ; l'autre écran est entièrement voilé. Aucun écran ne perd son voile quand le curseur passe de l'un à l'autre, dans les deux sens | non testé | non testé |
| W14 | Changement d'écran | Franchir la jonction lentement puis vite, aller-retour : pas de trou fantôme sur l'ancien écran, pas de clignotement, pas de retard visible du trou | non testé | non testé |
| W15 | Plein écran d'une autre app | Diaporama, navigateur et jeu sans bordure en plein écran sur un écran : voile et trou visibles. Noter si une vidéo plein écran dessous saccade ou change d'aspect (le voile peut empêcher la présentation directe) | non testé | non testé |
| W16 | Raccourci global | Ctrl + Alt + Maj + P depuis une autre application au premier plan : le voile disparaît, puis revient. Une pression longue ne fait pas clignoter. Fonctionne sans élévation | non testé | non testé |
| W17 | Conflit du raccourci | Si le raccourci est déjà pris par un autre logiciel ou par le système : l'application démarre quand même, le projecteur reste actif en permanence et la sonde contient « raccourci global indisponible ». Noter toute combinaison réservée constatée | non testé | non testé |
| W18 | Haute densité et échelles mixtes | Écrans à 100 % et 150 % (ou Retina et externe) : rayon visuellement identique, trou centré à ± 1 px logique sur chaque écran | non testé | non testé |
| W19 | Traversée des clics | Cliquer, glisser, faire défiler et cliquer droit à travers le voile agit sur la fenêtre dessous, sur chaque écran | non testé | non testé |
| W20 | Processeur (S3, S4) avec voile | Mêmes mesures que W7 avec la ligne « Bilan », sur un puis deux écrans. Seuils : < 3 % en mouvement, < 1 % au repos. Noter aussi la charge GPU du Gestionnaire des tâches | non testé | non testé |
| W21 | Bilan de fermeture | Après « Quitter », la ligne « Bilan » est dans `souriso-sonde.txt` avec une durée, un temps processeur et un nombre d'images cohérents. L'application se comporte comme avant | non testé | non testé |
| W22 | Projecteur coupé | Voile coupé : écran entièrement transparent, aucune image dessinée tant que rien ne change (le compteur d'images du bilan n'augmente pas) | non testé | non testé |

### Fonctions de la couche plateforme (ajouts)

| Fonction | Windows | macOS |
|---|---|---|
| Liste des écrans (position, taille, échelle) | non testé | non testé |
| Raccourci global (`RegisterHotKey` / Carbon par `global-hotkey`) | non testé | non testé |
| Temps processeur du processus (`GetProcessTimes` / `getrusage`) | non testé | non testé |
| Branchement ou retrait d'un écran en cours d'exécution | non géré (relancer l'application) | non géré |

### Vérifications automatiques

- [x] Format, lint, tests (11) et frontière système réussissent sous Linux ; `cargo clippy --all-targets --locked -- -D warnings` réussit aussi avec `--target x86_64-pc-windows-msvc` et `--target aarch64-apple-darwin` (vérification croisée, qui ne remplace pas W12 à W22).
- [ ] La CI est verte sur Windows et sur macOS.

### Cases à cocher (feat/projecteur)

- [ ] W12 : projecteur sur un écran, bords adoucis (Windows, macOS)
- [ ] W13 : un voile par écran, trou sur l'écran du curseur (Windows, macOS)
- [ ] W14 : changement d'écran sans trou fantôme ni clignotement (Windows, macOS)
- [ ] W15 : visible au-dessus d'une app en plein écran, vidéo dessous inchangée (Windows, macOS)
- [ ] W16 : raccourci Ctrl + Alt + Maj + P active et coupe depuis une autre app (Windows, macOS)
- [ ] W17 : raccourci refusé par le système : l'application démarre, la sonde le signale (Windows, macOS)
- [ ] W18 : échelles mixtes, trou centré à ± 1 px logique (Windows, macOS)
- [ ] W19 : clics traversants sur chaque écran (Windows, macOS)
- [ ] W20 : processeur S3 < 3 % et S4 < 1 %, un puis deux écrans, calculés avec la ligne « Bilan » (Windows, macOS)
- [ ] W21 : ligne « Bilan » cohérente après « Quitter » (Windows, macOS)
- [ ] W22 : voile coupé, aucune image dessinée au repos (Windows, macOS)

## fix/performance-projecteur

Réduction du coût processeur du projecteur (S3 : < 3 % d'un cœur logique). Rien ne change à l'écran : voile, trou lumineux, bords adoucis et suivi du curseur sont identiques. Ni la traversée des clics, ni la présentation par composition DX12, ni la frontière système ne sont touchées.

**Point de départ, relevé par le propriétaire sur Windows** (Quadro P620, DX12 en composition, 3 écrans, projecteur actif, curseur en mouvement continu 60 s) : 10,9 % d'un cœur logique. Une exécution mixte de 198,803 s a donné 9,219 s de processeur et 6612 images, soit 33,26 images/s et environ 1,4 ms de processeur par image (démarrage et repos compris : ordre de grandeur, pas une mesure propre).

**Ce qui change**

- **Sonde (A).** Le bilan sépare le temps noyau et le temps utilisateur, et ajoute : images par écran, réveils de la boucle, lectures du curseur, échecs d'acquisition d'image, reconfigurations de la surface. Il est écrit dans `souriso-sonde.txt` toutes les 10 s, en remplaçant la dernière ligne : une fermeture brutale ne le perd pas. Une ligne écrite entre deux bilans (« Projecteur activé ») passe devant lui.
- **Échec d'acquisition (B).** Sur `Timeout` ou `Occluded`, l'image est sautée et réessayée au passage suivant, sans reconfigurer la surface. Seuls `Outdated`, `Lost` et `Validation` la reconfigurent.
- **Position du curseur (C).** Elle est relue après l'acquisition de l'image, juste avant le dessin, et non plus lors du réveil précédent.
- **Reporté** : sondage réduit au repos (D), plafond à 60 images/s (E, les taux de rafraîchissement n'ont pas été communiqués), événements souris bruts (F).

**Méthode de calcul** (à appliquer aux deux scénarios, un écran de chaque taux noté au préalable)

1. Part d'un cœur logique (%) = temps processeur ÷ durée × 100, sur une fenêtre de 60 s hors démarrage : relever `(Get-Process souriso).CPU` (secondes de processeur, noyau et utilisateur) à l'instant t0 puis à t0 + 60 s, dans PowerShell ; part = (CPU(t0 + 60) − CPU(t0)) ÷ 60 × 100. Le résultat ne dépend pas du nombre de cœurs.
2. Recoupement avec le bilan : à « Quitter » (ou en lisant le dernier bilan écrit), part = temps processeur ÷ durée d'exécution × 100. La durée inclut le démarrage : c'est un contrôle, pas la valeur de référence.
3. Coût par image (ms) = temps processeur (s) ÷ images dessinées × 1000. Part du noyau (%) = temps noyau ÷ temps processeur × 100.
4. Réveils par seconde = réveils ÷ durée ; lectures du curseur par seconde = lectures ÷ durée.
5. Pour une fenêtre propre avec le bilan seul : noter la ligne du bilan à t0 (fichier lu pendant l'exécution, à ± 10 s) et celle de t0 + 60 s, puis soustraire les compteurs et les temps.

| # | Scénario (Windows) | Attendu | Windows |
|---|---|---|---|
| W23 | Usage actif, 60 s : 3 écrans, projecteur actif, curseur en mouvement continu | Part d'un cœur logique (méthode 1) à noter pour S3 (< 3 %), avec coût par image, part du noyau, images par écran, échecs d'acquisition et reconfigurations. Comparer au point de départ de 10,9 % | non testé |
| W24 | Repos, 60 s : mêmes réglages, curseur immobile | Part d'un cœur logique à noter pour S4 (< 1 %), avec réveils et lectures du curseur par seconde. Aucune image dessinée (cf. W22) | non testé |
| W25 | Échecs d'acquisition | Pendant W23, `échecs d'acquisition` et `reconfigurations de surface` du bilan : noter les valeurs. Une valeur élevée désigne la cause à traiter ensuite | non testé |
| W26 | Bilan en cours | Pendant l'exécution, `souriso-sonde.txt` contient une seule ligne « Bilan » qui change toutes les 10 s et reste la dernière ligne. Tuer le processus par le Gestionnaire des tâches : le dernier bilan est conservé | non testé |
| W27 | Apparence inchangée (voile, trou, bords adoucis, suivi du curseur) | Identique à W12 à W14, y compris au franchissement de la jonction. Noter si le trou semble en avance ou en retard sur le curseur matériel | non testé |
| W28 | Traversée des clics et plein écran | Identique à W19 et W15 | non testé |

Aucun chiffre de ce tableau n'a été relevé : tout est « non testé » jusqu'à ce que le propriétaire le renseigne. Même mesure à refaire sur macOS : non testé.

### Fonctions de la couche plateforme (modifiée)

| Fonction | Windows | macOS |
|---|---|---|
| Temps processeur séparé noyau et utilisateur (`GetProcessTimes` / `getrusage`) | non testé | non testé |

### Vérifications automatiques

- [x] Format, lint (`-D warnings`), 13 tests, frontière système et compilation release réussissent sous Linux ; le lint réussit aussi avec `--target x86_64-pc-windows-msvc` et `--target aarch64-apple-darwin` (vérification croisée, qui ne remplace pas W23 à W28).
- [ ] La CI est verte sur Windows et sur macOS.

### Cases à cocher (fix/performance-projecteur)

- [ ] W23 : usage actif, part d'un cœur logique relevée (Windows)
- [ ] W24 : repos, part d'un cœur logique relevée (Windows)
- [ ] W25 : échecs d'acquisition et reconfigurations relevés (Windows)
- [ ] W26 : bilan en cours, une seule ligne, conservé si le processus est tué (Windows, macOS)
- [ ] W27 : apparence inchangée (Windows, macOS)
- [ ] W28 : clics traversants et plein écran inchangés (Windows, macOS)
