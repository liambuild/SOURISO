# SOURISO

Application de bureau qui met en évidence le curseur de la souris, pour les créateurs de contenu et les formateurs.

## Présentation

SOURISO rend le curseur facile à suivre à l'écran, en direct comme en enregistrement. Elle propose :

- un projecteur autour du curseur ;
- un anneau ;
- une traînée ;
- une ondulation au clic ;
- un son au clic ;
- des profils de réglages nommés, chacun avec son raccourci et l'activation de chaque effet (par défaut, « Présentation » et « Enregistrement » au premier lancement), enregistrés localement dans le dossier de données de l'utilisateur ;
- une fenêtre de réglages, ouverte depuis l'icône de la zone de notification (Windows) ou de la barre de menus (macOS), qui règle chaque effet en direct et gère les profils.

## Périmètre

- Systèmes pris en charge : macOS et Windows. Linux est hors périmètre.
- Cibles : créateurs de contenu et formateurs.
- Aucun compte, aucun réseau, aucune télémétrie.
- Installeurs publiés dans les releases de ce dépôt.

## Fenêtre de réglages

L'icône de SOURISO a une entrée « Réglages… ». Elle ouvre une fenêtre où chaque changement s'applique tout de suite aux effets autour du curseur et s'enregistre dans le profil choisi : activer ou couper un effet, régler ses paramètres, créer, dupliquer, renommer ou supprimer des profils, saisir le raccourci de chacun, choisir la carte graphique (au lancement suivant). « Revenir aux valeurs du profil » défait les essais faits depuis l'ouverture. Tant que le projecteur est actif, la fenêtre n'est pas assombrie ; pour voir le cercle éclairé, sortez le pointeur de la fenêtre.

La fenêtre est un second programme, `souriso-reglages`, qui doit rester **dans le même dossier** que `souriso` (les deux sont livrés ensemble). Il n'est lancé qu'à l'ouverture de la fenêtre et s'arrête à sa fermeture : fenêtre fermée, SOURISO ne garde aucun moteur web.

**Windows : WebView2.** La fenêtre utilise le composant Microsoft Edge WebView2 du système, qu'elle n'embarque ni ne télécharge. Windows 11 et les installations récentes de Windows 10 l'ont déjà. S'il manque, SOURISO l'indique dans une boîte de message et continue de fonctionner avec le dernier profil utilisé ; installez le « Runtime WebView2 » proposé par Microsoft (installation sans droit administrateur possible, pour l'utilisateur), puis rouvrez les réglages. Aucune autre étape. Les profils peuvent aussi se modifier à la main dans `profils.json` (`%APPDATA%\SOURISO`).

**macOS.** Le moteur web (WKWebView) fait partie du système : rien à installer.

## Organisation du projet

Ce dépôt est la source de vérité du projet : code, documentation, décisions, recette et installeurs.

- `CLAUDE.md` : méthode de travail et règles du projet.
- `docs/INSTALLATION.md` : installation, avertissements SmartScreen et Gatekeeper, désinstallation.
- `docs/ARCHITECTURE.md` : choix de la base technique, risques et critères de succès.
- `docs/RECETTE.md` : recette du produit.
- `src/` : le programme principal (surcouche et effets) ; `reglages/` : la fenêtre de réglages (Tauri 2, page statique dans `reglages/ui/`).
