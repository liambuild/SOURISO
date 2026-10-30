<div align="center">

# SOURISO

**Un curseur facile à suivre à l'écran, en direct comme en enregistrement.**

[![CI](https://github.com/liambuild/SOURISO/actions/workflows/ci.yml/badge.svg)](https://github.com/liambuild/SOURISO/actions/workflows/ci.yml)
[![Dernière release](https://img.shields.io/github/v/release/liambuild/SOURISO)](https://github.com/liambuild/SOURISO/releases)

</div>

SOURISO met en évidence le curseur de la souris pour les créateurs de contenu et les formateurs. L'application s'installe sous Windows et macOS. Elle n'utilise ni compte, ni réseau, ni télémétrie.

## Les effets

Cinq effets, activables séparément :

- **Projecteur** : un voile assombrit l'écran et laisse un cercle éclairé, aux bords adoucis, autour du curseur.
- **Anneau** : un cercle suit le curseur.
- **Traînée** : le déplacement du curseur laisse une trace qui s'efface.
- **Ondulation au clic** : une onde part du curseur à chaque clic, un anneau pour le clic gauche, deux pour le droit.
- **Son au clic** : un bref son accompagne le clic. Trois sons sont proposés (clic, bulle, cloche), générés par le dépôt, sans échantillon tiers.

Les effets se dessinent sur tous les écrans. Les clics passent à travers la surcouche : l'application qui est dessous les reçoit normalement.

## Les profils

Un profil regroupe les réglages de tous les effets, l'activation de chacun et un raccourci. Trois profils existent au premier lancement :

| Profil | Raccourci |
|---|---|
| Par défaut | Ctrl + Alt + Maj + D |
| Présentation | Ctrl + Alt + Maj + E |
| Enregistrement | Ctrl + Alt + Maj + R |

Ctrl + Alt + Maj + N passe au profil suivant. Le dernier profil utilisé est restauré au lancement, avec son activation des effets.

Cinq autres raccourcis coupent ou rétablissent un effet, le temps de la session et sans modifier le profil : Ctrl + Alt + Maj + P (projecteur), A (anneau), T (traînée), O (ondulation) et S (son).

Les profils sont enregistrés dans `profils.json`, dans le dossier de données de l'utilisateur. Un fichier absent ou corrompu ramène aux valeurs par défaut, sans blocage.

## La fenêtre de réglages

Elle s'ouvre par l'entrée « Réglages… » de l'icône de SOURISO (zone de notification sous Windows, barre de menus sous macOS). Chaque changement s'applique aussitôt aux vrais effets et s'enregistre dans le profil choisi. On peut y :

- activer ou couper chaque effet et régler ses paramètres ;
- créer, dupliquer, renommer et supprimer des profils ;
- saisir le raccourci de chaque profil ;
- choisir la carte graphique (pris en compte au lancement suivant).

« Revenir aux valeurs du profil » défait les essais faits depuis l'ouverture. La fenêtre est un second programme, `souriso-reglages`, lancé à l'ouverture et arrêté à la fermeture.

![Page Projecteur de la fenêtre de réglages : interrupteur, rayon, adoucissement, opacité et couleur du voile](docs/images/img1.png)

Chaque effet a sa page, avec son interrupteur et le raccourci qui lui correspond :

| | |
|---|---|
| ![Page Anneau : rayon, épaisseur, opacité, couleur](docs/images/img2.png) | ![Page Traînée : longueur, durée du fondu, taille des points, opacité, couleur](docs/images/img3.png) |
| ![Page Ondulation : rayon final, durée, épaisseur, opacité, couleurs des clics gauche et droit](docs/images/img4.png) | ![Page Son : volume, choix du son, bouton Écouter](docs/images/img5.png) |

## Installation

Les installateurs sont publiés sur la [page des releases](https://github.com/liambuild/SOURISO/releases). Ils ne sont pas signés : le système affiche un avertissement au premier lancement.

**Windows 10 ou 11, 64 bits.** L'installateur (`SOURISO-x.y.z-windows-x64-setup.exe`) s'installe dans votre profil, sans droits administrateur. Windows SmartScreen affiche « Windows a protégé votre ordinateur » : cliquez sur **Informations complémentaires**, puis **Exécuter quand même**. La fenêtre de réglages utilise le composant Microsoft Edge WebView2, déjà présent sur Windows 11 et sur Windows 10 à jour. S'il manque, SOURISO le signale et continue de fonctionner avec le dernier profil utilisé.

**macOS, Apple Silicon uniquement (M1 et suivants).** Ouvrez l'image disque (`SOURISO-x.y.z-macos-aarch64.dmg`) et glissez SOURISO dans Applications. Gatekeeper bloque le premier lancement : ouvrez **Réglages Système > Confidentialité et sécurité**, puis cliquez sur **Ouvrir quand même**. Les Mac Intel ne sont pas pris en charge.

Le détail, les empreintes de vérification et la désinstallation sont dans [docs/INSTALLATION.md](docs/INSTALLATION.md).

## Légèreté

SOURISO interroge le curseur toutes les 8 ms pendant l'activité, puis toutes les 16 ms après une seconde sans mouvement. Un écran sans curseur ni animation ne reçoit aucune image.

Les seules mesures consignées dans le dépôt concernent le processeur, relevées par le propriétaire sur Windows : trois écrans, projecteur seul, curseur en mouvement continu pendant 60 s.

| Rendu porté par | Part d'un cœur logique |
|---|---|
| Quadro P620 | 6 à 11 % |
| Intel UHD, qui porte les écrans | 4,0 à 5,3 % |

L'objectif est de rester sous 3 % : il n'est pas atteint à la dernière mesure. Le choix de la carte graphique a changé depuis, et la nouvelle mesure n'est pas faite. Aucune mesure n'existe sur macOS.

## Limites connues

- Presque toute la recette reste « non testée » sur Windows et sur macOS : voir [docs/RECETTE.md](docs/RECETTE.md).
- Mémoire, temps de démarrage, cadence et décalage entre l'effet et le curseur ne sont pas mesurés.
- La latence entre le clic et le son est inconnue.
- Les effets en plein écran (diaporama, navigateur, jeu sans bordure) et dans les outils d'enregistrement sont à vérifier.
- Les installateurs ne sont pas signés.
- Le démarrage automatique avec le système n'existe pas.
- Linux n'est pas pris en charge.

## Pour les curieux

Les choix techniques, les risques et les critères de succès sont décrits dans [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
