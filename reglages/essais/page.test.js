/* Essai de la page des réglages (reglages/ui) dans jsdom, avec un faux processus principal.
   Hors build et hors CI : le dépôt n'a pas de dépendance npm. Pour le lancer, dans un dossier à part
   (pas dans le dépôt) : 
added 63 packages in 4s

13 packages are looking for funding
  run `npm fund` for details, puis, depuis la racine du dépôt :
     SOURISO_ETAT_EXEMPLE=/chemin/etat.json cargo test ecrire_etat_exemple
     NODE_PATH=/chemin/node_modules SOURISO_ETAT_EXEMPLE=/chemin/etat.json node reglages/essais/page.test.js
   L'état d'exemple est écrit par le vrai protocole (src/coeur/reglages.rs). */
const { JSDOM } = require("jsdom");
const fs = require("fs");
const path = require("path");
const UI = path.join(__dirname, "..", "ui");
const etat0 = JSON.parse(fs.readFileSync(process.env.SOURISO_ETAT_EXEMPLE || path.join(__dirname, "etat.json"), "utf8"));
let echecs = 0;
const ok = (c, m) => { if (!c) { echecs++; console.log("ÉCHEC :", m); } else console.log("ok     :", m); };
const attendre = (ms = 30) => new Promise((r) => setTimeout(r, ms));

(async () => {
  const envoyes = [];
  let ecoute = null, affichages = 0;
  const mock = {
    core: { invoke: (cmd, args) => { if (cmd === "envoyer") envoyes.push(JSON.parse(args.ligne)); if (cmd === "afficher") affichages++; return Promise.resolve(); } },
    event: { listen: (nom, cb) => { ecoute = cb; return Promise.resolve(() => {}); } },
  };
  const dom = await JSDOM.fromFile(path.join(UI, "index.html"), {
    runScripts: "dangerously", resources: "usable", pretendToBeVisual: true,
    beforeParse(w) { w.__TAURI__ = mock; },
  });
  const { window } = dom; const doc = window.document;
  await new Promise((r) => window.addEventListener("load", r));
  await attendre();
  ok(envoyes.some((m) => m.type === "pret"), "la page annonce qu'elle est prête");
  const livrer = (o) => ecoute({ payload: JSON.stringify(o) });
  const dernier = (t) => [...envoyes].reverse().find((m) => m.type === t);
  const $ = (s) => doc.querySelector(s), $$ = (s) => [...doc.querySelectorAll(s)];

  livrer(etat0); await attendre();
  ok(affichages === 1, "la fenêtre est affichée après le premier état");
  ok($$("[role=tab]").length === 7, "sept sections : 5 effets, profils, carte");
  ok($("h1").textContent === "Projecteur", "ouverture sur le projecteur");
  ok($$("#page input[type=range]").length === 3, "projecteur : 3 curseurs");
  ok($$("#page .reglage").length === 4, "projecteur : 4 réglages");
  ok($("#profil-actif").options.length === 3, "sélecteur de profil : 3 profils");
  ok($("#revenir").disabled, "« Revenir » est désactivé sans modification");
  ok($("#statut").textContent.startsWith("Enregistré"), "statut : enregistré automatiquement");
  ok($("[role=switch]").getAttribute("aria-checked") === "true", "interrupteur activé");
  ok(!doc.body.innerHTML.match(/https?:\/\//), "aucune adresse distante dans la page");

  // Curseur : un seul message par image, valeur dans l'unité affichée
  const rayon = $("#r-projecteur-rayon");
  rayon.value = "200"; rayon.dispatchEvent(new window.Event("input", { bubbles: true }));
  rayon.value = "210"; rayon.dispatchEvent(new window.Event("input", { bubbles: true }));
  await attendre(60);
  const regl = envoyes.filter((m) => m.type === "regler" && m.chemin === "projecteur.rayon");
  ok(regl.length === 1 && regl[0].valeur === 210, "deux mouvements du curseur → un seul message (210)");
  ok($("[data-cle='r-projecteur-rayon-n']").value === "210", "le champ numérique suit le curseur");
  ok(rayon.getAttribute("aria-valuetext") === "210 pixels", "valeur lue : « 210 pixels »");
  ok(!$("#revenir").disabled, "« Revenir » s'active après un changement");
  ok($("#statut").textContent.startsWith("Modifié"), "statut : modifié depuis l'ouverture");

  // Champ numérique hors bornes
  const champ = $("[data-cle='r-projecteur-rayon-n']");
  champ.value = "9999"; champ.dispatchEvent(new window.Event("change", { bubbles: true }));
  await attendre(60);
  ok(dernier("regler").valeur === 600 && champ.value === "600", "valeur hors bornes ramenée au maximum de la table (600)");

  // Couleur : pastille
  const pastille = $$("#page .pastille")[1];
  pastille.click(); await attendre(60);
  ok(dernier("regler").chemin === "projecteur.couleur" && dernier("regler").valeur === "#ff4033", "pastille rouge → #ff4033");
  ok(pastille.getAttribute("aria-pressed") === "true", "la pastille choisie est marquée");

  // Interrupteur
  $("[role=switch]").focus(); $("[role=switch]").click(); await attendre(60);
  ok(dernier("regler").chemin === "actif.projecteur" && dernier("regler").valeur === false, "interrupteur → actif.projecteur = false");
  ok($("[role=switch]").getAttribute("aria-checked") === "false", "interrupteur coupé");
  ok(!!$(".note.coupe"), "note « effet coupé » affichée");
  ok($$("[role=tab]")[0].textContent.includes("Coupé"), "la section indique « Coupé »");
  ok(doc.activeElement === $("[role=switch]"), "le focus reste sur l'interrupteur");

  // Message d'activation (raccourci pressé pendant que la fenêtre est ouverte)
  livrer({ type: "activation", effet: "projecteur", actif: true }); await attendre();
  ok($("[role=switch]").getAttribute("aria-checked") === "true", "un raccourci d'effet met l'interrupteur à jour");

  // Revenir
  $("#revenir").click(); await attendre();
  ok(dernier("revenir"), "« Revenir » envoie la demande");

  // Navigation clavier
  const tab0 = $$("[role=tab]")[0]; tab0.focus();
  tab0.dispatchEvent(new window.KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true })); await attendre();
  ok($("h1").textContent === "Anneau" && doc.activeElement.dataset.id === "anneau", "flèche bas : section Anneau, focus conservé sur l'onglet");
  $("#onglets").dispatchEvent(new window.KeyboardEvent("keydown", { key: "End", bubbles: true })); await attendre();
  ok($("h1").textContent === "Carte graphique", "Fin : dernière section");
  ok($$("[role=tab][tabindex='0']").length === 1, "un seul onglet dans l'ordre de tabulation");

  // Son
  $$("[role=tab]")[4].click(); await attendre();
  ok($("h1").textContent === "Son" && !!$("#page select"), "page Son : liste de sons");
  $("#page select").value = "cloche"; $("#page select").dispatchEvent(new window.Event("change", { bubbles: true })); await attendre(60);
  ok(dernier("regler").valeur === "cloche", "choix du son envoyé");

  // Carte graphique
  $$("[role=tab]")[6].click(); await attendre();
  ok($$("#page input[type=radio]").length === 3 && !$("#page input[type=radio]").disabled, "3 choix de carte, actifs");
  ok(!$("#page").textContent.includes("null") && !$("#page").textContent.includes("undefined"), "aucun « null » affiché sans variable d'environnement");
  ok($$("#page .note").length === 1, "sans variable : une seule note, celle du prochain lancement");
  $$("#page input[type=radio]")[1].click(); await attendre();
  ok(dernier("adaptateur").valeur === "econome", "adaptateur économe envoyé");
  ok($("#statut").textContent.includes("prochain lancement"), "mention « au prochain lancement »");
  livrer({ ...etat0, adaptateur_impose: "WGPU_POWER_PREF" }); await attendre();
  ok($$("#page input[type=radio]").every((r) => r.disabled) && $("#page .note").textContent.includes("WGPU_POWER_PREF"), "grisé si une variable d'environnement l'impose");
  ok($("#page .note").textContent === "Ce choix est imposé par la variable d'environnement WGPU_POWER_PREF : le réglage ci-dessous est sans effet tant qu'elle est définie.", "ligne attendue quand une variable impose la carte");
  ok(!$("#page").textContent.includes("null"), "aucun « null » affiché quand une variable impose la carte");

  // Profils
  $$("[role=tab]")[5].click(); await attendre();
  ok($$(".liste-profils > li").length === 3, "3 profils listés");
  ok($$(".liste-profils > li")[0].querySelector("button.danger").disabled, "le profil par défaut n'est pas supprimable");
  ok($$(".liste-profils > li")[0].textContent.includes("ne se supprime pas"), "la raison est écrite");
  $$(".liste-profils > li")[1].querySelector("input[type=radio]").click(); await attendre();
  ok(dernier("profil").action === "activer" && dernier("profil").indice === 1, "choisir le profil actif");
  // dupliquer
  $$(".liste-profils > li")[1].querySelectorAll("button.bouton")[1].click(); await attendre();
  ok(dernier("profil").action === "dupliquer" && dernier("profil").indice === 1, "dupliquer");
  // renommer
  $$(".liste-profils > li")[2].querySelectorAll("button.bouton")[0].click(); await attendre();
  const nom = $("input[data-cle='nom-2']");
  ok(!!nom && doc.activeElement === nom, "renommer : champ en place, focus dessus");
  nom.value = "Cours du soir"; nom.dispatchEvent(new window.Event("input", { bubbles: true }));
  nom.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Enter", bubbles: true })); await attendre();
  ok(dernier("profil").action === "renommer" && dernier("profil").nom === "Cours du soir", "renommer envoie le nom");
  livrer(etat0); await attendre();
  ok(doc.activeElement.dataset.cle === "renommer-2", "après le renommage, le focus revient sur « Renommer »");
  // supprimer avec confirmation
  $$(".liste-profils > li")[2].querySelector("button.danger").click(); await attendre();
  ok(!!$("[data-cle='confirmer-2']") && doc.activeElement.dataset.cle === "confirmer-2", "suppression : confirmation en ligne, focus sur « Supprimer »");
  ok(!envoyes.some((m) => m.action === "supprimer"), "rien n'est supprimé avant la confirmation");
  $("[data-cle='confirmer-2']").click(); await attendre();
  ok(dernier("profil").action === "supprimer" && dernier("profil").indice === 2, "suppression confirmée");
  livrer(etat0); await attendre();
  ok(doc.activeElement.dataset.cle === "profil-0", "après la suppression, le focus va au profil actif");
  // créer
  $("#nouveau-nom").value = "Atelier"; $("#nouveau-nom").dispatchEvent(new window.Event("input", { bubbles: true }));
  $("form.nouveau").dispatchEvent(new window.Event("submit", { bubbles: true, cancelable: true })); await attendre();
  ok(dernier("profil").action === "creer" && dernier("profil").nom === "Atelier", "créer un profil");
  // faute
  livrer({ ...etat0, faute: { cible: "nouveau", message: "un profil s'appelle déjà « Atelier »" } }); await attendre();
  ok($(".erreur") && $(".erreur").textContent.includes("déjà"), "refus du cœur affiché sous le champ");
  ok($(".erreur").getAttribute("role") === "alert", "l'erreur est annoncée");

  // Raccourcis : saisie
  livrer(etat0); await attendre();
  const brac = $("[data-cle='raccourci-1']");
  brac.focus(); brac.click(); await attendre();
  ok(dernier("capture") && dernier("capture").debut === true, "début de saisie : les raccourcis globaux sont suspendus");
  ok(brac.textContent.startsWith("Tapez"), "invite de saisie");
  const touche = (o) => brac.dispatchEvent(new window.KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...o }));
  touche({ key: "Shift", code: "ShiftLeft", shiftKey: true });
  touche({ key: "k", code: "KeyK" });
  ok(!envoyes.some((m) => m.type === "raccourci"), "une lettre seule est refusée");
  ok($("#statut").textContent.includes("Ctrl ou Alt"), "…avec l'explication");
  touche({ key: "7", code: "Digit7", ctrlKey: true });
  ok(!envoyes.some((m) => m.type === "raccourci"), "un chiffre est refusé");
  touche({ key: "K", code: "KeyK", ctrlKey: true, altKey: true, shiftKey: true });
  const r = dernier("raccourci");
  ok(r && r.cible === 1 && r.raccourci.touche === "K" && r.raccourci.ctrl && r.raccourci.alt && r.raccourci.maj, "Ctrl+Alt+Maj+K envoyé pour le profil 1");
  livrer({ ...etat0, faute: { cible: "profil:1", message: "indisponible : déjà utilisé par le profil « Enregistrement »" } }); await attendre();
  ok($(".erreur") && $(".erreur").textContent.includes("indisponible : déjà utilisé"), "« indisponible : déjà utilisé » affiché");
  // Échap
  $("[data-cle='raccourci-2']").click(); await attendre();
  const b2 = $("[data-cle='raccourci-2']");
  b2.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true })); await attendre();
  ok(dernier("capture").debut === false && b2.textContent === "Ctrl + Alt + Maj + R", "Échap annule et rétablit le texte");
  // Suppr
  $("[data-cle='raccourci-2']").click(); await attendre();
  $("[data-cle='raccourci-2']").dispatchEvent(new window.KeyboardEvent("keydown", { key: "Delete", code: "Delete", bubbles: true, cancelable: true })); await attendre();
  ok(dernier("raccourci").raccourci === null && dernier("raccourci").cible === 2, "Suppr efface le raccourci");
  // profil suivant
  livrer(etat0); await attendre();
  $("[data-cle='raccourci-suivant']").click(); await attendre();
  $("[data-cle='raccourci-suivant']").dispatchEvent(new window.KeyboardEvent("keydown", { key: "j", code: "KeyJ", altKey: true, bubbles: true, cancelable: true })); await attendre();
  ok(dernier("raccourci").cible === "suivant" && dernier("raccourci").raccourci.touche === "J", "raccourci du profil suivant");

  // Accessibilité de base
  ok($$("input:not([type=hidden]):not([type=radio])").every((i) => i.id && doc.querySelector(`label[for='${i.id}']`) || i.getAttribute("aria-label")), "chaque champ a un nom accessible");
  ok($$("button").every((b) => b.textContent.trim() || b.getAttribute("aria-label")), "chaque bouton a un nom accessible");
  ok($("html").lang === "fr", "langue de la page : fr");

  // Version de protocole incompatible
  livrer({ ...etat0, version: 99 }); await attendre();
  ok($("#statut").classList.contains("alerte"), "version incompatible : alerte");

  console.log(echecs ? `\n${echecs} échec(s)` : "\ntout est bon");
  process.exit(echecs ? 1 : 0);
})();
