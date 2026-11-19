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
    el.textContent = nvidia ? "cosmic-blue-nvidia" : "cosmic-blue";
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
const labels = {
  en: { copy: "Copy", copied: "Copied", failed: "Select and copy" },
  de: { copy: "Kopieren", copied: "Kopiert", failed: "Markieren und kopieren" },
  es: { copy: "Copiar", copied: "Copiado", failed: "Selecciona y copia" },
  fr: { copy: "Copier", copied: "Copié", failed: "Sélectionnez et copiez" },
  it: { copy: "Copia", copied: "Copiato", failed: "Seleziona e copia" },
  "pt-BR": { copy: "Copiar", copied: "Copiado", failed: "Selecione e copie" },
}[document.documentElement.lang] ?? { copy: "Copy", copied: "Copied", failed: "Select and copy" };

for (const button of document.querySelectorAll("[data-copy]")) {
  button.addEventListener("click", async () => {
    const text = button.previousElementSibling.innerText;
    try {
      await navigator.clipboard.writeText(text);
      button.textContent = labels.copied;
    } catch {
      button.textContent = labels.failed;
    }
    setTimeout(() => { button.textContent = labels.copy; }, 2000);
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

// Language menu: close it on a click elsewhere or with Escape.
const langMenu = document.querySelector("details.lang");
if (langMenu) {
  document.addEventListener("click", (event) => {
    if (langMenu.open && !langMenu.contains(event.target)) langMenu.open = false;
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && langMenu.open) {
      langMenu.open = false;
      langMenu.querySelector("summary").focus();
    }
  });
}
