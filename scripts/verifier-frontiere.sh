#!/usr/bin/env bash
# Contrôle S13 : aucune API ni condition système en dehors de src/plateforme. Le crate de la
# fenêtre de réglages (reglages/) n'a aucune exception : il ne contient aucune condition ni API
# système, et sa page n'appelle aucune ressource distante.
set -eu

motif='target_os|target_family|cfg\((not\()?(windows|unix)|unsafe|winit::platform|windows_sys|core_graphics|tray_icon|objc|std::os::'

trouves=$(grep -rnE "$motif" src --include='*.rs' --exclude-dir=plateforme || true)
reglages=$(grep -rnE "$motif" reglages/src reglages/build.rs --include='*.rs' || true)
trouves="$trouves$reglages"
# Une dépendance propre à un système dans le crate des réglages serait aussi une condition système.
if grep -nE '^\[target\.' reglages/Cargo.toml; then
  trouves="$trouves
reglages/Cargo.toml : dépendance conditionnelle à un système"
fi
distant=$(grep -rnE 'https?://|@import|url\(' reglages/ui || true)
if [ -n "$distant" ]; then
  echo "Ressource distante dans la page des réglages :" >&2
  echo "$distant" >&2
  exit 1
fi

if [ -n "$trouves" ]; then
  echo "Frontière système violée hors de src/plateforme :" >&2
  echo "$trouves" >&2
  exit 1
fi
echo "Frontière système respectée."
