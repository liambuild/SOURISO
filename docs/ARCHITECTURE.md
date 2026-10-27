# Architecture

Ce document tranche la base technique de SOURISO. Ce choix engage tout le reste : il est donc comparé avant d'être retenu. Il compare trois familles, puis une quatrième, mixte, ajoutée après la première analyse.

- **Statut** : décidé, sous réserve de la sonde comparative sur Windows (voir section 9).
- **Date** : 2026-10-26.
- **Niveau de preuve** : rien n'a encore été mesuré ni testé sur macOS ou Windows. Les chiffres ci-dessous sont des ordres de grandeur tirés de la documentation et de l'expérience courante des frameworks. Chaque affirmation à confirmer est marquée « à vérifier » et rattachée à un risque de la section 5.

## 1. Contraintes qui pilotent le choix

Elles viennent de `CLAUDE.md` et du README.

- macOS et Windows uniquement. Linux est hors périmètre pour le produit.
- Aucun compte, aucun réseau, aucune télémétrie. Cela exclut les mises à jour automatiques et tout téléchargement au premier lancement.
- Le produit sert à être vu en direct et en enregistrement. Une surcouche invisible dans les captures d'écran ne remplit pas sa mission.
- Développeur seul : tout ce qui est écrit en double, ou qui demande un entretien fréquent, coûte cher.
- Toute fonction qui dépend du système est testée sur le système concerné, ou marquée « non testé » dans `docs/RECETTE.md`.
- **Contrainte de terrain** : le poste de développement est sous Linux. Le plan est de livrer Windows et macOS. Une machine Windows est disponible dès maintenant ; le Mac existe mais n'est pas accessible pour l'instant (voir le risque R1).
- **Aucun droit administrateur** : l'installation et l'exécution sous Windows ne doivent jamais en demander (installeur par utilisateur, démarrage automatique par la clé `HKCU`, aucune élévation). Conséquence assumée : les fenêtres lancées en administrateur échappent à l'application (R8).
- **Exigence d'expérience** : l'interface et les effets doivent être élégants, propres, soignés et très esthétiques. C'est un critère de choix à part entière, pas une finition (voir 3.5 et S15 à S17).

## 2. Familles comparées

| Famille | Exemple retenu pour l'analyse | Principe |
|---|---|---|
| **A. Moteur web léger** | Tauri 2 (cœur Rust, WebView2 sous Windows, WKWebView sous macOS) | Le moteur web du système affiche les effets. Le code natif se limite au strict nécessaire. |
| **B. Navigateur embarqué** | Electron (Chromium et Node embarqués) | L'application embarque son propre navigateur. Le comportement est identique sur les deux systèmes. |
| **C. Natif par système** | Swift/AppKit sur macOS, C# ou C++/Win32 sur Windows, avec une couche partagée minimale (format de réglages, spécification des effets) | Chaque système a son application et son moteur de rendu. |

| **D. Mixte** | Rust avec rendu GPU direct (`wgpu`) pour la surcouche et les effets, Tauri 2 pour la seule fenêtre de réglages | Pas de moteur web en permanence. La page web n'existe que pendant le réglage. |

Note : les permissions système dépendent de l'API système appelée, pas de la famille. Une famille ne « réclame » pas une permission, c'est l'API de détection des clics choisie qui le fait.

## 3. Comparatif

Légende : `++` très bon, `+` bon, `~` acceptable ou incertain, `-` faible. Les cellules marquées « à vérifier » ne sont pas établies.

### 3.1 Fonctions système

| Critère | A. Tauri | B. Electron | C. Natif |
|---|---|---|---|
| **Surcouche plein écran transparente qui laisse passer les clics** | `+` Fenêtre transparente et `set_ignore_cursor_events`. Sous macOS, la transparence passe par une API privée (`macOSPrivateApi`), sans conséquence ici car la distribution se fait hors App Store. À vérifier. | `++` Fenêtre transparente sans cadre, `setIgnoreMouseEvents(true)`. Schéma très répandu et éprouvé. | `++` NSPanel non activant ; fenêtre « layered » transparente et topmost sous Windows. Référence du genre. |
| **Suivi global du curseur** | `+` Lecture directe par l'API système depuis Rust, sans permission. Idéalement par événements plutôt que par interrogation. | `~` `screen.getCursorScreenPoint()` par minuteur JavaScript : interrogation, gigue de minuterie. | `++` Événements système, sans permission. |
| **Détection globale des clics** | `+` Code Rust maintenu par le projet : crochet `WH_MOUSE_LL` sous Windows, moniteur global `NSEvent` ou `CGEventTap` sous macOS. Le crate `rdev` existe mais son entretien est irrégulier. | `~` Aucune API native : il faut un module natif (`uiohook-napi`), à recompiler à chaque version d'Electron. On paie le code natif sans en avoir le contrôle. | `++` Même API qu'en A, directement accessible. |
| **Lecture de sons** | `+` Lecture côté Rust (`rodio`) pour éviter les règles de lecture automatique du moteur web. À vérifier : latence clic vers son. | `+` Web Audio, avec l'option de lecture automatique à activer. | `++` AVAudioPlayer, XAudio2 ou équivalent. |

### 3.2 Performance, poids, entretien (ordres de grandeur, non mesurés)

| Critère | A. Tauri | B. Electron | C. Natif |
|---|---|---|---|
| **Processeur en usage actif** (% d'un cœur) | 1 à 4 % | 3 à 10 % | 0,5 à 2 % |
| **Processeur au repos** (curseur immobile, rendu arrêté) | < 0,5 % si le rendu est à la demande | 0,5 à 2 % | < 0,2 % |
| **Mémoire**, somme des processus, un écran | 80 à 200 Mo (WebView2 plus lourd que WKWebView) | 150 à 300 Mo | 20 à 60 Mo |
| **Démarrage** | 0,5 à 1,5 s | 1 à 3 s | < 0,5 s |
| **Poids de l'installeur** | 3 à 10 Mo, plus le runtime WebView2 (voir ci-dessous) | 80 à 130 Mo | 2 à 20 Mo |
| **Maintenabilité par un développeur seul** | `+` Effets écrits une fois (TypeScript, canvas). Un peu de Rust à apprendre. Ruptures possibles entre versions de Tauri 2. | `+` Un seul langage. Mais mises à jour de sécurité de Chromium fréquentes à suivre, et module natif à entretenir. | `-` Deux bases de code, deux langages, deux moteurs de rendu. Chaque effet est écrit, réglé et testé deux fois. |
| **Coût de développement relatif** (estimation) | 1 | 0,8 | 1,8 |

Le runtime WebView2 est préinstallé sur Windows 11 et sur Windows 10 à jour. Sur une machine qui n'en a pas, l'amorceur de Tauri le télécharge à l'installation, ce qui contredit « aucun réseau » au moment de l'installation. La parade, un installeur hors ligne, ajoute plus de 100 Mo. À vérifier (R11).

### 3.3 Permissions système

| Fonction | macOS | Windows |
|---|---|---|
| Surcouche transparente | Aucune permission. | Aucune. |
| Position du curseur | Aucune (`NSEvent.mouseLocation`). | Aucune (`GetCursorPos`). |
| Clics globaux | **À vérifier.** Le moniteur global `NSEvent` pour les événements souris ne demande en principe aucune permission, contrairement au clavier. Un `CGEventTap` demande en principe « Surveillance de l'entrée ». Le choix de l'API décide du nombre de permissions : 0 ou 1. | Aucune pour les fenêtres normales. **À vérifier** : les fenêtres lancées en administrateur (UIPI) peuvent ne pas être vues (R8). |
| Son | Aucune. | Aucune. |
| Enregistrement d'écran | Non requise pour SOURISO : l'application dessine, elle ne capture pas. | Non requise. |

Les trois familles ont les mêmes besoins. B est la seule à ne pas pouvoir choisir librement son API de clics.

### 3.4 Comportements d'affichage

| Critère | A. Tauri | B. Electron | C. Natif |
|---|---|---|---|
| **Apps en plein écran, macOS** (Spaces) | `~` Exige niveau de fenêtre élevé, comportement « tous les bureaux » et « auxiliaire du plein écran », absents de l'API Tauri : à poser via le code natif de l'adaptateur (`objc2`). Faisable, à vérifier (R2). | `+` `setVisibleOnAllWorkspaces(true, {visibleOnFullScreen: true})` et `setAlwaysOnTop('screen-saver')`. Documenté, avec des défauts connus selon les versions de macOS. | `++` Contrôle total. |
| **Apps en plein écran, Windows** | `+` Fenêtre topmost : visible au-dessus du plein écran sans bordure et des diaporamas. **Hors portée** : plein écran exclusif (anciens jeux DirectX), où aucune surcouche ne s'affiche. | idem | idem |
| **Plusieurs écrans** | `~` Une fenêtre, donc un moteur web, par écran. Mémoire multipliée, en partie partagée (R3). | `~` Même principe, même surcoût. | `++` Une fenêtre native légère par écran. |
| **Haute densité de pixels, échelles mixtes** | `+` Le moteur web gère le ratio par fenêtre. La conversion des coordonnées (macOS : points, origine en bas à gauche ; Windows : pixels, origine en haut à gauche) se fait dans l'adaptateur. Déplacement d'une fenêtre entre deux échelles : points délicats (R7). | idem | `++` Maîtrise totale, mais à coder deux fois. |
| **Visibilité dans les enregistrements** | `=` Une surcouche est une fenêtre normale composée par le système : visible dans une capture d'écran entière, invisible dans une capture de fenêtre ou de jeu. Ne jamais activer la « protection du contenu » (`content_protected`, `WDA_EXCLUDEFROMCAPTURE`), qui masquerait les effets. | `=` | `=` |

La visibilité en enregistrement ne départage pas les familles. Elle reste le risque produit n°1 : voir R6.

### 3.5 Qualité esthétique et expérience

Deux surfaces sont concernées : les **effets** (ce que voit le public) et la **fenêtre de réglages** (ce que voit l'utilisateur).

| Critère | A. Tauri | B. Electron | C. Natif |
|---|---|---|---|
| **Liberté de dessin des effets** (halo doux, flou, dégradés, courbes d'animation) | `++` Canvas 2D ou WebGL, rendu identique à celui qu'on prototype dans un navigateur. | `++` Idem. | `+` Core Animation / Direct2D : puissant, mais écrit et réglé deux fois, avec des rendus qui divergent d'un système à l'autre. |
| **Fenêtre de réglages** | `++` HTML/CSS : typographie, animations, mode sombre, aperçu en direct des effets, itération rapide. | `++` Idem. | `+` Contrôles natifs fidèles au système, mais deux interfaces à polir séparément. |
| **Cohérence entre macOS et Windows** | `~` Même interface, mais deux moteurs web (WebKit, Chromium) : rendu des polices, du flou et des ombres légèrement différent. À vérifier. | `++` Chromium partout, rendu identique. | `-` Deux identités visuelles à harmoniser à la main. |
| **Intégration au système** (barre de menus, zone de notification, mode sombre) | `+` Via l'API de la plateforme. | `+` | `++` |
| **Effort de polissage par un développeur seul** | `++` Un seul design à soigner. | `++` | `-` Deux. |

Pour une exigence esthétique forte tenue par une seule personne, le web est l'atout principal : une seule interface et un seul moteur d'effets à peaufiner. B est légèrement meilleur sur la cohérence visuelle entre systèmes, A reste préférable sur le poids et la mémoire.

### 3.6 Famille D, mixte : surcouche en Rust avec rendu GPU, réglages en Tauri

Variante ajoutée après la première analyse : les effets ne passent plus par un moteur web. Une fenêtre native transparente et passe-clics est dessinée directement par la carte graphique (Rust, `wgpu`). Seule la fenêtre de réglages est une page web (Tauri), créée à l'ouverture et détruite à la fermeture.

| Critère | A. Tauri seul | D. Mixte |
|---|---|---|
| **Surcouche passe-clics** | `+` Passe par le moteur web. | `++` Fenêtre native simple, sans moteur web entre le système et l'écran. |
| **Mémoire et processeur au repos** | `~` Un moteur web par écran, en permanence (R3). | `++` Aucun moteur web tant que les réglages sont fermés. |
| **Décalage effet / curseur** | `~` Un maillon de plus : le moteur web et sa composition (R5). | `+` Chaîne plus courte. À vérifier. |
| **Effets : liberté et rendu** | `++` Canvas, rendu un peu variable selon le moteur de chaque système. | `++` Shaders (halo, lueur, flou) au rendu identique sur les deux systèmes. |
| **Itération sur les effets** | `++` Rechargement immédiat dans le navigateur. | `~` Compilation Rust et shaders : plus lent. |
| **Fenêtre de réglages** | `++` | `++` Identique : c'est Tauri. |
| **Aperçu en direct des réglages** | `++` Dans la page. | `+` Les effets réels bougent autour du curseur pendant le réglage. Pas de second rendu à écrire. |
| **Complexité du projet** | `+` Une pile. | `~` Deux piles de rendu (GPU et web), mais jamais deux par système. |

## 4. Recommandation

**Famille D : surcouche et effets en Rust avec rendu GPU direct, fenêtre de réglages en Tauri 2, frontière système stricte.**

Pourquoi :

1. **La surcouche est la partie critique du produit, et c'est celle où un moteur web gêne le plus.** Sans lui, les risques sur la mémoire (R3) et sur la stabilité de la transparence (R9) disparaissent pour la surcouche, et la chaîne jusqu'à l'écran est plus courte (R5).
2. **Les effets ont un rendu identique sur Windows et macOS.** Les shaders produisent la même lueur et le même flou partout, là où deux moteurs web les rendent légèrement différemment.
3. **L'esthétique de l'interface est préservée.** La fenêtre de réglages reste une page web soignée, avec mode sombre et animations. Comme elle n'existe que lorsqu'elle est ouverte, le moteur web n'occupe ni mémoire ni processeur le reste du temps.
4. **La frontière système est conservée.** Le code qui dépend du système reste dans les adaptateurs, comme pour la famille A.

Contreparties assumées : les effets s'itèrent plus lentement (compilation, shaders) ; l'aperçu des réglages repose sur les effets réels autour du curseur ; il y a deux techniques de rendu à maintenir (GPU et web).

**Condition de départ : une sonde comparative sur Windows avant tout engagement.** La même surcouche (un anneau qui suit le curseur et laisse passer les clics) est réalisée en D et en A. D est confirmée si elle tient S1 à S5 sur Windows et si la transparence fonctionne sans droits administrateur. Sinon, on bascule sur A.

**Replis, par ordre de coût croissant :**

1. Si la surface GPU transparente est inutilisable sur un système : rendu logiciel de la surcouche (2D, sans carte graphique) sur ce système. Les effets sont simples, le surcoût de processeur est à mesurer.
2. Si D échoue sur la sonde : famille A (Tauri seul).
3. Si le plein écran ou la mémoire échoue sur **un** système dans la famille choisie : surcouche native pour ce système, derrière la même frontière.
4. Si le moteur web de réglages est inutilisable : Electron pour la seule fenêtre de réglages.

**Pourquoi pas les autres :** Electron pour le poids et la mémoire ; le natif intégral pour le double développement ; Qt pour le C++, la licence et la courbe d'apprentissage ; Flutter pour la fragilité des fenêtres transparentes et passe-clics.

### 4.1 Frontière système

Principe : **tout ce qui dépend d'un système vit dans un seul endroit, derrière un contrat unique, et le reste de l'application ne sait pas sur quel système elle tourne.**

Trois couches :

| Couche | Contenu | Connaît le système ? |
|---|---|---|
| **Cœur** | Réglages, état, modèle des effets (paramètres, durées), règles métier, événements normalisés. | Non. Testable sous Linux. |
| **Plateforme** (port + adaptateurs) | Contrat unique et trois implémentations : macOS, Windows, factice. | Oui, et seulement ici. |
| **Rendu** | Dessin des effets (Rust, GPU) à partir des événements normalisés. | Non. La création de la fenêtre passe par la couche plateforme. |
| **Interface** | Fenêtre de réglages (page web). | Non. Ne reçoit que des événements normalisés. |

Contrat de la couche plateforme :

| Capacité | Contenu du contrat |
|---|---|
| Écrans | Liste des écrans : identifiant, position et taille dans le bureau virtuel, facteur d'échelle. Notification d'ajout, de retrait, de changement. |
| Curseur | Flux de positions déjà normalisées : pixels logiques, origine en haut à gauche, avec l'identifiant de l'écran. |
| Clics | Flux d'événements : bouton, appui ou relâchement, position normalisée. |
| Surfaces | Création d'une surcouche par écran : transparente, passe-clics, au-dessus de tout, visible dans les apps en plein écran, jamais exclue de la capture. |
| Permissions | État modélisé (accordée, refusée, inconnue) et demande, par capacité. Un refus est un état, pas une exception. |
| Cartes graphiques | Nom de la carte qui porte chaque écran, si le système le donne (sinon aucun avis). Sert au choix de l'adaptateur graphique (section 4.2). |
| Son | Lecture d'un son à faible latence. |
| Cycle de vie | Lancement au démarrage de session, icône de barre de menus ou de zone de notification. |

Règles de la frontière, vérifiables mécaniquement :

1. Aucun `cfg(target_os)`, aucune API système, aucun `unsafe` en dehors du dossier de la couche plateforme. Contrôle par recherche textuelle, à ajouter aux « Commandes de vérification » de `CLAUDE.md`.
2. L'adaptateur factice rejoue des événements scriptés. Il permet de développer et tester le cœur et l'interface sous Linux, ce qui répond à la contrainte de terrain.
3. Chaque fonction d'un adaptateur a une ligne dans `docs/RECETTE.md`, par système, avec « testé » ou « non testé ».
4. Les conversions de coordonnées et d'échelle sont faites dans l'adaptateur, et nulle part ailleurs.
5. Un adaptateur peut être remplacé par un adaptateur natif complet (surcouche comprise) sans toucher au cœur ni à l'interface.

### 4.2 Choix de l'adaptateur graphique

Sur une machine à deux cartes, rendre sur une carte qui ne porte pas les écrans oblige le système à recopier chaque image d'une carte à l'autre : du temps noyau et un coût par image plus élevés (relevé du propriétaire sur Windows : Quadro P620 dédiée, Intel UHD intégrée qui porte les écrans ; mesures à refaire, voir `docs/RECETTE.md`, W31).

Ordre de décision, du plus prioritaire au moins prioritaire. La sonde écrit le critère qui a décidé dans la ligne « Sonde : adaptateur » :

1. `WGPU_ADAPTER_NAME` (fragment du nom, sans casse, parmi les adaptateurs qui peuvent présenter). Critère : « variable WGPU_ADAPTER_NAME ».
2. `WGPU_POWER_PREF` (`low`, `high`, `none`), transmise à wgpu. Critère : « variable WGPU_POWER_PREF ».
3. **La carte qui porte l'écran principal**, donnée par la couche plateforme (`plateforme::cartes_ecrans`) et rapprochée d'un adaptateur wgpu par son nom. Elle n'est retenue que si elle peut présenter sur la surface et si ce n'est pas un pilote logiciel. Critère : « carte qui porte les écrans ».
4. Sinon, le choix de wgpu (`PowerPreference::None`, premier adaptateur compatible avec la surface). Si wgpu tombe sur un pilote logiciel (type processeur, par exemple le « Microsoft Basic Render Driver ») alors qu'un autre adaptateur peut présenter, le premier de ceux-là est retenu. Critère : « choix de wgpu » ou « choix de wgpu, pilote logiciel écarté ».

Le pilote logiciel n'est jamais retenu aux étapes 3 et 4 tant qu'une autre carte peut présenter. Un nom donné par `WGPU_ADAPTER_NAME` reste un choix explicite et n'est pas filtré.

**Mécanisme par système.**

- *Windows.* `EnumDisplayDevicesW` liste les sorties actives du bureau avec le nom de la carte qui les porte ; `EnumDisplaySettingsW` donne la position de chaque sortie, rapprochée de l'origine de l'écran (pixels physiques). Deux appels de `user32` via `windows-sys`, sans COM ni DXGI. Le nom de la carte est rapproché de celui de wgpu par égalité ou inclusion, sans casse. Les deux noms viennent de la même description du pilote ; un désaccord de libellé conduit à l'étape 4, jamais à un échec.
- *macOS.* La couche plateforme ne donne pas d'avis : le comportement est celui de l'étape 4.
- *Factice (Linux).* Aucun avis.

**Limites.**

- *Poste fixe.* L'heuristique suit la carte qui porte les écrans, quelle qu'elle soit : sur un poste fixe dont la carte dédiée porte les écrans, c'est la dédiée qui est retenue (et non l'intégrée, même si elle est active et peut présenter). Ce cas n'a pas été mesuré (« non testé », W32).
- *Écrans répartis entre plusieurs cartes.* Une seule carte rend pour tous les écrans (un seul périphérique, section « Rendu »). On retient celle de l'écran principal ; les écrans portés par l'autre carte paient la recopie. Un adaptateur par écran demanderait un périphérique par carte, hors de cette modification. La sonde donne la carte de chaque écran, ce qui rend le cas visible.
- *Rapprochement par nom.* Deux cartes de même nom (deux cartes identiques) ne se distinguent pas : la première énumérée est retenue.
- *Écran dont la carte est inconnue* (sortie virtuelle, bureau à distance, système muet) : si c'est l'écran principal, on retombe sur l'étape 4.
- *Observée, non expliquée.* `WGPU_POWER_PREF=low` aurait retenu la Quadro chez le propriétaire (W34).

**À prévoir.** Un réglage de l'adaptateur (automatique, économe, performant) rejoindra la fenêtre de réglages du prompt 5. « Automatique » est le comportement ci-dessus ; les deux autres reprennent `WGPU_POWER_PREF`.

## 5. Risques

Classés par gravité (impact multiplié par probabilité). Le test indiqué est le moins coûteux qui permette de conclure. Les « sondes » sont de petits programmes jetables, à écrire sur une branche à part après l'arbitrage. Aucun n'est écrit à ce stade.

| # | Risque | Gravité | Test le moins coûteux |
|---|---|---|---|
| **R1** | **Mac indisponible pour l'instant.** Le développement est sous Linux ; la surcouche, les permissions et le plein écran ne se vérifient que sur le vrai système. Tant que le Mac n'est pas accessible, tout ce qui touche macOS (R2, R4, échelles mixtes) reste « non testé » et les décisions qui en dépendent sont provisoires. | Élevé | Fixer la date à laquelle le Mac devient accessible et réserver ce créneau aux sondes R2, R3, R4 et R6 sur macOS. Travailler en attendant sur le cœur, l'interface et Windows. Les runners GitHub Actions compilent mais ne montrent pas l'écran ; macOS en VM est exclu en pratique. Test : une demi-heure. |
| **R2** | **Surcouche absente au-dessus des apps en plein écran sous macOS** (Spaces, Keynote, navigateur en plein écran). | Critique | Sonde Tauri minimale : fenêtre transparente avec les drapeaux de niveau et d'espace, puis ouvrir Keynote et Safari en plein écran. Une demi-journée sur un Mac. |
| **R3** | **Mémoire et processeur de WebView2, multipliés par le nombre d'écrans.** Le seuil de 150 Mo est menacé. Ne concerne plus que le repli A et la fenêtre de réglages ouverte. | Élevé | Même sonde avec 1, puis 2 fenêtres. Mesurer la somme des processus (octets privés sous Windows, `footprint` sous macOS) au repos et pendant un mouvement continu de 60 s. Une demi-journée par système. |
| **R4** | **Détection des clics macOS demandant une permission** que l'API choisie n'exigerait pas. Dégrade le premier lancement. | Élevé | Mini-programme comparant le moniteur global `NSEvent` et `CGEventTap` sur un compte macOS neuf : observer si une invite apparaît et si les clics arrivent. Deux heures. |
| **R5** | **Décalage visible entre l'effet et le curseur.** Le curseur matériel est dessiné au dernier instant, la surcouche passe par la composition : un décalage de 1 à 3 images est probable. | Élevé | Anneau serré autour du curseur dans la sonde, filmé à 240 images/s au téléphone, comptage d'images d'écart. Une heure une fois la sonde prête. |
| **R6** | **Effets absents des enregistrements** selon l'outil : OBS (capture d'écran vs de fenêtre vs de jeu), Barre de jeu de Windows (capture d'une seule app), Cmd+Maj+5, ScreenCaptureKit. | Élevé | Enregistrer 10 s avec chaque outil usuel, sur chaque système, et relire. Une demi-journée. Livre aussi le contenu de la documentation utilisateur sur les modes de capture compatibles. |
| **R7** | **Coordonnées fausses avec haute densité ou échelles mixtes** : anneau décalé sur le deuxième écran ou en franchissant la jonction. | Moyen à élevé | Windows avec deux écrans à 100 % et 150 %, Mac Retina avec écran externe : promener le curseur d'un écran à l'autre, vérifier le centrage à ± 1 px logique. Une heure par configuration. |
| **R8** | **Windows : fenêtres administrateur et jeux en plein écran exclusif invisibles ou muets** (UIPI). | Moyen | Lancer le Gestionnaire des tâches en administrateur, cliquer, observer l'ondulation. Puis un jeu en bordure et en exclusif. Une heure. |
| **R9** | **Transparence et passe-clics du moteur web instables** (régressions Tauri, WebView2 ou macOS). | Moyen | Couvert par la sonde de R2 et R3, sur Windows 10, Windows 11 et deux versions de macOS. |
| **R10** | **Son : latence ou blocage par la règle de lecture automatique** si le son passe par le moteur web. | Moyen | Lecture côté Rust dans la sonde ; enregistrer écran et audio ensemble et mesurer l'écart entre clic et son. Une heure. |
| **R11** | **Distribution : signature et notarisation macOS, SmartScreen sous Windows, runtime WebView2 absent.** Touche à la signature et à la publication : voir `CLAUDE.md`, constat à rapporter avant toute modification. | Moyen | Installeur non signé lancé sur une machine Windows 10 sans WebView2 et sur un Mac vierge (Gatekeeper). Observer les écrans d'alerte et les téléchargements réseau. Une demi-journée. |
| **R13** | **Levé sous Windows par la composition DX12 (voir section 9) ; reste à confirmer sous macOS.** Famille D : surface GPU transparente et cohabitation avec Tauri.** La transparence d'une surface `wgpu` n'est pas garantie sur tous les systèmes et pilotes ; la boucle d'événements de Tauri et la fenêtre de rendu doivent cohabiter dans un seul processus. | Élevé | Sonde Windows comparative (condition de la section 9) : fenêtre transparente passe-clics dessinée par `wgpu`, puis fermeture et réouverture de la fenêtre Tauri. Si la cohabitation échoue, deux processus (surcouche, réglages) échangeant par fichier de réglages. Une demi-journée. |
| **R12** | **Entretien en solo** : Rust à apprendre, ruptures de versions de Tauri. | Faible à moyen | Pas de test. Épingler les versions, et relire les notes de version avant chaque mise à jour. |

Ordre conseillé des sondes : R13 et R3 ensemble sur Windows (la sonde comparative D contre A), puis R5 et R6 sur Windows. Les sondes macOS (R2, R4, R7) suivent dès que le Mac est accessible.

**Règle d'arrêt :** si R13 échoue, on applique les replis de la section 4 dans l'ordre. Si R2 échoue sur macOS, surcouche native pour ce système avant de poursuivre les effets.

## 6. Seuils de départ : lesquels sont réalistes

| Seuil demandé | Verdict | Raison |
|---|---|---|
| Effet **sans décalage visible** à 60 images par seconde | **Irréaliste tel quel** | « Visible » ne se mesure pas, et un décalage de 1 à 3 images derrière le curseur matériel est quasi inévitable. Il se voit sur un anneau serré, pas sur un projecteur large et doux. Reformuler en latence mesurée. De plus, 60 images/s est inférieur au taux de nombreux écrans (120 Hz), où l'effet aura l'air en retard. |
| Processeur **< 3 %** en usage actif | **Réaliste en natif, plausible en Tauri, incertain en Electron** | **Définition obligatoire** : le Gestionnaire des tâches de Windows divise par le nombre de cœurs, le Moniteur d'activité de macOS non. Le même seuil serait 8 fois plus tendu sur un Mac que sur un PC à 8 cœurs. Je propose « % d'un cœur logique, somme des processus de l'application », la lecture la plus stricte. |
| Processeur **< 1 %** au repos | **Réaliste si le rendu est à la demande** | Atteignable si le rendu s'arrête curseur immobile et si les événements ne sont pas interrogés en boucle. Irréaliste si le projecteur est redessiné en continu. À définir : « repos » = curseur immobile 60 s, effets actifs. |
| Mémoire **< 150 Mo** | **À risque en Tauri, irréaliste en Electron** | Mesure ambiguë (RSS, octets privés, empreinte) : à définir. WebView2 pèse plus que WKWebView, et chaque écran supplémentaire ajoute un moteur. À tenir pour un seul écran ; prévoir un plafond par écran en plus (R3). |
| Démarrage **< 2 s** | **Réaliste** en Tauri et en natif, limite en Electron | À définir : du lancement du processus au premier effet affiché, médiane de 10 lancements, hors démarrage à froid de la machine. |

En résumé, deux seuils sont à reformuler (décalage et mémoire), un à préciser (processeur : unité de mesure). Aucun n'est absurde pour la famille recommandée.

## 7. Critères de succès de la v1

Tous sont mesurables. Les machines de référence (un Mac à puce Apple et un PC Windows 11 d'entrée de gamme à carte graphique intégrée) restent à désigner. Chaque critère se mesure sur chaque système et se consigne dans `docs/RECETTE.md`.

| # | Critère | Seuil | Mesure |
|---|---|---|---|
| S1 | Cadence | ≥ 58 images/s en moyenne, 1 % pires ≥ 50, 60 s de mouvement continu, 1 écran | Compteur d'images de la sonde. |
| S2 | Décalage effet / curseur | ≤ 2 images à 60 Hz (33 ms) au 95e centile | Vidéo à 240 images/s. |
| S3 | Processeur, usage actif | < 3 % d'un cœur logique, moyenne sur 60 s de mouvement | Moniteur d'activité (macOS), compteurs de performance (Windows), somme des processus. |
| S4 | Processeur, repos | < 1 % d'un cœur logique, curseur immobile 60 s, effets actifs | Idem. |
| S5 | Mémoire | < 150 Mo à un écran, +40 Mo au plus par écran supplémentaire (valeur à confirmer par R3) | Somme des processus : octets privés (Windows), `footprint` (macOS). |
| S6 | Démarrage | < 2 s, médiane de 10 lancements | Du lancement au premier effet affiché. |
| S7 | Installeur | ≤ 25 Mo par système, sans téléchargement à l'installation | Taille du fichier publié ; installation hors ligne sur machine vierge. |
| S8 | Permissions | macOS : ≤ 1 invite, 0 si possible. Windows : 0 invite d'élévation | Premier lancement sur compte neuf. |
| S9 | Plein écran | Effets visibles sur diaporama (Keynote, PowerPoint), navigateur en plein écran, jeu en fenêtre sans bordure. Plein écran exclusif : limite documentée | Recette manuelle, par cas. |
| S10 | Plusieurs écrans, échelles mixtes | Anneau centré à ± 1 px logique sur 2 écrans d'échelles différentes, y compris au franchissement de la jonction | Recette manuelle sur écrans à 100 % et 150 % (Windows), Retina et externe (macOS). |
| S11 | Enregistrement | Effets visibles dans la capture d'écran entière de chaque outil de la liste fixée en recette ; comportement documenté pour les captures de fenêtre ou de jeu | Enregistrement de 10 s par outil. |
| S12 | Aucun réseau | 0 connexion sortante sur 10 minutes d'usage | Surveillance des connexions (`lsof`, Moniteur de ressources). |
| S13 | Frontière | 0 occurrence d'API ou de condition système hors de la couche plateforme | Recherche textuelle, ajoutée aux commandes de vérification. |
| S14 | Honnêteté de la recette | Toute fonction dépendant du système est « testée » ou « non testé » dans `docs/RECETTE.md` | Relecture de la recette. |
| S15 | Finition visuelle des effets | Aucun effet saccadé, crénelé ou à bords durs à 100 % et 200 % de mise à l'échelle ; apparition et disparition adoucies (aucune coupure sèche) | Relecture de captures vidéo, effet par effet, sur chaque système testé. |
| S16 | Fenêtre de réglages | Mode clair et sombre suivant le système ; aperçu en direct de chaque réglage ; aucune barre de défilement parasite ; navigation complète au clavier ; contraste du texte ≥ 4,5:1 | Revue point par point en recette, sur chaque système testé. |
| S17 | Prise en main | Un utilisateur qui n'a jamais vu l'application active un effet et le règle en moins de 60 s, sans documentation | Test avec au moins 3 personnes extérieures, chronométré. |
| S19 | Mémoire une fois les réglages fermés | Aucun processus de moteur web en cours ; mémoire revenue sous le seuil S5 | Liste des processus après fermeture de la fenêtre de réglages. |
| S18 | Droits | Installation et exécution sans élévation ni invite UAC sous Windows | Installation sur un compte standard, vérification de l'absence d'invite. |

## 8. Questions ouvertes

1. **Postes de test** : une machine Windows est disponible ; le Mac existe mais sa date de disponibilité est à fixer. Quelles seront les machines de référence (R1) ?
2. **Versions minimales** : macOS 12 ou 13 et plus ? Windows 10 22H2 et plus, ou Windows 11 seul ? Le runtime WebView2 et l'API de transparence en dépendent.
3. **Taux d'images** : 60 images/s suffisent-elles, ou faut-il suivre le taux de l'écran (120 Hz) ?
4. **Mise à jour** : sans réseau, les mises à jour se font à la main depuis les releases. Est-ce acceptable ?

## 9. Décision

**Date** : 2026-10-26.

**Famille retenue : D.** La surcouche et les effets sont écrits en Rust avec rendu GPU direct. La fenêtre de réglages est une page web Tauri 2, créée à l'ouverture et détruite à la fermeture. Tout ce qui dépend du système est isolé dans la couche plateforme (section 4.1).

**Motifs** : performance et stabilité de la surcouche sans moteur web permanent ; rendu des effets identique sur les deux systèmes ; interface de réglages web très soignée ; développeur seul, un seul moteur d'effets à peaufiner.

**Contraintes confirmées** :

- Windows et macOS sont tous deux livrés.
- Aucun droit administrateur sous Windows.
- Exigence d'élégance de l'interface et des effets.
- Le Mac n'est pas accessible pour l'instant. Jusqu'à ce qu'il le soit, tout ce qui touche macOS reste « non testé » dans `docs/RECETTE.md`, et les décisions qui en dépendent sont provisoires.

**Conditions et repli** : la décision est conditionnée à la sonde comparative sur Windows (section 4). En cas d'échec, on bascule sur la famille A. La liste complète des replis est en section 4.

**R13 levé par la composition sur DX12 (Windows).** La sonde du socle n'avait trouvé aucune surface transparente : Vulkan n'annonce que l'alpha opaque et DX12, dans son mode par défaut (`DxgiFromHwnd`), aussi. Le mode `DxgiFromVisual` de wgpu 30 (chaîne d'échange de composition attachée à un visuel DirectComposition) annonce `PreMultiplied` et `PostMultiplied`. Essayé par le propriétaire sur Windows (carte NVIDIA Quadro P620, backend DX12) : un mode alpha transparent est listé et le bureau reste visible sous le repère. Le repli 1 (rendu logiciel) n'est donc pas retenu : il aurait imposé d'écrire chaque effet deux fois et, en repeinture complète à 1080p, coûtait environ 12 à 30 % d'un cœur contre un seuil S3 de 3 % (estimations, non mesurées).

- **Mise en œuvre** : la couche plateforme expose `options_instance`, qui fixe le backend (DX12) et le mode de présentation (`DxgiFromVisual`) sous Windows. Le rendu ne contient aucune condition système ; il y ajoute seulement les variables `WGPU_*` de l'environnement. Aucun droit administrateur.
- **Ce qui reste à confirmer sous Windows** (voir `docs/RECETTE.md`) : cadence et décalage (S1, S2), processeur (S3, S4), absence de fond noir ou de cadre, passe-clics, échelles, plusieurs écrans. Le coût processeur de la composition (≈ 0,6 à 1,8 % d'un cœur à 60 images/s) est une estimation.
- **macOS** : inchangé, toujours « non testé ».

**Projecteur, plusieurs écrans et interface des effets (branche `feat/projecteur`).**

- **Une surcouche par écran**, un seul périphérique graphique partagé (mémoire : pas un périphérique par écran). Le curseur n'est cherché que dans l'écran qui le contient ; les autres écrans portent un voile plein.
- **Redessin seulement sur changement** : une image par écran dont la position du curseur sur lui, l'état d'activation ou la taille a changé. Écran sans curseur et curseur immobile : aucune image. Lorsque le curseur passe d'un écran à l'autre, chacun des deux est redessiné une fois. Le voile ne multiplie donc pas le coût processeur par le nombre d'écrans.
- **Risque pour S3 et S4 : non écarté, non mesuré.** Côté processeur, la conception ne devrait pas dépasser le socle (même cadence d'interrogation, mêmes images). Reste le coût de la composition (DWM) d'une surface plein écran qui n'est plus presque transparente : charge GPU et mémoire par écran, et effet possible sur une vidéo plein écran dessous. Rien de cela n'est mesuré ; voir W7, W8 et W15 dans `docs/RECETTE.md`. Si W7 dépasse 3 % ou 1 %, le repli est à décider avec le propriétaire avant de toucher à la cadence ou à la présentation.
- **Interface des effets** : `rendu::effet::Effet` (nom, préparation des données GPU, dessin dans la passe) et `creer_effets`, seul endroit où déclarer un effet. L'activation par nom vit dans `coeur::activation`. Ajouter un effet ne touche ni la boucle, ni la plateforme.
- **Configuration unique** : `coeur::config::Config`, valeurs par défaut sans interface de réglage.
- **Frontière** : le raccourci global (`RegisterHotKey` / Carbon via la crate `global-hotkey`, sans permission ni droit administrateur) et la lecture du temps processeur du processus (`GetProcessTimes` / `getrusage`) vivent dans `src/plateforme`. Défaut : Ctrl + Alt + Maj + P. Un refus du système n'arrête pas l'application, il est écrit dans la sonde.
- **Limite connue** : les écrans sont détectés au lancement ; brancher ou débrancher un écran, ou changer sa résolution, demande de relancer l'application.

**Seuils** : les reformulations de la section 6 (latence en images, unité de mesure du processeur, mesure de la mémoire, définition du démarrage) sont proposées mais **pas encore validées**. Elles sont à valider avant la v1.

**Questions encore ouvertes** (section 8) : versions minimales de Windows et de macOS ; cadence cible (60 images/s ou taux de l'écran) ; mises à jour manuelles depuis les releases ; date de disponibilité du Mac.

**Étapes suivantes** : aucune n'est lancée par ce document. La première sera la sonde Windows, sur une branche à part, après la fusion de celle-ci. Les commandes de vérification de `CLAUDE.md` (dont le contrôle de la frontière, S13) seront à compléter à ce moment.
