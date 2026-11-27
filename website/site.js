// Lets the styles know that the menus below can open and close.
document.documentElement.classList.add("js");

// The site root, from this script's own address (site.js sits at the root).
const siteRoot = new URL(".", document.currentScript.src);

// Graphics card choice: NVIDIA swaps the ISO links and the image name.
// Remembered per browser; ?gpu=nvidia in the address selects it too.
const setGpu = (gpu) => {
  const nvidia = gpu === "nvidia";
  for (const link of document.querySelectorAll("[data-nvidia-href]")) {
    link.dataset.mainHref ??= link.getAttribute("href");
    link.setAttribute("href", nvidia ? link.dataset.nvidiaHref : link.dataset.mainHref);
  }
  for (const el of document.querySelectorAll("[data-nvidia-only]")) el.hidden = !nvidia;
  for (const el of document.querySelectorAll("[data-main-only]")) el.hidden = nvidia;
  for (const el of document.querySelectorAll("[data-image]")) {
    el.textContent = nvidia ? "celenn-nvidia" : "celenn";
  }
  document.querySelector(`input[name="gpu"][value="${nvidia ? "nvidia" : "main"}"]`).checked = true;
};
// Only the home pages have the choice (not the documentation pages).
if (document.querySelector('input[name="gpu"]')) {
  let gpu = new URLSearchParams(location.search).get("gpu");
  try { gpu ??= localStorage.getItem("gpu"); } catch {}
  setGpu(gpu);
}
for (const input of document.querySelectorAll('input[name="gpu"]')) {
  input.addEventListener("change", () => {
    setGpu(input.value);
    try { localStorage.setItem("gpu", input.value); } catch {}
  });
}

// Copy buttons: labels follow the page language.
const allLabels = {
  en: { copy: "Copy", copied: "Copied", failed: "Select and copy", search: "Search the documentation", none: "No results", close: "Close", previous: "Previous", next: "Next" },
  de: { copy: "Kopieren", copied: "Kopiert", failed: "Markieren und kopieren", search: "Dokumentation durchsuchen", none: "Keine Ergebnisse", close: "Schließen", previous: "Zurück", next: "Weiter" },
  es: { copy: "Copiar", copied: "Copiado", failed: "Selecciona y copia", search: "Buscar en la documentación", none: "Sin resultados", close: "Cerrar", previous: "Anterior", next: "Siguiente" },
  fr: { copy: "Copier", copied: "Copié", failed: "Sélectionnez et copiez", search: "Rechercher dans la documentation", none: "Aucun résultat", close: "Fermer", previous: "Précédente", next: "Suivante" },
  it: { copy: "Copia", copied: "Copiato", failed: "Seleziona e copia", search: "Cerca nella documentazione", none: "Nessun risultato", close: "Chiudi", previous: "Precedente", next: "Successiva" },
  "pt-BR": { copy: "Copiar", copied: "Copiado", failed: "Selecione e copie", search: "Buscar na documentação", none: "Nenhum resultado", close: "Fechar", previous: "Anterior", next: "Próxima" },
};
const labels = allLabels[document.documentElement.lang] ?? allLabels.en;

// Screen readers do not always read a button whose label changes, so the
// result is also said in a hidden status region.
const copyButtons = document.querySelectorAll("[data-copy]");
let copyStatus;
if (copyButtons.length) {
  copyStatus = document.createElement("span");
  copyStatus.className = "visually-hidden";
  copyStatus.setAttribute("role", "status");
  document.body.append(copyStatus);
}
for (const button of copyButtons) {
  button.addEventListener("click", async () => {
    const text = button.previousElementSibling.innerText;
    try {
      await navigator.clipboard.writeText(text);
      button.textContent = labels.copied;
    } catch {
      button.textContent = labels.failed;
    }
    copyStatus.textContent = button.textContent;
    setTimeout(() => {
      button.textContent = labels.copy;
      copyStatus.textContent = "";
    }, 2000);
  });
}

// Dates (screenshots page): written as 2026-11-13 by build.sh, shown in the
// page language.
for (const time of document.querySelectorAll("time[datetime]")) {
  const date = new Date(`${time.dateTime}T12:00:00Z`);
  if (!Number.isNaN(date.getTime())) {
    time.textContent = date.toLocaleDateString(document.documentElement.lang, {
      day: "numeric", month: "long", year: "numeric", timeZone: "UTC",
    });
  }
}

// Language and site menus: close them on a click elsewhere or with Escape.
const menus = document.querySelectorAll(".top details");
document.addEventListener("click", (event) => {
  for (const menu of menus) {
    if (menu.open && !menu.contains(event.target)) menu.open = false;
  }
});
document.addEventListener("keydown", (event) => {
  if (event.key !== "Escape") return;
  for (const menu of menus) {
    if (menu.open) {
      menu.open = false;
      menu.querySelector("summary").focus();
    }
  }
});
for (const menu of menus) {
  menu.addEventListener("toggle", () => {
    if (menu.open) for (const other of menus) if (other !== menu) other.open = false;
  });
}

// Documentation side menu: marks the section being read, and on phones
// folds into a bar that names it.
const docSide = document.querySelector(".doc-side");
if (docSide) {
  const toggle = docSide.querySelector(".doc-toggle");
  const current = toggle.querySelector("span");
  const links = [...docSide.querySelectorAll('.doc-nav a[href^="#"]')];
  const targets = links.map((link) => document.getElementById(link.hash.slice(1))).filter(Boolean);
  const setOpen = (open) => {
    docSide.classList.toggle("open", open);
    toggle.setAttribute("aria-expanded", String(open));
  };
  toggle.addEventListener("click", () => setOpen(!docSide.classList.contains("open")));
  for (const link of links) link.addEventListener("click", () => setOpen(false));
  document.addEventListener("click", (event) => {
    if (!docSide.contains(event.target)) setOpen(false);
  });

  // The section being read: the last one whose title went above the middle
  // of the screen
  const mark = () => {
    const line = window.innerHeight * 0.35;
    let active = targets[0];
    for (const target of targets) {
      if (target.getBoundingClientRect().top <= line) active = target;
    }
    for (const link of links) {
      if (link.hash === `#${active?.id}`) {
        link.setAttribute("aria-current", "location");
        current.textContent = link.textContent;
      } else {
        link.removeAttribute("aria-current");
      }
    }
  };
  let queued = false;
  window.addEventListener("scroll", () => {
    if (queued) return;
    queued = true;
    requestAnimationFrame(() => { queued = false; mark(); });
  }, { passive: true });
  mark();
}

// Documentation search: Pagefind, a static index built with the site
// (.github/workflows/pages.yml). The button shows only when the index is
// there, so a site built without it has no broken search.
const searchButton = document.querySelector("[data-search]");
if (searchButton) {
  let pagefind;
  let dialog;
  const escape = (text) => text.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
  // Pagefind marks the words found with <mark> and escapes the rest
  const results = async (query) => {
    const found = await pagefind.debouncedSearch(query, {}, 150);
    if (found === null) return null;
    const items = [];
    for (const result of found.results.slice(0, 6)) {
      const data = await result.data();
      for (const sub of (data.sub_results?.length ? data.sub_results : [data]).slice(0, 3)) {
        items.push(`<li><a href="${escape(sub.url)}"><strong>${escape(sub.title ?? data.meta.title)}</strong><span>${sub.excerpt}</span></a></li>`);
      }
    }
    return items.length ? items.join("") : `<li class="empty">${labels.none}</li>`;
  };
  const open = async () => {
    if (!dialog) {
      dialog = document.createElement("dialog");
      dialog.className = "search";
      dialog.innerHTML = `<form method="dialog" role="search"><svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></svg><input type="search" aria-label="${labels.search}" placeholder="${labels.search}" autocomplete="off"><button class="close" value="close">${labels.close}</button></form><ol aria-live="polite"></ol>`;
      document.body.append(dialog);
      const input = dialog.querySelector("input");
      const list = dialog.querySelector("ol");
      input.addEventListener("input", async () => {
        const query = input.value.trim();
        if (!query) { list.innerHTML = ""; return; }
        const html = await results(query);
        if (html !== null) list.innerHTML = html;
      });
      input.addEventListener("keydown", (event) => {
        if (event.key === "Enter") {
          event.preventDefault();
          list.querySelector("a")?.click();
        }
      });
      dialog.addEventListener("click", (event) => {
        if (event.target === dialog || event.target.closest("a")) dialog.close();
      });
    }
    dialog.showModal();
    dialog.querySelector("input").select();
  };
  import(new URL("pagefind/pagefind.js", siteRoot).href).then(async (module) => {
    pagefind = module;
    await pagefind.options({ baseUrl: siteRoot.pathname });
    searchButton.hidden = false;
    searchButton.addEventListener("click", open);
    document.addEventListener("keydown", (event) => {
      if (event.key === "/" && !event.target.closest("input, textarea") && !dialog?.open) {
        event.preventDefault();
        open();
      }
    });
  }).catch(() => {});
}

// Screenshots: a full size viewer instead of opening the image alone; the
// link still works without JavaScript.
const shotLinks = [...document.querySelectorAll(".gallery figure > a")];
if (shotLinks.length && window.HTMLDialogElement) {
  const viewer = document.createElement("dialog");
  viewer.className = "lightbox";
  viewer.innerHTML = `<header><button type="button" data-step="-1" aria-label="${labels.previous}">←</button><button type="button" data-step="1" aria-label="${labels.next}">→</button><button type="button" data-close aria-label="${labels.close}">✕</button></header><img alt=""><div class="caption"></div>`;
  document.body.append(viewer);
  const image = viewer.querySelector("img");
  const caption = viewer.querySelector(".caption");
  let index = 0;
  const show = (i) => {
    index = (i + shotLinks.length) % shotLinks.length;
    const link = shotLinks[index];
    image.src = link.href;
    image.alt = link.querySelector("img").alt;
    const figcaption = link.parentElement.querySelector("figcaption");
    caption.innerHTML = figcaption ? figcaption.innerHTML : "";
  };
  shotLinks.forEach((link, i) => link.addEventListener("click", (event) => {
    event.preventDefault();
    show(i);
    viewer.showModal();
  }));
  viewer.addEventListener("click", (event) => {
    const step = event.target.closest("[data-step]");
    if (step) show(index + Number(step.dataset.step));
    else if (event.target.closest("[data-close]") || event.target === viewer || event.target === image.parentElement) viewer.close();
  });
  viewer.addEventListener("keydown", (event) => {
    if (event.key === "ArrowLeft") show(index - 1);
    if (event.key === "ArrowRight") show(index + 1);
  });
}
