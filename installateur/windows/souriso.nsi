; Installateur Windows de SOURISO (NSIS), par utilisateur : aucun droit administrateur, aucune
; élévation. Tout se passe dans le profil de l'utilisateur (HKCU, %LOCALAPPDATA%, menu Démarrer
; de l'utilisateur). Paramètres passés par makensis (/D) : VERSION, BIN (dossier des deux .exe),
; ICONE (icône du dépôt), SORTIE (fichier produit). Chemins absolus.
;
; La désinstallation retire les fichiers, le raccourci et l'entrée de désinstallation. Elle ne
; touche jamais à %APPDATA%\SOURISO (profils.json) et ne touche pas au démarrage automatique,
; que l'application n'implémente pas encore (docs/ARCHITECTURE.md, section 9).

Unicode true
RequestExecutionLevel user

!include "MUI2.nsh"

Name "SOURISO"
OutFile "${SORTIE}"
InstallDir "$LOCALAPPDATA\Programs\SOURISO"
InstallDirRegKey HKCU "Software\SOURISO" "Dossier"
SetCompressor /SOLID lzma

!define CLE_DESINSTALLATION "Software\Microsoft\Windows\CurrentVersion\Uninstall\SOURISO"

!define MUI_ICON "${ICONE}"
!define MUI_UNICON "${ICONE}"
!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "French"

Section "SOURISO"
  SetShellVarContext current
  SetOutPath "$INSTDIR"
  ; Les deux exécutables côte à côte : souriso cherche souriso-reglages dans son dossier.
  File "${BIN}\souriso.exe"
  File "${BIN}\souriso-reglages.exe"
  File "/oname=souriso.ico" "${ICONE}"

  CreateShortCut "$SMPROGRAMS\SOURISO.lnk" "$INSTDIR\souriso.exe" "" "$INSTDIR\souriso.ico" 0

  WriteUninstaller "$INSTDIR\desinstaller.exe"
  WriteRegStr HKCU "Software\SOURISO" "Dossier" "$INSTDIR"
  WriteRegStr HKCU "${CLE_DESINSTALLATION}" "DisplayName" "SOURISO"
  WriteRegStr HKCU "${CLE_DESINSTALLATION}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${CLE_DESINSTALLATION}" "DisplayIcon" "$INSTDIR\souriso.ico"
  WriteRegStr HKCU "${CLE_DESINSTALLATION}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${CLE_DESINSTALLATION}" "UninstallString" '"$INSTDIR\desinstaller.exe"'
  WriteRegDWORD HKCU "${CLE_DESINSTALLATION}" "NoModify" 1
  WriteRegDWORD HKCU "${CLE_DESINSTALLATION}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  SetShellVarContext current
  Delete "$SMPROGRAMS\SOURISO.lnk"
  Delete "$INSTDIR\souriso.exe"
  Delete "$INSTDIR\souriso-reglages.exe"
  Delete "$INSTDIR\souriso.ico"
  Delete "$INSTDIR\desinstaller.exe"
  ; Sans /r : le dossier n'est retiré que s'il est vide, jamais avec des fichiers qui ne sont
  ; pas les nôtres (le dossier d'installation peut avoir été choisi par l'utilisateur).
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${CLE_DESINSTALLATION}"
  DeleteRegKey HKCU "Software\SOURISO"
  ; %APPDATA%\SOURISO (profils.json) n'est volontairement pas touché.
SectionEnd
