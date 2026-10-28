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

## fix/cout-presentation-projecteur

Les taux de rafraîchissement des écrans ne sont pas connus, et la piste E (plafond à 60 images/s, reportée par `fix/performance-projecteur`) en dépend. La ligne de sonde de chaque écran donne maintenant ce taux, lu par le système (winit : `EnumDisplaySettings` sous Windows, mode d'affichage CoreGraphics sous macOS). Rien ne change dans l'application : seule la sonde en dit plus.

**Évaluation de la piste E avec ce taux.** La présentation est en `Fifo` : le nombre d'images par seconde d'un écran est borné par son taux. Un plafond à 60 images/s ne coûte donc rien sur un écran à 60 Hz, et fait perdre de la fluidité (effet en retard, cf. question ouverte 3 de `docs/ARCHITECTURE.md`) sur un écran à 120 Hz ou plus. Après W29, rapprocher chaque taux des images par écran du bilan de W23 : si un écran dessine nettement plus de 60 images/s, E économiserait du processeur sur lui, au prix de la fluidité ; sinon E n'apporte rien. **Constat du propriétaire (Windows, hors sonde) :** les écrans tournent autour de 60 Hz. Le plafond de E n'apporte donc rien : E est écartée (W29 le confirmera par les taux de la sonde).

**Piste G — choix de l'adaptateur graphique (reportée tant que le test n'est pas fait).**

*Constat du propriétaire :* deux cartes, une NVIDIA Quadro P620 sans mode d'affichage actif rapporté et une Intel UHD Graphics à 59 Hz. L'application rend sur la Quadro alors que les écrans semblent branchés sur l'Intel. Hypothèse : à chaque image, le système recopie le rendu d'une carte à l'autre, ce qui expliquerait un temps noyau élevé. Elle n'est pas vérifiée : la part du noyau du bilan la rend plausible sans la prouver.

*Comment l'application choisit aujourd'hui* (`Gpu::nouveau`, `src/rendu/mod.rs`) : `request_adapter` reçoit la surface de la première fenêtre (`compatible_surface`) et le reste par défaut, donc `power_preference: None`. Avec `None`, wgpu ne trie pas : il prend le premier adaptateur de l'énumération du backend (DX12) dont la surface est utilisable. Le critère « compatible avec la surface » écarte un adaptateur qui ne sait pas présenter ; on ne sait pas s'il écarte celui qui ne porte pas l'écran (à vérifier par la sonde). Ni l'écran qui porte les surcouches ni la préférence graphique ne sont consultés. Les variables `WGPU_POWER_PREF` et `WGPU_ADAPTER_NAME` n'ont aucun effet : `with_env()` ne lit que le backend, les drapeaux et les options de backend, et le code n'appelle ni `PowerPreference::from_env` ni `initialize_adapter_from_env`. Seul l'adaptateur retenu est écrit dans la sonde (« Sonde : adaptateur … »), pas la liste des adaptateurs.

*Options et coût en rendu :*

| Option | Principe | Coût et risques |
|---|---|---|
| G1. Préférence économe (`LowPower`) | Trier les adaptateurs en préférant l'intégré (Intel) | Une ligne de code. Le rendu est un plein écran très simple (voile, trou, bords adoucis) : l'Intel devrait suffire. Si les écrans sont bien sur l'Intel, la recopie disparaît. Si le système trie autrement, rien ne change. Sur une machine qui n'a qu'une carte, aucun effet. |
| G2. Préférence performante (`HighPerformance`) | Préférer la carte discrète (Quadro) | Revient sans doute à l'état actuel. Utile surtout comme contre-épreuve. |
| G3. Adaptateur qui porte les écrans | Interroger le système pour savoir quelle carte porte chaque écran, et prendre celle-là | Le plus sûr, mais le plus coûteux : il faut un appel système (DXGI sous Windows) dans la couche plateforme, du code `unsafe`, une version macOS, et un cas où les trois écrans sont répartis sur deux cartes (un adaptateur par écran). Ni winit ni wgpu ne donnent cette information. |
| G4. Réglage système par application | Réglage Windows « Paramètres > Affichage > Graphiques » pour `souriso.exe` | Aucun code. Non fiable pour un installeur grand public, mais utile pour tester : à confirmer par la sonde que l'adaptateur change. |

*Ce que le propriétaire doit tester pour valider (Windows) :*

1. Relever l'adaptateur et le backend de la ligne de sonde au lancement (cas actuel : attendu Quadro P620, DX12).
2. Relever la part d'un cœur logique, la part du noyau et le coût par image (méthode de W23) dans le cas actuel.
3. Forcer l'adaptateur économe : par le réglage Windows de G4 (les variables `WGPU_*` sont sans effet dans le code actuel). Vérifier d'abord dans la sonde que l'adaptateur affiché est bien l'Intel ; sinon le test n'a rien mesuré.
4. Refaire les mesures de l'étape 2 sur l'Intel. La piste est validée si la part du noyau et le coût par image baissent nettement, avec une apparence identique (W27) et sans échec d'acquisition (W25). Elle est écartée si elles ne bougent pas.
5. Noter à quelle carte chaque écran est branché (Paramètres Windows > Affichage > Affichage avancé, ou la carte indiquée par écran) : c'est ce qui départage G1 et G3.

*Ce qui change (version minimale, rien d'autre).* L'application lit `WGPU_ADAPTER_NAME` (fragment du nom, sans casse, parmi les adaptateurs qui peuvent présenter) puis `WGPU_POWER_PREF` (`low`, `high`, `none`). Sans l'une ni l'autre, le choix est celui d'avant (`PowerPreference::None`). Un nom sans correspondance est signalé dans la sonde et le choix par défaut s'applique. La sonde écrit en plus la liste des adaptateurs que wgpu énumère (nom, type intégré ou dédié, backend, présentation sur la surface possible ou impossible) et le type de l'adaptateur retenu. Aucune interrogation DXGI : la frontière système, la traversée des clics, la présentation par composition et le rendu visible ne sont pas touchés.

*Constat dans `wgpu-hal` 30.0.1 (code, pas mesure).* Les capacités d'une surface DX12 ne dépendent pas de l'adaptateur : les modes alpha de la surface de composition sont les mêmes pour l'Intel et la Quadro, donc l'Intel peut en principe présenter. Pour la même raison, `compatible_surface` ne départage pas les cartes, et le défaut prend la première de l'énumération. À confirmer par la ligne « présentation sur la surface » de la sonde.

*Résultat du test de l'adaptateur économe :* en attente du propriétaire.

| # | Scénario | Attendu | Windows | macOS |
|---|---|---|---|---|
| W29 | Taux de rafraîchissement dans la sonde | Au lancement, chaque ligne « Sonde : écran N » indique un taux de rafraîchissement en Hz qui correspond aux réglages d'affichage du système (ou « inconnu » si le système ne le donne pas). Noter le taux de chacun des 3 écrans. L'application se comporte comme avant | non testé | non testé |
| W30 | Adaptateur graphique (piste G) : trois lancements, chacun avec 60 s de mouvement continu, 3 écrans : défaut (aucune variable), `WGPU_POWER_PREF=low`, `WGPU_POWER_PREF=high` | Pour chaque lancement, noter la ligne « Sonde : adaptateur » (nom, type, backend), la part d'un cœur logique (méthode 1), la part du noyau et le coût par image (méthode 3). Contrôler d'abord la liste « adaptateur disponible » : les deux cartes y figurent et la présentation est « possible » pour l'Intel. Apparence identique (W27), sans échec d'acquisition (W25). Valide si noyau et coût par image baissent nettement avec un adaptateur autre que le défaut | non testé | non testé |

### Vérifications automatiques

- [x] Format, lint (`-D warnings`), 16 tests, frontière système et compilation release réussissent sous Linux ; le lint réussit aussi avec `--target x86_64-pc-windows-msvc` et `--target aarch64-apple-darwin` (vérification croisée, qui ne remplace pas W29).
- [ ] La CI est verte sur Windows et sur macOS.

### Cases à cocher (fix/cout-presentation-projecteur)

- [ ] W29 : taux de rafraîchissement relevés pour chaque écran (Windows, macOS)
- [ ] W30 : adaptateur graphique, trois lancements (défaut, low, high) mesurés (Windows)

## fix/adaptateur-par-defaut

Sans variable d'environnement, l'application rendait sur la Quadro P620 (choix de wgpu) alors que les écrans sont portés par l'Intel UHD : forcer l'Intel par son nom a fait passer le processeur en mouvement continu de 6 à 11 % à 4,0 % d'un cœur logique (seuil S3 : 3 %). Le coût par image sur la Quadro était de 1,2 à 1,5 ms, dont 57 % de temps noyau, cohérent avec une recopie entre cartes.

**Ce qui change.** Par défaut, l'application choisit la carte qui porte l'écran principal, demandée à la couche plateforme (`plateforme::cartes_ecrans`). Ordre complet, mécanisme et limites : `docs/ARCHITECTURE.md`, 4.2. Les variables `WGPU_ADAPTER_NAME` et `WGPU_POWER_PREF` restent prioritaires. Le pilote logiciel (type processeur) n'est retenu que faute d'autre carte. La sonde écrit, par écran, la carte qui le porte (« Sonde : écran N porté par la carte … ») et le critère décisif dans la ligne « Sonde : adaptateur … critère : … ». Rien ne change dans le rendu visible, la traversée des clics, la présentation par composition ni l'enchaînement par image.

**Anomalie à vérifier (W34).** L'énumération de la sonde donne l'Intel typé « intégré », avec présentation possible. Dans `wgpu-core` 30.0.1, `LowPower` trie pourtant l'intégré avant le dédié ; `WGPU_POWER_PREF=low` devrait donc retenir l'Intel. Le propriétaire soupçonne un fichier de sonde issu de cet essai citant la Quadro, sans résultat confirmé. Non corrigée dans cette MR : la cause n'est pas établie.

### Fonctions de la couche plateforme (modifiée)

| Fonction | Windows | macOS |
|---|---|---|
| `cartes_ecrans` (carte qui porte chaque écran) | non testé (`EnumDisplayDevicesW`, `EnumDisplaySettingsW`) | sans objet : aucun avis, comportement inchangé (non testé) |

| # | Scénario | Attendu | Windows | macOS |
|---|---|---|---|---|
| W31 | Re-mesure par défaut : lancer sans aucune variable `WGPU_*`, puis 60 s de mouvement continu, 3 écrans | La liste « adaptateur disponible » montre les trois adaptateurs ; chaque ligne « écran N porté par la carte » cite l'Intel UHD ; la ligne « Sonde : adaptateur » cite l'Intel UHD (intégré) avec le critère « carte qui porte les écrans ». Noter la part d'un cœur logique (méthode 1), la part du noyau et le coût par image (méthode 3). Valide si l'on retrouve environ 4 % d'un cœur et un coût par image nettement plus bas que sur la Quadro (1,2 à 1,5 ms, 57 % de noyau), apparence identique (W27), sans échec d'acquisition (W25) | non testé | sans objet |
| W32 | Poste fixe dont la carte dédiée porte les écrans (intégrée active, ou absente) | Chaque écran est dit porté par la carte dédiée ; la ligne « Sonde : adaptateur » cite la dédiée avec le critère « carte qui porte les écrans » ; processeur et coût par image au moins aussi bons qu'avec l'intégrée forcée par `WGPU_ADAPTER_NAME` | non testé | sans objet |
| W33 | Priorité des variables : lancer avec `WGPU_ADAPTER_NAME=Quadro`, puis avec `WGPU_POWER_PREF=high`, puis avec un nom inexistant | Le critère de la sonde est « variable WGPU_ADAPTER_NAME », puis « variable WGPU_POWER_PREF » ; avec un nom inexistant, la sonde le signale et le choix suit les étapes suivantes | non testé | non testé |
| W34 | Anomalie : `WGPU_POWER_PREF=low`, sans `WGPU_ADAPTER_NAME` | La ligne « Sonde : adaptateur » cite l'Intel UHD. Si elle cite la Quadro : conserver le fichier de sonde (liste des adaptateurs comprise) et le joindre au signalement | non testé | non testé |
| W35 | Écrans répartis entre deux cartes (si le matériel existe) | Chaque ligne « écran N porté par la carte » donne la carte réelle ; l'adaptateur retenu est celui de l'écran principal ; noter le coût des écrans de l'autre carte | non testé | sans objet |
| W36 | macOS : lancement sans variable | Aucune ligne de carte par écran utile (« inconnue ») ; critère « choix de wgpu » ; comportement et apparence inchangés | sans objet | non testé |

### Vérifications automatiques

- [x] Format, lint (`-D warnings`), 21 tests, frontière système et compilation release réussissent sous Linux ; le lint réussit aussi avec `--target x86_64-pc-windows-msvc` et `--target aarch64-apple-darwin` (vérification croisée, qui ne remplace pas W31).
- [ ] La CI est verte sur Windows et sur macOS.

### Cases à cocher (fix/adaptateur-par-defaut)

- [ ] W31 : re-mesure par défaut, ligne d'adaptateur = Intel (Windows)
- [ ] W32 : poste fixe, carte dédiée qui porte les écrans (Windows, non testé)
- [ ] W33 : priorité des variables (Windows)
- [ ] W34 : anomalie `WGPU_POWER_PREF=low` (Windows)
- [ ] W35 : écrans répartis entre deux cartes (Windows, si le matériel existe)
- [ ] W36 : macOS inchangé

## feat/effets-clic

Quatre effets s'ajoutent au projecteur : anneau, traînée, ondulation au clic (rendus distincts pour le clic gauche et le clic droit) et son de clic. Chacun s'active et se coupe indépendamment au clavier (Ctrl + Alt + Maj + P projecteur, A anneau, T traînée, O ondulation, S son) ; leurs paramètres sont dans `coeur::config::Config`, sans interface de réglage. Mécanisme, budget et limites : `docs/ARCHITECTURE.md` (section 9), sons : `docs/SONS.md`.

**Rien de ce qui suit n'est mesuré ni testé sous Windows ou macOS.** Les chiffres de coût sont à relever par le propriétaire.

**Limites assumées, à constater en recette.**

- Les clics sont lus par interrogation des boutons toutes les 8 ms : **un appui plus court que cette période peut être manqué** (ni ondulation ni son).
- Un son qui arrive pendant un autre **coupe le précédent**.
- **La latence entre le clic et le son est inconnue** : elle dépend du système, du pilote audio et du premier appel (initialisation du lecteur). À mesurer (W46).
- Un clic maintenu ne donne qu'une ondulation et qu'un son (front d'appui).
- Au plus 8 ondulations simultanées par écran ; au-delà, la plus ancienne disparaît. Au plus 128 points de traînée.
- Sur une surface sans alpha prémultiplié (repli `PostMultiplied`), les effets ne se mélangent pas : le dernier écrase les autres (déjà noté dans `rendu::effet::Sortie`).

**Budget de coût (structurel, retenu avec le propriétaire).** Une seule présentation par écran et par image, tous les effets dans la même passe ; aucune image tant que rien ne change ; les animations ne demandent des images que pendant leur durée ; S4 sous 1 % d'un cœur logique. S3 n'est pas visé par cette modification : il est déjà dépassé avec le projecteur seul (4,0 à 5,3 % sur l'Intel) et ses seuils seront révisés dans une modification séparée. On vise seulement à ne pas faire monter ce coût de façon notable.

### Fonctions de la couche plateforme (nouvelles ou modifiées)

| Fonction | Windows | macOS |
|---|---|---|
| `boutons` (état des boutons gauche et droit, sens logique) | non testé (`GetAsyncKeyState`, `GetSystemMetrics(SM_SWAPBUTTON)`) | non testé (`CGEventSourceButtonState`, aucune invite attendue) |
| `jouer_son` (WAV en mémoire, asynchrone) | non testé (`PlaySoundW`) | non testé (`NSSound`) |
| `creer_raccourcis` (cinq raccourcis, un seul gestionnaire) | non testé | non testé |

| # | Scénario | Attendu | Windows | macOS |
|---|---|---|---|---|
| W37 | Cumul : les cinq effets actifs, curseur en mouvement puis clics gauche et droit | Voile, traînée, anneau et ondulation se superposent sans fond noir ni cadre ; l'ordre est voile < traînée < anneau < ondulation ; le centre de l'écran reste lisible | non testé | non testé |
| W38 | Anneau : taille, épaisseur, couleur, opacité (modifier `Config::default`, recompiler) | L'anneau est centré sur le curseur à ± 1 px logique, à 100 % et 150 % d'échelle, y compris au franchissement de la jonction de deux écrans (S10) ; bords adoucis (S15) | non testé | non testé |
| W39 | Traînée : longueur et durée de fondu | La traînée suit le chemin sans trous aux mouvements rapides, s'efface en environ `duree_fondu` après l'arrêt, est plus courte quand le fondu est plus rapide. Après l'arrêt du curseur elle disparaît complètement (aucun résidu) | non testé | non testé |
| W40 | Ondulation : clic gauche puis clic droit | Gauche : un anneau bleu sur un léger disque. Droit : deux anneaux concentriques orange. Les deux s'élargissent en s'effaçant puis disparaissent sans coupure sèche | non testé | non testé |
| W41 | Clic maintenu 2 s | Une seule ondulation et un seul son au moment de l'appui. Dans le bilan : clics gauche + 1, sons joués + 1 | non testé | non testé |
| W42 | Clics rapides répétés : 20 clics en 3 s, puis 10 clics en 1 s, gauche et droit mélangés | Pas de blocage ni de retard cumulé ; au plus 8 ondulations visibles à la fois ; le son se coupe et se relance sans crépitement gênant (limite acceptée : un son coupe le précédent). Comparer le nombre de clics faits au bilan : noter les clics manqués | non testé | non testé |
| W43 | Boutons de la souris permutés (Paramètres Windows > Bluetooth et appareils > Souris > Bouton principal : Droit) | Le clic fait avec le bouton qui est le « gauche » pour l'utilisateur donne le rendu gauche et compte dans « clics gauche » ; l'autre le rendu droit | non testé | sans objet |
| W44 | Clic très bref (tapotement sur un pavé tactile ou une souris à très faible durée d'appui) | Constater si certains clics sont manqués (limite acceptée : appui plus court que 8 ms). Noter la proportion | non testé | non testé |
| W45 | Désactivation effet par effet : Ctrl + Alt + Maj + A, T, O, S, P, chacun deux fois | Chaque raccourci coupe puis rétablit uniquement son effet (ligne « Anneau coupé », etc. dans la sonde) ; les autres ne changent pas ; effet coupé = plus d'image demandée pour lui (si tout est coupé, le nombre d'images par écran ne bouge plus). Une combinaison refusée par le système est signalée dans la sonde sans empêcher les autres | non testé | non testé |
| W46 | Son : volume, choix, latence | Le son est audible au volume réglé ; changer `Config::son.choix` (clic, bulle, cloche) et `volume` change ce qu'on entend. Mesurer l'écart clic → son (enregistrement écran + audio, comme R10) et noter le premier son après le lancement séparément des suivants. Rapporter si la latence gêne | non testé | non testé |
| W47 | Plusieurs écrans : clics sur chacun | L'ondulation apparaît sur l'écran où est le curseur, centrée sur le clic (à ± 1 px logique) ; un seul son par clic, quel que soit le nombre d'écrans | non testé | non testé |
| W48 | S3, mouvement continu 60 s avec **tous** les effets actifs, mêmes conditions que W31 | Noter la part d'un cœur logique (méthode de W7, ligne « Bilan ») et la comparer au projecteur seul (4,0 à 5,3 % sur l'Intel). **Attendu : pas de hausse notable.** Le seuil de 3 % n'est pas l'objectif ici. Noter aussi les images/s (S1) : la cadence doit rester celle du projecteur seul | non testé | non testé |
| W49 | S4, curseur immobile 60 s **une fois les animations terminées** (attendre 2 s après le dernier clic ou mouvement), tous les effets actifs | Part d'un cœur logique sous 1 % (méthode de W7) | non testé | non testé |
| W50 | Images par écran au repos : relever la ligne « Bilan » (écriture toutes les 10 s), attendre 60 s sans bouger ni cliquer, relever à nouveau | Le nombre d'images par écran n'a pas augmenté (à une ou deux images près pour la fin de l'animation). S'il augmente continuellement, une animation ne s'arrête pas : conserver le fichier de sonde | non testé | non testé |
| W51 | Ligne « Bilan » : clics et sons | Faire exactement 10 clics gauche et 4 clics droits à vitesse normale : « clics gauche 10, clics droit 4, sons joués 14 ». Couper le son (Ctrl + Alt + Maj + S), refaire 3 clics : les clics augmentent, les sons joués non | non testé | non testé |
| W52 | macOS : premier lancement sur un compte neuf | Aucune invite de permission (notamment « Surveillance de l'entrée ») pour la détection des clics ni pour le son ; les clics gauche et droit sont détectés ; avec « bouton principal » permuté dans les réglages de la souris, noter quel bouton est vu comme gauche | sans objet | non testé |

### Vérifications automatiques

- [x] Format, lint (`-D warnings`), 47 tests, frontière système et compilation release réussissent sous Linux. Le lint réussit aussi avec `--target x86_64-pc-windows-msvc` et `--target aarch64-apple-darwin` (vérification croisée, qui ne remplace pas la recette).
- [x] Un essai graphique (`rendu::essai_gpu`) crée les quatre pipelines, dessine tous les effets dans une seule passe hors écran sur le pilote logiciel Vulkan de Linux, vérifie la présence de l'anneau, de l'ondulation et de la traînée puis leur disparition, et que plus aucune animation n'est en cours. Il est ignoré, avec un message, sur une machine sans adaptateur graphique. Il ne mesure rien et ne remplace ni W37 ni W48.
- [ ] La CI est verte sur Windows et sur macOS.

### Cases à cocher (feat/effets-clic)

- [ ] W37 : cumul des effets (Windows, macOS)
- [ ] W38 : anneau (Windows, macOS)
- [ ] W39 : traînée (Windows, macOS)
- [ ] W40 : ondulation gauche et droite distinctes (Windows, macOS)
- [ ] W41 : clic maintenu (Windows, macOS)
- [ ] W42 : clics rapides répétés (Windows, macOS)
- [ ] W43 : boutons permutés (Windows)
- [ ] W44 : clics très brefs manqués, proportion notée (Windows, macOS)
- [ ] W45 : désactivation effet par effet (Windows, macOS)
- [ ] W46 : son, volume, choix, latence (Windows, macOS)
- [ ] W47 : plusieurs écrans (Windows, macOS)
- [ ] W48 : S3 en mouvement continu, tous effets actifs (Windows, macOS)
- [ ] W49 : S4 au repos 60 s après les animations (Windows, macOS)
- [ ] W50 : images par écran au repos dans le bilan (Windows, macOS)
- [ ] W51 : clics et sons dans le bilan (Windows, macOS)
- [ ] W52 : macOS, aucune invite de permission (macOS)

## feat/profils

Profils nommés (par défaut, « Présentation », « Enregistrement »), fichier `profils.json`, raccourcis de profils, réglage de l'adaptateur. Tout reste « non testé » : rien n'a pu être essayé sous Windows ni macOS. Dossier des profils : `%APPDATA%\SOURISO` (Windows), `~/Library/Application Support/SOURISO` (macOS). Raccourcis par défaut (Ctrl + Alt + Maj + …) : D Par défaut, E Présentation, R Enregistrement, N profil suivant.

### Fonctions de la couche plateforme (nouvelles ou modifiées)

| Fonction | Windows | macOS |
|---|---|---|
| `dossier_donnees` (dossier des profils, sans droit particulier) | non testé (`%APPDATA%`) | non testé (`~/Library/Application Support`) |
| `creer_raccourcis` (jusqu'à 9 raccourcis : cinq effets, profil suivant, trois profils) | non testé | non testé |

### Scénarios

| # | Scénario | Attendu | Windows | macOS |
|---|---|---|---|---|
| W53 | Premier lancement : supprimer le dossier des profils, lancer sans droit administrateur | Le dossier et `profils.json` sont créés. La sonde dit « premier lancement : profils d'exemple créés » et « profil actif « Par défaut » (1 sur 3) ». Le fichier contient `"version": 2` et trois profils, chacun avec un bloc `actifs` (tous les effets à vrai) | non testé | non testé |
| W54 | Persistance après redémarrage : passer à « Présentation » (Ctrl + Alt + Maj + E), quitter par l'icône, relancer | La sonde dit « profil actif « Présentation » (2 sur 3) » ; l'anneau est celui de Présentation (grand, rouge) dès la première image. Refaire avec « Enregistrement », puis en tuant le processus (pas de fermeture propre) : le dernier profil choisi est restauré | non testé | non testé |
| W55 | Raccourci d'un profil : D, E, R | Chaque raccourci passe à son profil, la sonde écrit « profil « A » → « B » (raccourci du profil) ». Couper l'anneau (Ctrl + Alt + Maj + A) puis E : l'anneau revient (« Présentation » active tous les effets). Appuyer sur E alors que « Présentation » est actif : « conservé », l'activation enregistrée est rétablie, pas de changement compté | non testé | non testé |
| W56 | Profil suivant : N, quatre fois | Par défaut → Présentation → Enregistrement → Par défaut → Présentation. N applique l'activation enregistrée dans le profil atteint (voir W76). Touche maintenue 2 s : un seul changement | non testé | non testé |
| W57 | Changement de profil en pleine utilisation : curseur en mouvement continu, clics fréquents, traînée et ondulations en cours, 20 changements en 10 s (E, R, D en alternance, en respectant 0,4 s entre deux) | Aucun blocage, aucun clignotement, pas de fond noir ni de cadre ; les ondulations et la traînée en cours continuent avec les nouvelles valeurs ; le son suit le profil (cloche, bulle, clic). Dans le bilan, « changements de profil » = nombre de changements effectifs, et les images par écran n'augmentent pas plus vite qu'en mouvement continu sans changement (aucune présentation en plus) | non testé | non testé |
| W58 | Raccourci déjà pris : occuper Ctrl + Alt + Maj + E avec une autre application (ou lancer une seconde instance de SOURISO) avant le lancement | L'application démarre ; la sonde dit « raccourci global « Profil « Présentation » » indisponible : raccourci refusé par le système (déjà pris ?) » ; les autres raccourcis (D, R, N, P, A, T, O, S) fonctionnent ; Présentation reste atteignable par N. Libérer la combinaison et relancer : le raccourci fonctionne. **macOS : noter si le refus est détecté** (Carbon peut ne rien signaler quand une autre application détient la combinaison) | non testé | non testé |
| W59 | Fichier corrompu : (a) fichier vide, (b) tronqué au milieu, (c) texte quelconque, (d) `"actif": "trois"` | Démarrage normal avec les trois profils d'exemple, sans plantage ni boîte d'erreur ; la sonde dit « fichier des profils inutilisable (…) : valeurs par défaut, fichier conservé sous …profils.json.corrompu » ; ce fichier contient l'ancien contenu ; un `profils.json` valide est recréé | non testé | non testé |
| W60 | Fichier d'une autre version : mettre `"version": 3` | Traité comme W59 (mis de côté, pas écrasé sans copie), la sonde cite « version 3 du format non prise en charge » | non testé | non testé |
| W61 | Fichier modifié à la main : `"actif": 9`, deux profils de même nom, un raccourci identique à celui du projecteur (P), `intensite` à 12, un champ supprimé | Démarrage normal ; les valeurs sont ramenées dans leurs bornes, les noms rendus uniques, le raccourci en conflit retiré, le profil actif ramené au premier, le champ absent pris par défaut ; une ligne de sonde par correction | non testé | non testé |
| W62 | Dossier non inscriptible (Windows : retirer le droit d'écriture sur `profils.json` et le dossier ; macOS : `chmod -w`) puis changer de profil | L'application continue ; la sonde dit « enregistrement des profils impossible » ; le changement de profil agit quand même, mais le choix n'est pas restauré au lancement suivant | non testé | non testé |
| W63 | Dossier de données introuvable (Windows : lancer avec la variable `APPDATA` vidée) | Profils d'exemple en mémoire, sonde « dossier de données introuvable… rien ne sera enregistré » ; aucun fichier écrit ailleurs | non testé | sans objet |
| W64 | Adaptateur : mettre `"adaptateur": "econome"` puis `"performant"`, relancer à chaque fois ; ensuite `WGPU_POWER_PREF=high` avec « econome » | La ligne « Sonde : adaptateur » cite « réglage de l'adaptateur (économe ou performant) » et la carte retenue suit le réglage ; avec la variable d'environnement, le critère est « variable WGPU_POWER_PREF » (la variable l'emporte). Un changement du réglage n'agit qu'au lancement suivant | non testé | non testé |
| W65 | S4 après changements de profil : changer plusieurs fois de profil, attendre 2 s, curseur immobile 60 s, tous les effets actifs | Part d'un cœur logique sous 1 % (méthode de W7, ligne « Bilan »), et aucune image dessinée pendant la minute (comme W50) | non testé | non testé |
| W66 | Ligne « Bilan » : profil | Elle se termine par « profil actif « X », changements de profil N » ; N compte les changements effectifs (pas les « conservé ») | non testé | non testé |
| W67 | Écriture sans droit administrateur et sans réseau | Lancement avec un compte standard : aucune invite d'élévation, `profils.json` écrit dans le dossier de l'utilisateur ; aucune connexion réseau ouverte par le processus | non testé | non testé |

### Vérifications automatiques

- [x] Format, lint (`-D warnings`), 70 tests, frontière système et compilation release réussissent sous Linux. Le lint réussit aussi avec `--target x86_64-pc-windows-msvc` et `--target aarch64-apple-darwin` (vérification croisée, qui ne remplace pas la recette).
- [x] Tests du cœur : création, copie, renommage, suppression des profils (le premier est refusé), noms uniques et bornés, conflits de raccourcis, réparation d'un fichier incohérent, aller-retour du fichier, fichier absent, vide, tronqué, mal typé, d'une autre version, partiel, aux valeurs absurdes, chemin illisible.
- [x] Un essai graphique vérifie que `Effet::appliquer` change le dessin (anneau de Présentation) sans recréer les effets.
- [ ] La CI est verte sur Windows et sur macOS.

### Cases à cocher (feat/profils)

- [ ] W53 : premier lancement (Windows, macOS)
- [ ] W54 : persistance après redémarrage (Windows, macOS)
- [ ] W55 : raccourci d'un profil (Windows, macOS)
- [ ] W56 : profil suivant (Windows, macOS)
- [ ] W57 : changement de profil en pleine utilisation (Windows, macOS)
- [ ] W58 : raccourci déjà pris, détection notée (Windows, macOS)
- [ ] W59 : fichier corrompu (Windows, macOS)
- [ ] W60 : fichier d'une autre version (Windows, macOS)
- [ ] W61 : fichier modifié à la main (Windows, macOS)
- [ ] W62 : dossier non inscriptible (Windows, macOS)
- [ ] W63 : dossier de données introuvable (Windows)
- [ ] W64 : réglage de l'adaptateur et priorité des variables (Windows, macOS)
- [ ] W65 : S4 après changements de profil (Windows, macOS)
- [ ] W66 : profil dans le bilan (Windows, macOS)
- [ ] W67 : sans droit administrateur, sans réseau (Windows, macOS)

## feat/reglages

Fenêtre de réglages avec aperçu en direct, activation des effets enregistrée dans chaque profil, format du fichier des profils en version 2. Deux exécutables à côté l'un de l'autre : `souriso` et `souriso-reglages`. **Tout reste « non testé »** : rien n'a pu être lancé sous Windows ni macOS (ni même la fenêtre elle-même, faute d'écran dans l'environnement de développement). Ce qui a été vérifié sous Linux est dit en tête.

**Récupérer les deux exécutables sans rien assembler.** Chaque exécution de la CI conserve, 7 jours, un artefact par système (`souriso-Windows`, `souriso-macOS`) qui contient les deux exécutables côte à côte à la racine de l'archive. Extraire l'archive dans un dossier où l'on peut écrire, lancer `souriso` (`souriso.exe`), puis ouvrir la fenêtre par l'icône (Réglages…). Même consigne qu'au socle pour SmartScreen (Windows) et la quarantaine (macOS : `chmod +x souriso souriso-reglages && xattr -d com.apple.quarantine souriso souriso-reglages`).

### Vérifications automatiques (Linux)

- [x] Frontière système (qui porte désormais aussi sur `reglages/` et sur ses ressources distantes), format, lint `-D warnings` sur tout l'espace de travail, 99 tests du processus principal et 6 de la page, réussissent. Le script de frontière a été essayé sur quatre violations fabriquées (condition système dans le crate des réglages, dépendance propre à un système, ressource distante) : il les détecte.
- [x] Le lint du processus principal réussit aussi avec `--target x86_64-pc-windows-msvc` et `--target aarch64-apple-darwin` (vérification croisée : elle couvre le registre Windows, la boîte de message, `CREATE_NO_WINDOW`, `osascript`). **La vérification croisée du crate des réglages est impossible depuis Linux** (compilateur de ressources Windows et chaîne Objective-C absents) : il ne sera compilé pour Windows et macOS que par la CI.
- [x] Protocole (`coeur::reglages`) : chaque réglage a son champ ; les bornes de la table restent dans celles du cœur ; les profils d'exemple sont dans les bornes ; pourcentages, couleurs et sons ; refus d'une valeur absurde sans rien changer ; interrupteur écrit dans le profil ; « Revenir » ; lecture de chaque demande ; état envoyé.
- [x] Format du fichier : un fichier de version 1 est migré (tous les effets actifs, copie `profils.json.v1`, réécrit en version 2, plus de migration au deuxième lancement) ; l'activation d'un profil survit à un aller-retour ; une version 3 est mise de côté.
- [x] Raccourcis : sans Ctrl ni Alt refusé (à l'enregistrement et à la lecture), raccourci du profil suivant avec les mêmes règles.
- [x] Processus des réglages : lignes envoyées et reçues par les tuyaux, arrêt à la fermeture de l'entrée, arrêt forcé d'un processus qui l'ignore, exécutable absent.
- [x] Essai graphique : le voile est nul dans le rectangle de la fenêtre, intact à 10 px de son bord, et un rectangle vide n'est pas un trou.
- [x] Page : contraste du texte ≥ 4,5:1 et des contours et du focus ≥ 3:1 dans les deux thèmes (le plus faible : texte d'accent sur le fond, 5,57:1 en clair ; contour de contrôle sur le fond, 3,91:1 en clair), aucune ressource distante ni style ou script en ligne.
- [x] Page, essai jsdom lancé à la main (`reglages/essais/page.test.js`, 64 contrôles) : sections et navigation aux flèches, un seul message par image, bornes, couleur, interrupteur, saisie de raccourci (lettre seule refusée, Échap, Suppr), création, duplication, renommage, suppression avec confirmation, focus après chaque action, refus affichés, carte graphique grisée, version de protocole.
- [ ] La CI est verte sur Windows et sur macOS (compile et teste les deux crates, publie les deux exécutables).

### Fonctions de la couche plateforme (nouvelles)

| Fonction | Windows | macOS |
|---|---|---|
| `moteur_web_disponible` (WebView2 : clés du registre `EdgeUpdate`, machine ou utilisateur) | non testé | sans objet (WKWebView toujours présent) |
| `afficher_boite` (message natif, dans son fil, au premier plan) | non testé (`MessageBoxW`) | non testé (`osascript`) |
| `preparer_enfant` (aucune console pour le processus des réglages) | non testé (`CREATE_NO_WINDOW`) | sans objet |
| `FenetreReglages` (lancement de `souriso-reglages` à côté de `souriso`, tuyaux, arrêt) | non testé (testé sous Linux avec `cat`) | non testé (testé sous Linux avec `cat`) |
| Entrée « Réglages… » de l'icône | non testé | non testé |
| `suspendre_raccourcis` puis `creer_raccourcis` (saisie d'un raccourci) | non testé | non testé |

### Scénarios

| # | Scénario | Attendu | Windows | macOS |
|---|---|---|---|---|
| W68 | Ouverture : icône, « Réglages… » | La fenêtre s'ouvre en moins de 2 s, sans console, centrée, déjà affichée avec ses réglages (pas de page blanche ni de clignotement), sur la section Projecteur. La sonde dit « fenêtre de réglages ouverte ». Un second clic sur « Réglages… » ramène la même fenêtre devant (pas de seconde fenêtre) | non testé | non testé |
| W69 | Fermeture de la fenêtre (croix, puis Alt+F4 / Cmd+W) et mémoire | `souriso-reglages` et tous les processus du moteur web (`msedgewebview2` sous Windows, `com.apple.WebKit.*` sous macOS) ont disparu du Gestionnaire des tâches ou du Moniteur d'activité dans la seconde. La mémoire du processus principal est revenue à sa valeur d'avant l'ouverture (à quelques Mo près) ; ouvrir et fermer 10 fois ne la fait pas croître. La sonde dit « fenêtre de réglages fermée » et le bilan « ouvertures des réglages 10 » | non testé | non testé |
| W70 | S4 fenêtre fermée | Après plusieurs ouvertures et fermetures, curseur immobile 60 s, effets actifs : part d'un cœur logique sous 1 % (méthode de W7, ligne « Bilan »), aucune image dessinée pendant la minute (comme W50), aucun processus enfant | non testé | non testé |
| W71 | Fermeture avec le processus principal : ouvrir la fenêtre, quitter par l'icône | La fenêtre se ferme d'elle-même, sans erreur, avec tous les processus du moteur web ; plus aucun processus `souriso*` | non testé | non testé |
| W72 | Plantage de la fenêtre : ouvrir, puis terminer `souriso-reglages` dans le Gestionnaire des tâches | Les effets continuent sans à-coup, le voile reprend toute la place (plus de trou), la sonde dit « fenêtre de réglages fermée (processus terminé) », « Réglages… » rouvre une fenêtre | non testé | non testé |
| W73 | Aperçu en direct, un réglage à la fois | Chaque réglage de chaque page (projecteur : rayon, adoucissement, opacité, couleur ; anneau : rayon, épaisseur, opacité, couleur ; traînée : longueur, fondu, taille, opacité, couleur ; ondulation : rayon final, durée, épaisseur, opacité, couleurs gauche et droite ; son : volume et choix) change aussitôt l'effet réel autour du curseur, sans à-coup, sans fond noir, sans images en plus (comparer « images par écran » du bilan avec un mouvement continu sans réglage). Glisser un curseur 10 s ne fait pas ramer le curseur. Le son se fait entendre à chaque changement de volume ou de choix, et au bouton « Écouter » | non testé | non testé |
| W74 | Enregistrement automatique et restauration | Changer plusieurs réglages dans le profil « Présentation », attendre 1 s, quitter par l'icône (ou terminer le processus), relancer : les réglages sont restaurés dès la première image. Le fichier `profils.json` ne change qu'environ 300 ms après le dernier réglage (pas à chaque image : surveiller sa date) | non testé | non testé |
| W75 | « Revenir aux valeurs du profil » | Désactivé à l'ouverture (« Enregistré automatiquement »), actif dès le premier réglage (« Modifié depuis l'ouverture »). Un clic rétablit réglages et activation, aussitôt visibles à l'écran et dans la page. Après un changement de profil dans la fenêtre ou par raccourci, le point de retour est celui du nouveau profil | non testé | non testé |
| W76 | Activation enregistrée dans chaque profil | Dans « Enregistrement », couper le son par l'interrupteur de la page Son (effet immédiat, plus de son au clic). Passer à « Présentation » (son joué), puis revenir à « Enregistrement » par son raccourci, par le profil suivant et par la liste : le son est coupé à chaque fois. Quitter, relancer : toujours coupé dans ce profil, actif dans les autres | non testé | non testé |
| W77 | Interrupteur de la fenêtre et raccourcis de session | Le raccourci d'un effet (par exemple Ctrl + Alt + Maj + A) coupe l'effet en direct et l'interrupteur de la page (et l'indication « Coupé » de la section) suit ; le profil n'est pas modifié (« Revenir » reste grisé, le fichier ne change pas). Passer à un autre profil puis revenir : l'effet est rétabli comme l'enregistre le profil. Appuyer sur le raccourci du profil actif rétablit son activation. Quitter et relancer : l'état des raccourcis n'est pas conservé | non testé | non testé |
| W78 | Migration d'un ancien fichier | Avec un `profils.json` de version 1 (celui d'un lancement de la branche précédente) : démarrage normal, tous les effets actifs, la sonde dit « migré de la version 1 à la version 2 », `profils.json.v1` contient l'ancien contenu, `profils.json` est en version 2. Au lancement suivant, plus de message de migration | non testé | non testé |
| W79 | Créer, dupliquer, renommer, supprimer | Créer « Cours » (le nouveau profil devient actif, effets par défaut, tous actifs, sans raccourci) ; le dupliquer (copie « Cours (copie) » active, mêmes réglages et même activation, sans raccourci, focus sur « Renommer ») ; la renommer ; tenter un nom vide, un nom déjà pris (sans tenir compte de la casse) et un nom de plus de 40 caractères : refus affiché sous le champ, rien ne change. Le profil par défaut ne peut pas être supprimé (bouton grisé, raison écrite). Supprimer un profil demande une confirmation ; supprimer un autre profil que l'actif ne change ni les réglages ni l'activation en cours ; supprimer le profil actif ramène au profil par défaut. Les raccourcis des profils restants fonctionnent encore après chaque suppression | non testé | non testé |
| W80 | Raccourci d'un profil, saisi au clavier | Entrée sur le champ, taper Ctrl + Alt + Maj + K : enregistré, affiché, et le raccourci passe bien au profil. Pendant la saisie, appuyer sur le raccourci d'un autre profil ne change pas de profil (les raccourcis globaux sont suspendus) ; ils fonctionnent de nouveau après. Échap annule, Suppr efface, une lettre seule ou un chiffre est refusé avec une explication, Tab quitte la saisie sans rien changer | non testé | non testé |
| W81 | Raccourci déjà pris | Dans la page, donner à un profil la combinaison d'un autre profil, d'un effet ou du profil suivant : « indisponible : déjà utilisé par … » (avec le nom), l'ancien raccourci est conservé. Occuper Ctrl + Alt + Maj + K avec une autre application, puis la saisir : « indisponible : déjà utilisé par une autre application », ancien raccourci conservé et tous les autres raccourcis fonctionnent. **macOS : noter si la combinaison prise par une autre application est détectée** (Carbon peut ne rien signaler) | non testé | non testé |
| W82 | Choix de l'adaptateur | Automatique, économe, performant : le choix est enregistré tout de suite (`profils.json`), la page dit « s'appliquera au prochain lancement », rien ne change à l'écran. Au lancement suivant, la sonde cite le critère « réglage de l'adaptateur ». Avec `WGPU_POWER_PREF=high` défini au lancement de SOURISO, les trois choix sont grisés avec la mention de la variable | non testé | non testé |
| W83 | WebView2 absent (Windows) | Sur une machine ou une session sans WebView2 (ou en renommant temporairement la clé de registre concernée, avec prudence), « Réglages… » affiche une boîte native qui cite WebView2 et renvoie au README ; aucune fenêtre ne s'ouvre ; les effets, les raccourcis et le dernier profil continuent de fonctionner ; la sonde dit « fenêtre de réglages non ouverte ». Aucun téléchargement n'est tenté. La boîte est lisible au-dessus du voile du projecteur (noter si elle est assombrie) | non testé | sans objet |
| W84 | Exécutable des réglages absent | Lancer `souriso` seul, sans `souriso-reglages` à côté : « Réglages… » affiche une boîte qui l'explique, rien d'autre ne change | non testé | non testé |
| W85 | Lisibilité de la fenêtre sous le voile | Projecteur actif à opacité 100 % : la fenêtre de réglages reste entièrement lisible et utilisable (pas assombrie), le voile reste plein tout autour. Déplacer la fenêtre : pas de liseré sombre qui s'attarde (un bref retard est possible, le noter). La redimensionner, la réduire (le voile reprend la place), la restaurer. Sur deux écrans : la fenêtre à cheval sur les deux, puis sur le second seulement. Échelle 100 % et 150 % ou Retina | non testé | non testé |
| W86 | Limite 1 du voile (acceptée) | Pointeur sur la fenêtre, le cercle éclairé n'est pas visible et la page le dit ; en sortant de la fenêtre, le cercle réapparaît avec ses réglages. L'anneau, la traînée et l'ondulation se voient sur la fenêtre | non testé | non testé |
| W87 | Limite 2 du voile (acceptée) | Noter le retard du trou au déplacement rapide de la fenêtre (sans marge de 8 px, un liseré serait visible) | non testé | non testé |
| W88 | Mode clair et sombre | Basculer le thème du système (fenêtre ouverte, sans la relancer) : la page suit, sans zone restée de l'autre thème, texte lisible partout, focus visible, interrupteurs et pastilles distincts | non testé | non testé |
| W89 | Navigation au clavier | Sans souris : Tab traverse la barre (profil, Revenir), la section choisie, puis la page ; flèches haut et bas, Début et Fin changent de section ; curseurs aux flèches et Pg↑/Pg↓ ; interrupteur à Espace ; saisie de raccourci à Entrée ; renommage à Entrée et Échap ; « Aller au contenu » apparaît au premier Tab. Le focus est toujours visible et jamais perdu après une action (liste des profils comprise). Lecteur d'écran (Narrateur, VoiceOver) : chaque contrôle est nommé, les valeurs sont dites avec leur unité, les erreurs sont annoncées | non testé | non testé |
| W90 | Pas de défilement parasite | À 880 × 600 puis au minimum 720 × 520, aucune barre de défilement sur aucune section ; si la fenêtre est trop petite ou le texte agrandi, seule la zone centrale défile, jamais horizontalement | non testé | non testé |
| W91 | Prise en main (S17) | Un utilisateur qui n'a jamais vu l'application, sans documentation, active un effet et le règle en moins de 60 s (chronométrer : icône → Réglages… → interrupteur → un réglage visible). Au moins 3 personnes extérieures | non testé | non testé |
| W92 | Aucune ressource distante, aucun droit | Outil de surveillance du réseau (Moniteur de ressources, `lsof -i`) : aucune connexion ouverte par `souriso` ni par `souriso-reglages` (ni par ses processus web) pendant l'ouverture et l'usage de la fenêtre. Compte standard : aucune invite d'élévation, aucun téléchargement | non testé | non testé |
| W93 | Contraste vérifié à l'œil | Le test automatique garantit les couleurs ; vérifier qu'aucun texte n'est posé sur autre chose que ces fonds (relire chaque section dans les deux thèmes, avec les modes de contraste élevé du système) | non testé | non testé |

### Cases à cocher (feat/reglages)

- [ ] W68 : ouverture (Windows, macOS)
- [ ] W69 : fermeture, processus du moteur web disparus, mémoire (Windows, macOS)
- [ ] W70 : S4 fenêtre fermée (Windows, macOS)
- [ ] W71 : fermeture avec le processus principal (Windows, macOS)
- [ ] W72 : plantage de la fenêtre (Windows, macOS)
- [ ] W73 : aperçu en direct de chaque réglage (Windows, macOS)
- [ ] W74 : enregistrement et restauration (Windows, macOS)
- [ ] W75 : revenir aux valeurs du profil (Windows, macOS)
- [ ] W76 : activation enregistrée dans chaque profil (Windows, macOS)
- [ ] W77 : interrupteur de la fenêtre et raccourcis de session (Windows, macOS)
- [ ] W78 : migration d'un ancien fichier (Windows, macOS)
- [ ] W79 : créer, dupliquer, renommer, supprimer (Windows, macOS)
- [ ] W80 : raccourci saisi au clavier (Windows, macOS)
- [ ] W81 : raccourci déjà pris, détection notée (Windows, macOS)
- [ ] W82 : adaptateur (Windows, macOS)
- [ ] W83 : WebView2 absent (Windows)
- [ ] W84 : exécutable des réglages absent (Windows, macOS)
- [ ] W85 : lisibilité sous le voile (Windows, macOS)
- [ ] W86 : limite 1 du voile (Windows, macOS)
- [ ] W87 : limite 2 du voile (Windows, macOS)
- [ ] W88 : mode clair et sombre (Windows, macOS)
- [ ] W89 : navigation au clavier (Windows, macOS)
- [ ] W90 : pas de défilement parasite (Windows, macOS)
- [ ] W91 : prise en main, S17 (Windows, macOS)
- [ ] W92 : aucune ressource distante, aucun droit (Windows, macOS)
- [ ] W93 : contraste à l'œil (Windows, macOS)
