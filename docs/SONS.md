# Sons

Les sons de clic sont embarqués dans l'exécutable (`include_bytes!` dans `src/coeur/son.rs`). Aucun n'est téléchargé, aucun ne vient d'une source externe.

## Origine et licence

| Fichier | Origine | Licence |
|---|---|---|
| `assets/sons/clic.wav` | Synthétisé par `examples/generer_sons.rs` : deux sinusoïdes (1 800 et 3 600 Hz) à enveloppe exponentielle de 6 ms. | Produit par le code du dépôt, sans échantillon tiers. |
| `assets/sons/bulle.wav` | Synthétisé par `examples/generer_sons.rs` : une sinusoïde qui monte de 500 à 1 100 Hz en 80 ms. | Idem. |
| `assets/sons/cloche.wav` | Synthétisé par `examples/generer_sons.rs` : 880 Hz et un partiel à 2 425 Hz, extinction en 120 ms. | Idem. |

Aucun son n'est un enregistrement ni une copie : la licence ne dépend d'aucun tiers. Le dépôt ne contient pas encore de fichier de licence : **le propriétaire doit en fixer une**, et ces fichiers suivront celle du dépôt.

## Format

WAV PCM 16 bits, mono, 44 100 Hz, en-tête de 44 octets, crête à 80 % de la pleine échelle. Le volume de `coeur::config::SonClic` multiplie les échantillons au lancement (`coeur::son::avec_volume`), car la lecture à la demande des systèmes n'a pas de réglage de volume.

## Régénérer

```
cargo run --example generer_sons
```

Le calcul est déterministe : relancer le générateur redonne les mêmes octets. Si l'un des sons change, commiter le générateur et les `.wav` ensemble.

## Lecture

Par la couche plateforme (`plateforme::jouer_son`), à la demande, sans flux audio permanent :

- Windows : `PlaySoundW` depuis la mémoire, en asynchrone.
- macOS : `NSSound` initialisé depuis la mémoire.
- Factice (Linux) : ne joue rien.

Un son qui arrive pendant un autre coupe le précédent. Latence entre le clic et le son : inconnue, à mesurer (`docs/RECETTE.md`).
