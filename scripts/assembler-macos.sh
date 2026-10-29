#!/usr/bin/env bash
# Assemble SOURISO.app puis l'image disque (macOS, Apple Silicon).
# Usage : scripts/assembler-macos.sh <version> <dossier des exécutables> <image .dmg à produire>
# Les deux exécutables vont côte à côte dans Contents/MacOS : souriso cherche souriso-reglages
# dans son propre dossier. Utilise l'icône du dépôt (reglages/icons/icon.png), sans en créer.
# L'application n'est pas signée avec une identité : seule la signature « ad hoc » (sans
# certificat ni secret) est posée, car macOS sur Apple Silicon refuse un paquet sans aucune
# signature. La vraie signature viendra dans le workflow (.github/workflows/installateurs.yml).
set -eu
cd "$(dirname "$0")/.."

if [ "$#" -ne 3 ]; then
  echo "Usage : $0 <version> <dossier des exécutables> <image .dmg>" >&2
  exit 2
fi
version=$1
bin=$2
dmg=$3

for exe in souriso souriso-reglages; do
  [ -f "$bin/$exe" ] || { echo "Exécutable absent : $bin/$exe" >&2; exit 1; }
  # Seule l'architecture Apple Silicon est prise en charge.
  if ! lipo -archs "$bin/$exe" | grep -qw arm64; then
    echo "$exe n'est pas compilé pour arm64." >&2
    exit 1
  fi
done

travail=$(mktemp -d)
trap 'rm -rf "$travail"' EXIT
app="$travail/image/SOURISO.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

cp "$bin/souriso" "$bin/souriso-reglages" "$app/Contents/MacOS/"
chmod 755 "$app/Contents/MacOS/souriso" "$app/Contents/MacOS/souriso-reglages"

# Icône : jeu d'icônes tiré de l'image 256 x 256 du dépôt, sans agrandissement.
jeu="$travail/SOURISO.iconset"
mkdir "$jeu"
source=reglages/icons/icon.png
for t in 16 32 128 256; do sips -z "$t" "$t" "$source" --out "$jeu/icon_${t}x${t}.png" >/dev/null; done
cp "$jeu/icon_32x32.png" "$jeu/icon_16x16@2x.png"
cp "$jeu/icon_256x256.png" "$jeu/icon_128x128@2x.png"
iconutil -c icns "$jeu" -o "$app/Contents/Resources/SOURISO.icns"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>SOURISO</string>
  <key>CFBundleDisplayName</key><string>SOURISO</string>
  <key>CFBundleIdentifier</key><string>fr.souriso.app</string>
  <key>CFBundleExecutable</key><string>souriso</string>
  <key>CFBundleIconFile</key><string>SOURISO</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleVersion</key><string>${version}</string>
  <key>CFBundleShortVersionString</key><string>${version}</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# Signature ad hoc (sans identité), de l'intérieur vers l'extérieur.
codesign --force --sign - "$app/Contents/MacOS/souriso-reglages"
codesign --force --sign - "$app/Contents/MacOS/souriso"
codesign --force --sign - "$app"

ln -s /Applications "$travail/image/Applications"
hdiutil create -volname "SOURISO" -srcfolder "$travail/image" -ov -format UDZO "$dmg"
echo "Image produite : $dmg"
