# Installation de SOURISO

Les installateurs se téléchargent dans les [releases](https://github.com/liambuild/SOURISO/releases) du dépôt : l'installateur Windows (`SOURISO-x.y.z-windows-x64-setup.exe`) et l'image disque macOS (`SOURISO-x.y.z-macos-aarch64.dmg`). Le fichier `SHA256SUMS.txt` donne leurs empreintes pour les vérifier.

Ces installateurs **ne sont pas signés** pour l'instant. Windows et macOS affichent donc un avertissement au premier lancement. Il ne signale pas un défaut de SOURISO : le système ne connaît simplement pas l'éditeur. Compte rendu de ce qu'il faut faire pour passer.

## Windows

- Windows 10 ou 11, 64 bits. Aucun droit administrateur n'est demandé : SOURISO s'installe dans votre profil (`%LOCALAPPDATA%\Programs\SOURISO`), avec un raccourci « SOURISO » dans le menu Démarrer.
- **Avertissement SmartScreen** (« Windows a protégé votre ordinateur ») à l'ouverture de l'installateur :
  1. cliquez sur **Informations complémentaires** ;
  2. cliquez sur **Exécuter quand même**.
- **WebView2.** La fenêtre de réglages utilise le composant Microsoft Edge WebView2. Il est déjà présent sur Windows 11 et sur Windows 10 à jour. L'installateur ne le télécharge pas (SOURISO n'utilise aucun réseau). S'il manque, SOURISO l'indique par une boîte de message native au moment d'ouvrir les réglages et continue de fonctionner avec le dernier profil utilisé ; installez le « Runtime WebView2 » de Microsoft (installation possible sans droit administrateur), puis rouvrez les réglages.
- **Désinstallation** : Paramètres > Applications > SOURISO > Désinstaller. Vos profils (`%APPDATA%\SOURISO\profils.json`) sont conservés ; supprimez ce dossier à la main si vous n'en voulez plus. Quittez SOURISO (icône de la zone de notification > Quitter) avant de désinstaller.

## macOS

- **Apple Silicon (M1 et suivants) uniquement.** Les Mac Intel ne sont pas pris en charge pour l'instant.
- Ouvrez l'image disque et glissez **SOURISO** dans le dossier **Applications**. SOURISO n'a pas d'icône dans le Dock : elle vit dans la barre de menus.
- **Blocage Gatekeeper** (« SOURISO ne peut pas être ouvert car son développeur ne peut pas être vérifié » ou « est endommagé ») au premier lancement :
  1. ouvrez **Réglages Système > Confidentialité et sécurité** ;
  2. en bas de la page, à la ligne qui mentionne SOURISO, cliquez sur **Ouvrir quand même**, puis confirmez avec votre mot de passe ;
  3. relancez SOURISO.

  Sur les macOS antérieurs à Sequoia (15), un clic droit sur l'application puis **Ouvrir** suffit. Si le message dit que l'application est « endommagée », retirez l'attribut de quarantaine dans le Terminal : `xattr -d com.apple.quarantine /Applications/SOURISO.app`.
- **Désinstallation** : glissez SOURISO de **Applications** à la corbeille. Vos profils (`~/Library/Application Support/SOURISO/profils.json`) sont conservés.
