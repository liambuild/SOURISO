/* Réglages de SOURISO. Pas de framework, pas de dépendance, pas de réseau : la page affiche l'état
   que lui envoie le processus principal (une ligne de JSON par message, voir
   src/coeur/reglages.rs) et lui renvoie chaque changement, qui s'applique aussitôt aux effets
   réels. Les libellés, les bornes et les unités viennent de ce processus : rien n'est copié ici. */
"use strict";

(() => {
  const T = window.__TAURI__;
  const EVENEMENT = "souriso-message";
  const VERSION_PROTOCOLE = 1;
  const DELAI_SECOURS_MS = 2500;

  const PALETTE = [
    ["#ffcc1a", "Jaune"], ["#ff4033", "Rouge"], ["#2ecc8f", "Vert"], ["#4da6ff", "Bleu"],
    ["#c46bff", "Violet"], ["#ffffff", "Blanc"], ["#000000", "Noir"],
  ];
  const MOTS_UNITE = { px: "pixels", "%": "pour cent", s: "secondes" };
  const ICONES = {
    revenir: '<path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>',
    plus: '<path d="M12 5v14M5 12h14"/>',
    copie: '<rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>',
    crayon: '<path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/>',
    corbeille: '<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>',
    alerte: '<circle cx="12" cy="12" r="10"/><path d="M12 8v5"/><path d="M12 16h.01"/>',
    ecouter: '<path d="M11 5 6 9H2v6h4l5 4Z"/><path d="M15.5 8.5a5 5 0 0 1 0 7"/>',
  };

  /* --- État --------------------------------------------------------------------------------- */

  const etat = {
    effets: [], parametres: [], profils: [], actif: 0, live: {}, base: null,
    suivant: null, adaptateur: "automatique", impose: null, faute: null,
  };
  const ui = {
    page: null, edition: null, texteEdition: "", confirmation: null, saisie: null, nouveau: "",
    // Où poser le focus quand la réponse du processus principal aura remplacé la page.
    focusApres: null,
  };
  let affichee = false;
  let delaiMessage = null;

  const $ = (s) => document.querySelector(s);
  const profilActif = () => etat.profils[etat.actif];
  const parametre = (chemin) => etat.parametres.find((p) => p.chemin === chemin);
  const valeur = (chemin) => profilActif().valeurs[chemin];

  function egal(a, b) {
    return typeof a === "number" && typeof b === "number" ? Math.abs(a - b) < 1e-6 : a === b;
  }

  /** Vrai si les réglages ou l'activation du profil actif diffèrent de ceux de l'ouverture. */
  function modifie() {
    if (!etat.base || !profilActif()) return false;
    const p = profilActif();
    return Object.keys(etat.base.valeurs).some((k) => !egal(p.valeurs[k], etat.base.valeurs[k]))
      || Object.keys(etat.base.actifs).some((k) => p.actifs[k] !== etat.base.actifs[k]);
  }

  /* --- Construction du DOM ------------------------------------------------------------------ */

  function h(balise, props = {}, ...enfants) {
    const el = document.createElement(balise);
    for (const [cle, val] of Object.entries(props)) {
      if (val === false || val == null) continue;
      if (cle.startsWith("on")) el.addEventListener(cle.slice(2), val);
      else if (cle === "value") el.value = val;
      else if (cle === "checked") el.checked = val;
      else if (cle === "disabled") el.disabled = true;
      else el.setAttribute(cle, val === true ? "" : val);
    }
    el.append(...enfants.flat().filter((e) => e != null && e !== false));
    return el;
  }

  function icone(nom) {
    const s = h("span", { class: "icone", "aria-hidden": "true" });
    s.innerHTML = `<svg viewBox="0 0 24 24" focusable="false">${ICONES[nom]}</svg>`;
    return s;
  }

  function bouton(texte, nomIcone, props = {}) {
    return h("button", { type: "button", class: "bouton", ...props }, nomIcone ? icone(nomIcone) : null, texte);
  }

  /* --- Échanges avec le processus principal ------------------------------------------------- */

  const attente = new Map();
  let planifie = false;

  function envoyerBrut(message) {
    if (!T) return;
    T.core.invoke("envoyer", { ligne: JSON.stringify(message) }).catch(() => {});
  }

  /** Envoie les réglages en attente, au plus un par réglage et par image. */
  function vider() {
    planifie = false;
    for (const [chemin, v] of attente) envoyerBrut({ type: "regler", chemin, valeur: v });
    attente.clear();
  }

  function regler(chemin, v) {
    attente.set(chemin, v);
    if (!planifie) {
      planifie = true;
      requestAnimationFrame(vider);
    }
    majStatut();
  }

  /** Les autres messages gardent l'ordre : les réglages en attente partent avant. */
  function envoyer(message) {
    vider();
    envoyerBrut(message);
  }

  function recevoir(m) {
    switch (m.type) {
      case "etat": appliquerEtat(m); break;
      case "activation":
        etat.live[m.effet] = m.actif;
        rendreTout();
        break;
      case "erreur": annoncer(m.message, true); break;
      case "montrer": afficher(); break;
      default: break;
    }
  }

  function afficher() {
    if (T) T.core.invoke("afficher").catch(() => {});
  }

  function appliquerEtat(m) {
    if (m.version !== VERSION_PROTOCOLE) {
      annoncer("Cette fenêtre et SOURISO ne sont pas de la même version.", true);
      return;
    }
    for (const cle of ["effets", "parametres", "profils", "actif", "live", "base", "suivant", "adaptateur", "adaptateur_impose"]) {
      if (m[cle] !== undefined) etat[cle === "adaptateur_impose" ? "impose" : cle] = m[cle];
    }
    etat.faute = m.faute || null;
    ui.saisie = null;
    ui.confirmation = null;
    if (!ui.page || !pageValide(ui.page)) ui.page = etat.effets[0] ? etat.effets[0].id : "profils";
    if (etat.faute && !fauteVisible(etat.faute.cible)) annoncer(etat.faute.message, true);
    rendreTout();
    if (ui.focusApres) {
      focaliser(ui.focusApres());
      ui.focusApres = null;
    }
    if (!affichee) {
      affichee = true;
      afficher();
    }
  }

  /* --- Statut ------------------------------------------------------------------------------- */

  function annoncer(texte, alerte = false) {
    const s = $("#statut");
    s.textContent = texte;
    s.classList.toggle("alerte", alerte);
    s.setAttribute("role", alerte ? "alert" : "status");
    clearTimeout(delaiMessage);
    if (alerte) delaiMessage = setTimeout(majStatut, 7000);
  }

  function majStatut() {
    if (!profilActif()) return;
    const m = modifie();
    const s = $("#statut");
    s.classList.remove("alerte");
    s.setAttribute("role", "status");
    s.textContent = m ? "Modifié depuis l'ouverture · enregistré automatiquement" : "Enregistré automatiquement";
    $("#revenir").disabled = !m;
  }

  /* --- Rendu général ------------------------------------------------------------------------ */

  function pageValide(id) {
    return id === "profils" || id === "carte" || etat.effets.some((e) => e.id === id);
  }

  function rendreTout(cleFocus) {
    const cle = cleFocus || (document.activeElement && document.activeElement.dataset
      ? document.activeElement.dataset.cle : null);
    rendreBarre();
    rendreOnglets();
    rendrePage();
    if (cle) {
      const cible = [...document.querySelectorAll("[data-cle]")].find((e) => e.dataset.cle === cle);
      if (cible && cible !== document.activeElement) cible.focus({ preventScroll: true });
    }
  }

  function rendreBarre() {
    const select = $("#profil-actif");
    select.replaceChildren(...etat.profils.map((p, i) => h("option", { value: i }, p.nom)));
    select.value = String(etat.actif);
    select.onchange = () => envoyer({ type: "profil", action: "activer", indice: Number(select.value) });
    $("#revenir").onclick = () => envoyer({ type: "revenir" });
    majStatut();
  }

  function rendreOnglets() {
    const liste = $("#onglets");
    const elements = etat.effets.map((e) => ongletEffet(e));
    elements.push(h("div", { class: "separateur-nav", "aria-hidden": "true" }));
    elements.push(onglet("profils", "Profils et raccourcis"));
    elements.push(onglet("carte", "Carte graphique"));
    liste.replaceChildren(...elements);
    liste.onkeydown = surToucheOnglets;
  }

  function onglet(id, titre, etatTexte, coupe) {
    const choisi = ui.page === id;
    return h("button", {
      type: "button", role: "tab", class: "onglet", id: `onglet-${id}`, "data-id": id,
      "data-cle": `onglet-${id}`, "aria-selected": choisi ? "true" : "false",
      "aria-controls": "page", tabindex: choisi ? "0" : "-1",
      onclick: () => choisirPage(id),
    }, h("span", {}, titre), etatTexte ? h("span", { class: `etat${coupe ? " coupe" : ""}` }, etatTexte) : null);
  }

  function ongletEffet(e) {
    const actif = etat.live[e.id] !== false;
    return onglet(e.id, e.titre, actif ? "Actif" : "Coupé", !actif);
  }

  function choisirPage(id, parClavier) {
    if (ui.saisie) annulerSaisie();
    ui.page = id;
    ui.edition = null;
    ui.confirmation = null;
    rendreTout(parClavier ? `onglet-${id}` : undefined);
    $("#contenu").scrollTop = 0;
  }

  function surToucheOnglets(ev) {
    const ids = [...document.querySelectorAll("#onglets [role=tab]")].map((b) => b.dataset.id);
    const i = ids.indexOf(ui.page);
    const cible = { ArrowDown: i + 1, ArrowUp: i - 1, Home: 0, End: ids.length - 1 }[ev.key];
    if (cible === undefined) return;
    ev.preventDefault();
    choisirPage(ids[(cible + ids.length) % ids.length], true);
  }

  function rendrePage() {
    const racine = $("#page");
    racine.setAttribute("aria-labelledby", `onglet-${ui.page}`);
    const effet = etat.effets.find((e) => e.id === ui.page);
    if (effet) racine.replaceChildren(...pageEffet(effet));
    else if (ui.page === "profils") racine.replaceChildren(...pageProfils());
    else racine.replaceChildren(...pageCarte());
  }

  /* --- Page d'un effet ---------------------------------------------------------------------- */

  function pageEffet(e) {
    const actif = etat.live[e.id] !== false;
    const interrupteur = h("button", {
      type: "button", role: "switch", class: "interrupteur", "aria-checked": String(actif),
      "data-cle": `interrupteur-${e.id}`, onclick: () => basculer(e.id),
    }, actif ? "Activé" : "Coupé", h("span", { class: "piste", "aria-hidden": "true" }));
    const notes = [
      h("p", { class: "note" },
        `L'interrupteur est enregistré dans le profil « ${profilActif().nom} ». Le raccourci ${e.raccourci} `
        + "coupe ou rétablit l'effet en direct, sans modifier le profil."),
    ];
    if (!actif) {
      notes.push(h("p", { class: "note coupe" }, "Cet effet est coupé : activez-le pour voir vos changements en direct."));
    }
    if (e.id === "projecteur") {
      notes.push(h("p", { class: "note" },
        "Quand le pointeur est sur cette fenêtre, le voile n'est pas affiché pour qu'elle reste lisible. "
        + "Sortez de la fenêtre pour voir le cercle éclairé."));
    }
    const reglages = etat.parametres.filter((p) => p.effet === e.id).map(reglage);
    return [
      h("div", { class: "entete" },
        h("div", {}, h("h1", {}, e.titre), h("p", { class: "description" }, e.description)),
        interrupteur),
      ...notes,
      h("div", { class: "groupe" }, reglages),
    ];
  }

  function basculer(id) {
    const v = etat.live[id] === false;
    etat.live[id] = v;
    profilActif().actifs[id] = v;
    regler(`actif.${id}`, v);
    rendreTout();
  }

  function reglage(p) {
    const id = `r-${p.chemin.replace(/[._]/g, "-")}`;
    const fabrique = { nombre: reglageNombre, couleur: reglageCouleur, son: reglageSon }[p.genre];
    return h("div", { class: "reglage" },
      h("label", { class: "libelle", for: id }, p.libelle),
      h("p", { class: "aide", id: `${id}-aide` }, p.aide),
      h("div", { class: "controle" }, fabrique(p, id)));
  }

  function decimales(pas) {
    return (String(pas).split(".")[1] || "").length;
  }

  function reglageNombre(p, id) {
    const dec = decimales(p.pas);
    const arrondir = (x) => Number(x.toFixed(dec));
    const texte = (x) => `${x} ${MOTS_UNITE[p.unite] || p.unite}`;
    const v = valeur(p.chemin);
    const curseur = h("input", {
      type: "range", id, min: p.min, max: p.max, step: p.pas, value: v, "data-cle": id,
      "aria-describedby": `${id}-aide`, "aria-valuetext": texte(v),
    });
    const champ = h("input", {
      type: "number", min: p.min, max: p.max, step: p.pas, value: v, "data-cle": `${id}-n`,
      "aria-label": `${p.libelle}, valeur en ${MOTS_UNITE[p.unite] || p.unite}`,
    });
    const appliquer = (x) => {
      profilActif().valeurs[p.chemin] = x;
      curseur.setAttribute("aria-valuetext", texte(x));
      regler(p.chemin, x);
    };
    curseur.addEventListener("input", () => {
      const x = arrondir(Number(curseur.value));
      champ.value = x;
      appliquer(x);
    });
    champ.addEventListener("input", () => {
      const x = Number(champ.value);
      if (champ.value !== "" && Number.isFinite(x) && x >= p.min && x <= p.max) {
        curseur.value = x;
        appliquer(arrondir(x));
      }
    });
    champ.addEventListener("change", () => {
      const x = arrondir(Math.min(p.max, Math.max(p.min, Number(champ.value) || p.min)));
      champ.value = x;
      curseur.value = x;
      appliquer(x);
    });
    return [curseur, h("span", { class: "nombre" }, champ, h("span", { class: "unite" }, p.unite))];
  }

  function reglageCouleur(p, id) {
    const courante = valeur(p.chemin);
    const pastilles = PALETTE.map(([c, nom]) => {
      const b = h("button", {
        type: "button", class: "pastille", "aria-label": nom, title: nom,
        "aria-pressed": String(c === courante), "data-cle": `${id}-${c}`,
        onclick: () => definir(c),
      });
      b.style.background = c;
      b.dataset.couleur = c;
      return b;
    });
    const choix = h("input", {
      type: "color", id, value: courante, "data-cle": id, "aria-describedby": `${id}-aide`,
      title: "Choisir une autre couleur",
    });
    function definir(c) {
      profilActif().valeurs[p.chemin] = c;
      choix.value = c;
      for (const b of pastilles) b.setAttribute("aria-pressed", String(b.dataset.couleur === c));
      regler(p.chemin, c);
    }
    choix.addEventListener("input", () => definir(choix.value.toLowerCase()));
    return [h("div", { class: "pastilles" }, pastilles, choix)];
  }

  function reglageSon(p, id) {
    const select = h("select", { id, "data-cle": id, "aria-describedby": `${id}-aide` },
      p.choix.map((c) => h("option", { value: c.id }, c.nom)));
    select.value = valeur(p.chemin);
    select.addEventListener("change", () => {
      profilActif().valeurs[p.chemin] = select.value;
      regler(p.chemin, select.value);
    });
    const ecouter = bouton("Écouter", "ecouter", {
      "data-cle": `${id}-ecouter`,
      onclick: () => regler(p.chemin, select.value),
    });
    return [select, ecouter];
  }

  /* --- Page des profils --------------------------------------------------------------------- */

  function fauteVisible(cible) {
    if (ui.page !== "profils") return false;
    return cible === "nouveau" || cible === "suivant" || /^(profil|renommer):\d+$/.test(cible);
  }

  function messageFaute(cible) {
    return etat.faute && etat.faute.cible === cible
      ? h("p", { class: "erreur", role: "alert" }, icone("alerte"), etat.faute.message)
      : null;
  }

  function pageProfils() {
    const lignes = etat.profils.map(ligneProfil);
    return [
      h("h1", {}, "Profils et raccourcis"),
      h("p", { class: "description" },
        "Un profil regroupe les réglages de tous les effets, ceux qui sont activés et un raccourci pour y passer. "
        + "Les changements que vous faites s'enregistrent dans le profil choisi."),
      h("ul", { class: "liste-profils", "aria-label": "Profils" }, lignes),
      formulaireNouveau(),
      h("h2", {}, "Profil suivant"),
      h("p", { class: "description" }, "Ce raccourci passe au profil suivant, puis revient au premier après le dernier."),
      h("div", { class: "profil" },
        h("div", { class: "profil-ligne" },
          h("span", { class: "profil-nom" }, "Raccourci du profil suivant"),
          boutonRaccourci("suivant", etat.suivant, "du profil suivant")),
        messageFaute("suivant")),
    ];
  }

  function ligneProfil(p, i) {
    const idr = `profil-${i}`;
    const enEdition = ui.edition === i;
    const nom = enEdition
      ? h("input", {
        type: "text", value: ui.texteEdition, maxlength: "40", "data-cle": `nom-${i}`,
        "aria-label": `Nouveau nom du profil ${p.nom}`,
        oninput: (ev) => { ui.texteEdition = ev.target.value; },
        onkeydown: (ev) => {
          if (ev.key === "Enter") { ev.preventDefault(); validerNom(i); }
          if (ev.key === "Escape") { ev.preventDefault(); ui.edition = null; rendreTout(`renommer-${i}`); }
        },
      })
      : h("label", { for: idr }, p.nom, i === 0 ? h("span", { class: "etiquette" }, " (par défaut)") : null);
    const radio = h("input", {
      type: "radio", name: "profil-actif-choix", id: idr, "data-cle": idr, checked: i === etat.actif,
      onchange: () => envoyer({ type: "profil", action: "activer", indice: i }),
    });
    const actions = enEdition
      ? [bouton("Valider", null, { class: "bouton principal", onclick: () => validerNom(i) }),
        bouton("Annuler", null, { onclick: () => { ui.edition = null; rendreTout(`renommer-${i}`); } })]
      : [
        bouton("Renommer", "crayon", {
          "data-cle": `renommer-${i}`,
          onclick: () => { ui.edition = i; ui.texteEdition = p.nom; ui.confirmation = null; rendreTout(); focaliser(`nom-${i}`); },
        }),
        bouton("Dupliquer", "copie", {
          "data-cle": `dupliquer-${i}`,
          onclick: () => {
            // La copie devient le profil actif : on propose de la renommer.
            ui.focusApres = () => `renommer-${etat.actif}`;
            envoyer({ type: "profil", action: "dupliquer", indice: i });
          },
        }),
        bouton("Supprimer", "corbeille", {
          class: "bouton danger", "data-cle": `supprimer-${i}`, disabled: i === 0,
          title: i === 0 ? "Le profil par défaut ne se supprime pas" : null,
          "aria-describedby": i === 0 ? "aide-defaut" : null,
          onclick: () => { ui.confirmation = i; ui.edition = null; rendreTout(); focaliser(`confirmer-${i}`); },
        }),
      ];
    return h("li", { class: `profil${i === etat.actif ? " actif" : ""}` },
      h("div", { class: "profil-ligne" },
        h("span", { class: "profil-nom" }, radio, nom),
        boutonRaccourci(i, p.raccourci, `du profil ${p.nom}`),
        h("div", { class: "profil-actions" }, actions)),
      i === 0 ? h("p", { class: "etiquette", id: "aide-defaut" }, "Le profil par défaut ne se supprime pas.") : null,
      ui.confirmation === i
        ? h("div", { class: "profil-confirmation", role: "group", "aria-label": "Confirmer la suppression" },
          h("span", {}, `Supprimer le profil « ${p.nom} » ?`),
          bouton("Supprimer", null, {
            class: "bouton danger", "data-cle": `confirmer-${i}`,
            onclick: () => {
              ui.focusApres = () => `profil-${etat.actif}`;
              envoyer({ type: "profil", action: "supprimer", indice: i });
            },
          }),
          bouton("Annuler", null, { "data-cle": `annuler-${i}`, onclick: () => { ui.confirmation = null; rendreTout(`supprimer-${i}`); } }))
        : null,
      messageFaute(`profil:${i}`),
      messageFaute(`renommer:${i}`));
  }

  function focaliser(cle) {
    const cible = [...document.querySelectorAll("[data-cle]")].find((e) => e.dataset.cle === cle);
    if (cible) cible.focus();
  }

  function validerNom(i) {
    ui.edition = null;
    ui.focusApres = () => `renommer-${i}`;
    envoyer({ type: "profil", action: "renommer", indice: i, nom: ui.texteEdition });
  }

  function formulaireNouveau() {
    const champ = h("input", {
      type: "text", id: "nouveau-nom", value: ui.nouveau, maxlength: "40", placeholder: "Nom du nouveau profil",
      "aria-label": "Nom du nouveau profil", "data-cle": "nouveau-nom",
      oninput: (ev) => { ui.nouveau = ev.target.value; },
    });
    const creer = () => {
      const nom = ui.nouveau;
      ui.nouveau = "";
      envoyer({ type: "profil", action: "creer", nom });
    };
    return h("form", {
      class: "nouveau", onsubmit: (ev) => { ev.preventDefault(); creer(); },
    }, champ, h("button", { type: "submit", class: "bouton principal", "data-cle": "nouveau-creer" }, icone("plus"), "Créer le profil"),
    messageFaute("nouveau"));
  }

  /* --- Saisie d'un raccourci ---------------------------------------------------------------- */

  function boutonRaccourci(cible, r, sujet) {
    const texte = r ? r.texte : "Aucun raccourci";
    const b = h("button", {
      type: "button", class: `raccourci${r ? "" : " vide"}`, "data-cle": `raccourci-${cible}`,
      "aria-label": `Raccourci ${sujet} : ${r ? r.texte : "aucun"}. Appuyer sur Entrée pour le modifier.`,
    }, texte);
    b.addEventListener("click", () => commencerSaisie(cible, b, texte));
    return b;
  }

  function commencerSaisie(cible, b, texteInitial) {
    if (ui.saisie) annulerSaisie();
    ui.saisie = { cible, b, texteInitial };
    b.classList.add("saisie");
    b.textContent = "Tapez la combinaison…";
    b.setAttribute("aria-label", "Tapez la nouvelle combinaison. Échap pour annuler, Suppr pour effacer.");
    envoyer({ type: "capture", debut: true });
    annoncer("Tapez la combinaison : Ctrl ou Alt, plus une lettre de A à Z. Échap annule, Suppr efface.");
    b.addEventListener("keydown", surToucheSaisie);
    b.addEventListener("blur", surPerteFocusSaisie);
  }

  function terminerSaisie() {
    const s = ui.saisie;
    ui.saisie = null;
    if (s) {
      s.b.removeEventListener("keydown", surToucheSaisie);
      s.b.removeEventListener("blur", surPerteFocusSaisie);
    }
    return s;
  }

  function annulerSaisie() {
    const s = terminerSaisie();
    if (!s) return;
    s.b.classList.remove("saisie");
    s.b.textContent = s.texteInitial;
    envoyer({ type: "capture", debut: false });
    majStatut();
  }

  function surPerteFocusSaisie() {
    if (ui.saisie) annulerSaisie();
  }

  function surToucheSaisie(ev) {
    if (ev.key === "Tab") return;
    ev.preventDefault();
    ev.stopPropagation();
    if (ev.repeat) return;
    if (ev.key === "Escape") { annulerSaisie(); return; }
    if (["Control", "Alt", "Shift", "Meta", "AltGraph", "OS"].includes(ev.key)) return;
    const s = ui.saisie;
    const cible = s.cible;
    if (ev.key === "Delete" || ev.key === "Backspace") {
      terminerSaisie();
      envoyer({ type: "raccourci", cible, raccourci: null });
      return;
    }
    const lettre = /^Key([A-Z])$/.exec(ev.code);
    if (!lettre) { annoncer("Seules les lettres de A à Z sont prises en charge.", true); return; }
    if (!ev.ctrlKey && !ev.altKey) { annoncer("Ajoutez Ctrl ou Alt à la lettre.", true); return; }
    terminerSaisie();
    envoyer({
      type: "raccourci", cible,
      raccourci: { ctrl: ev.ctrlKey, alt: ev.altKey, maj: ev.shiftKey, touche: lettre[1] },
    });
  }

  /* --- Carte graphique ---------------------------------------------------------------------- */

  const CARTES = [
    ["automatique", "Automatique", "SOURISO choisit la carte qui porte votre écran principal."],
    ["econome", "Économe", "La carte la plus sobre, en général la carte intégrée : moins de chaleur et de batterie."],
    ["performant", "Performant", "La carte la plus puissante, en général la carte dédiée."],
  ];

  function pageCarte() {
    const impose = etat.impose;
    const radios = CARTES.map(([id, titre, aide]) => h("label", {},
      h("input", {
        type: "radio", name: "carte", value: id, checked: etat.adaptateur === id, disabled: !!impose,
        "data-cle": `carte-${id}`,
        onchange: () => { etat.adaptateur = id; envoyer({ type: "adaptateur", valeur: id }); majCarte(); },
      }),
      h("span", {}, h("span", { class: "titre" }, titre), h("span", { class: "aide" }, aide))));
    return [
      h("h1", {}, "Carte graphique"),
      h("p", { class: "description" }, "Sur un ordinateur qui a deux cartes graphiques, choisit celle qui dessine les effets."),
      impose
        ? h("p", { class: "note" }, `Ce choix est imposé par la variable d'environnement ${impose} : le réglage ci-dessous est sans effet tant qu'elle est définie.`)
        : null,
      h("div", { class: "choix", role: "radiogroup", "aria-label": "Carte graphique" }, radios),
      h("p", { class: "note", id: "note-carte" }, "Ce choix s'appliquera au prochain lancement de SOURISO."),
    ];
  }

  function majCarte() {
    annoncer("Ce choix s'appliquera au prochain lancement de SOURISO.");
  }

  /* --- Démarrage ---------------------------------------------------------------------------- */

  function demarrer() {
    if (!T) {
      annoncer("Cette page est la fenêtre de réglages de SOURISO : ouvrez-la depuis l'icône de SOURISO.", true);
      return;
    }
    T.event.listen(EVENEMENT, (e) => {
      try {
        recevoir(JSON.parse(e.payload));
      } catch (erreur) {
        annoncer("Message illisible reçu de SOURISO.", true);
      }
    }).then(() => envoyerBrut({ type: "pret" }));
    // Sans réponse du processus principal, la fenêtre s'affiche quand même, avec le constat.
    setTimeout(() => {
      if (!affichee) {
        affichee = true;
        annoncer("SOURISO ne répond pas. Fermez cette fenêtre et rouvrez-la depuis l'icône.", true);
        afficher();
      }
    }, DELAI_SECOURS_MS);
  }

  // Le script est différé : le document est lu en entier.
  $("#icone-revenir").replaceChildren(icone("revenir").firstChild);
  demarrer();
})();
