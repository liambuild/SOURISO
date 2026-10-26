#!/usr/bin/env bash
# Contrôle S13 : aucune API ni condition système en dehors de src/plateforme.
set -eu

motif='target_os|target_family|cfg\((not\()?(windows|unix)|unsafe|winit::platform|windows_sys|core_graphics|tray_icon|objc|std::os::'

trouves=$(grep -rnE "$motif" src --include='*.rs' --exclude-dir=plateforme || true)

if [ -n "$trouves" ]; then
  echo "Frontière système violée hors de src/plateforme :" >&2
  echo "$trouves" >&2
  exit 1
fi
echo "Frontière système respectée."
