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

## Fonctions dépendant du système

Toute fonction qui dépend du système est soit testée sur le système concerné, soit marquée « non testé » dans `docs/RECETTE.md`.

## Commandes de vérification

## Points de contrôle au démarrage
