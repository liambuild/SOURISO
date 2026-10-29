#!/usr/bin/env bash
# Affiche la version de SOURISO après avoir vérifié que les trois endroits qui la portent
# concordent : Cargo.toml, reglages/Cargo.toml et reglages/tauri.conf.json. Avec un tag en
# argument (v0.1.0), vérifie aussi qu'il vaut « v » suivi de cette version. Sert à la CI et à la
# release : un écart fait échouer le workflow.
set -eu
cd "$(dirname "$0")/.."

lire_cargo() { sed -n 's/^version = "\([^"]*\)".*$/\1/p' "$1" | head -n 1; }

principale=$(lire_cargo Cargo.toml)
reglages=$(lire_cargo reglages/Cargo.toml)
tauri=$(sed -n 's/^ *"version": "\([^"]*\)".*$/\1/p' reglages/tauri.conf.json | head -n 1)

if ! printf '%s' "$principale" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "Version illisible ou hors du format x.y.z dans Cargo.toml : « $principale »" >&2
  exit 1
fi
if [ "$principale" != "$reglages" ] || [ "$principale" != "$tauri" ]; then
  echo "Versions discordantes : Cargo.toml=$principale, reglages/Cargo.toml=$reglages, reglages/tauri.conf.json=$tauri" >&2
  exit 1
fi
tag=${1:-}
if [ -n "$tag" ] && [ "$tag" != "v$principale" ]; then
  echo "Le tag « $tag » ne correspond pas à la version du dépôt (v$principale)." >&2
  exit 1
fi
echo "$principale"
