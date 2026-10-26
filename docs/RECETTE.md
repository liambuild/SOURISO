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
