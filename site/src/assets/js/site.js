// Shared behaviour of every page: theme toggle, mobile menu, copy buttons, docs navigation.

const root = document.documentElement;

function currentTheme() {
  if (root.dataset.theme) return root.dataset.theme;
  return matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export function setTheme(theme) {
  root.dataset.theme = theme;
  try {
    localStorage.setItem("fps-theme", theme);
  } catch {
    // Private mode: the choice lasts for this page only.
  }
  document.dispatchEvent(new CustomEvent("fps-site-theme", { detail: theme }));
}

document.querySelector(".theme-toggle")?.addEventListener("click", () => setTheme(currentTheme() === "dark" ? "light" : "dark"));

const menu = document.querySelector(".menu-toggle");
const nav = document.querySelector(".main-nav");
menu?.addEventListener("click", () => {
  const open = nav?.classList.toggle("open") ?? false;
  menu.setAttribute("aria-expanded", String(open));
  document.querySelector(".docs-side")?.classList.toggle("open", open);
});

async function copy(text, button) {
  try {
    await navigator.clipboard.writeText(text);
    const label = button.textContent;
    button.textContent = "Copied";
    setTimeout(() => (button.textContent = label), 1400);
  } catch {
    button.textContent = "Press Ctrl+C";
  }
}

document.addEventListener("click", (e) => {
  const button = e.target instanceof Element ? e.target.closest("button.copy, button[data-copy]") : null;
  if (!button) return;
  const text = button.dataset.copy ?? button.closest(".code")?.querySelector("pre")?.innerText ?? "";
  void copy(text.trimEnd(), button);
});

// Docs: filter the sidebar, highlight the heading in view.
const filter = document.querySelector(".side-search input");
filter?.addEventListener("input", () => {
  const q = filter.value.trim().toLowerCase();
  for (const a of document.querySelectorAll(".side-section a")) {
    a.hidden = q !== "" && !a.textContent.toLowerCase().includes(q);
  }
  for (const s of document.querySelectorAll(".side-section")) {
    s.hidden = [...s.querySelectorAll("a")].every((a) => a.hidden);
  }
});

const tocLinks = [...document.querySelectorAll(".on-page a")];
if (tocLinks.length && "IntersectionObserver" in window) {
  const byId = new Map(tocLinks.map((a) => [decodeURIComponent(a.hash.slice(1)), a]));
  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        for (const a of tocLinks) a.classList.remove("active");
        byId.get(entry.target.id)?.classList.add("active");
      }
    },
    { rootMargin: "-80px 0px -70% 0px" },
  );
  for (const id of byId.keys()) {
    const h = document.getElementById(id);
    if (h) observer.observe(h);
  }
}

// Tabs (framework examples): <div class="tabs"> with buttons[data-tab] and .tab-panel[data-tab].
for (const tabs of document.querySelectorAll(".tabs")) {
  const buttons = [...tabs.querySelectorAll(".tab-list button")];
  const select = (name) => {
    for (const b of buttons) b.setAttribute("aria-selected", String(b.dataset.tab === name));
    for (const p of tabs.querySelectorAll(".tab-panel")) p.classList.toggle("active", p.dataset.tab === name);
  };
  for (const b of buttons) b.addEventListener("click", () => select(b.dataset.tab));
}
