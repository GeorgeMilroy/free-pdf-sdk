// Home page: reveal-on-scroll, parallax and the hero tilt, the counters, the Layers story, the
// menu mock, the mode comparison slider and the modes showcase (from the generated reference).

const reduced = matchMedia("(prefers-reduced-motion: reduce)");
// The motion setup below is decided once; a change of preference reloads the page.
reduced.addEventListener("change", () => location.reload());
const finePointer = matchMedia("(pointer: fine) and (min-width: 901px)");
const reference = JSON.parse(document.getElementById("reference")?.textContent ?? "{}");

function esc(s) {
  return String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

// Observes elements once; `onEnter` runs the first time each one is in view.
function whenVisible(elements, onEnter, options = {}) {
  if (!elements.length) return;
  if (reduced.matches || !("IntersectionObserver" in window)) {
    for (const el of elements) onEnter(el);
    return;
  }
  const io = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      io.unobserve(entry.target);
      onEnter(entry.target);
    }
  }, options);
  for (const el of elements) io.observe(el);
}

// --- Reveal ---------------------------------------------------------------------------------
whenVisible([...document.querySelectorAll(".reveal")], (el) => el.classList.add("in"), { rootMargin: "0px 0px -8% 0px", threshold: 0.08 });

// --- Parallax: each element moves against the scroll by its own factor -----------------------
const parallax = [...document.querySelectorAll("[data-parallax]")];
if (parallax.length && !reduced.matches) {
  let queued = false;
  const update = () => {
    queued = false;
    const middle = innerHeight / 2;
    for (const el of parallax) {
      const r = el.getBoundingClientRect();
      const distance = r.top + r.height / 2 - middle;
      el.style.setProperty("--py", `${(distance * parseFloat(el.dataset.parallax)).toFixed(1)}px`);
    }
  };
  addEventListener(
    "scroll",
    () => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(update);
    },
    { passive: true },
  );
  update();
}

// --- Hero tilt: the screenshot follows the pointer, the cards move by their depth -----------
const stage = document.querySelector("[data-tilt]");
if (stage && !reduced.matches && finePointer.matches) {
  const shot = stage.querySelector(".hero-shot");
  const cards = [...stage.querySelectorAll(".float-card")];
  stage.addEventListener("pointermove", (e) => {
    const r = stage.getBoundingClientRect();
    const x = (e.clientX - r.left) / r.width - 0.5;
    const y = (e.clientY - r.top) / r.height - 0.5;
    shot.style.setProperty("--ry", `${(-9 + x * 10).toFixed(2)}deg`);
    shot.style.setProperty("--rx", `${(4 - y * 8).toFixed(2)}deg`);
    for (const card of cards) {
      const depth = Number(card.dataset.depth) || 1;
      card.style.setProperty("--fx", `${(x * 10 * depth).toFixed(1)}px`);
      card.style.setProperty("--fy", `${(y * 10 * depth).toFixed(1)}px`);
    }
  });
  stage.addEventListener("pointerleave", () => {
    shot.style.removeProperty("--rx");
    shot.style.removeProperty("--ry");
    for (const card of cards) {
      card.style.removeProperty("--fx");
      card.style.removeProperty("--fy");
    }
  });
}

// --- Counters -------------------------------------------------------------------------------
whenVisible([...document.querySelectorAll("[data-count]")], (el) => {
  const target = Number(el.dataset.count);
  if (reduced.matches || target === 0) {
    el.textContent = String(target);
    return;
  }
  // The HTML holds the real number (for crawlers and no-JS); count up from 0 once visible.
  el.textContent = "0";
  const start = performance.now();
  const tick = (now) => {
    const t = Math.min(1, (now - start) / 1300);
    const eased = 1 - Math.pow(1 - t, 3);
    el.textContent = String(Math.round(target * eased));
    if (t < 1) requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
});

// --- Layers story: the steps drive the stage as they scroll by; the panel toggles by click --
const story = document.querySelector(".story");
if (story) {
  const steps = [...story.querySelectorAll(".story-step")];
  const names = ["structure", "furniture", "electrical", "dimensions"];
  const rows = new Map(names.map((n) => [n, story.querySelector(`.mock-row[data-layer="${n}"]`)]));
  const groups = new Map(names.map((n) => [n, story.querySelector(`.l-${n}`)]));
  const show = (visible) => {
    for (const n of names) {
      const on = visible.has(n);
      groups.get(n)?.classList.toggle("off", !on);
      rows.get(n)?.setAttribute("aria-pressed", String(on));
    }
  };
  const shown = () => new Set(names.filter((n) => rows.get(n)?.getAttribute("aria-pressed") === "true"));
  for (const [n, row] of rows) {
    row?.addEventListener("click", () => {
      const set = shown();
      if (set.has(n)) set.delete(n);
      else set.add(n);
      show(set);
    });
  }
  const activate = (step) => {
    for (const s of steps) s.classList.toggle("is-active", s === step);
    show(new Set(step.dataset.layers.split(" ")));
  };
  if ("IntersectionObserver" in window) {
    // The active step is the one filling most of the band: the middle of the viewport, or on
    // narrow screens the strip below the sticky stage. Every step's latest ratio is kept, so the
    // choice is among all of them, not only those that changed in a callback.
    const ratios = new Map(steps.map((s) => [s, 0]));
    const stageEl = story.querySelector(".story-stage");
    const narrow = matchMedia("(max-width: 900px)");
    let current = steps[0];
    let io = null;
    const connect = () => {
      io?.disconnect();
      let rootMargin = "-38% 0px -38% 0px";
      if (narrow.matches && stageEl) {
        const headerH = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--header-h")) || 64;
        const top = Math.round(headerH + stageEl.getBoundingClientRect().height);
        rootMargin = `-${top}px 0px -${Math.max(0, innerHeight - top - 240)}px 0px`;
      }
      for (const s of steps) ratios.set(s, 0);
      io = new IntersectionObserver(
        (entries) => {
          for (const e of entries) ratios.set(e.target, e.isIntersecting ? e.intersectionRatio : 0);
          let best = null;
          for (const [s, r] of ratios) if (r > 0 && (!best || r > ratios.get(best))) best = s;
          if (best && best !== current) {
            current = best;
            activate(best);
          }
        },
        { rootMargin, threshold: [0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1] },
      );
      for (const s of steps) io.observe(s);
    };
    connect();
    let resizeTimer = 0;
    addEventListener(
      "resize",
      () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(connect, 150);
      },
      { passive: true },
    );
  }
}

// --- Menu mock: a menu whose entries vanish as the mode changes -----------------------------
const MENU = [
  ["comments", "Add comment"],
  ["markup", "Highlight"],
  ["shapes", "Draw shape"],
  ["ink", "Freehand"],
  ["stamps", "Stamp"],
  ["measurement", "Measure"],
  ["fillSign", "Fill & Sign"],
  ["formFill", "Fill form"],
  ["formDesign", "Design form"],
  ["pages", "Organize pages"],
  ["editContent", "Edit text"],
  ["redaction", "Redact"],
  ["security", "Protect"],
  ["export", "Export"],
  ["save", "Save"],
  ["print", "Print"],
];
const menuList = document.querySelector("[data-menu-list]");
if (menuList && reference.modes) {
  const label = document.querySelector("[data-menu-mode]");
  const byKey = new Map(reference.modes.map((m) => [m.key, new Set(m.features)]));
  menuList.innerHTML = MENU.map(([key, text]) => `<li data-feature="${esc(key)}">${esc(text)}</li>`).join("");
  const items = [...menuList.querySelectorAll("li")];
  const apply = (mode) => {
    const on = byKey.get(mode);
    if (!on) return;
    if (label) label.textContent = mode;
    // Visible entries first, so the two-column list packs without holes where entries vanished.
    const kept = items.filter((li) => on.has(li.dataset.feature));
    const gone = items.filter((li) => !on.has(li.dataset.feature));
    for (const li of kept) li.classList.remove("off");
    for (const li of gone) li.classList.add("off");
    menuList.append(...kept, ...gone);
  };
  const order = ["full", "annotate", "view", "fill", "organize", "review", "edit", "sign"].filter((m) => byKey.has(m));
  let index = 1;
  apply(order[index]);
  if (!reduced.matches && order.length > 1) {
    let timer = 0;
    const card = menuList.closest(".bento-card") ?? menuList;
    const io = new IntersectionObserver(([entry]) => {
      clearInterval(timer);
      if (entry.isIntersecting) {
        timer = setInterval(() => {
          index = (index + 1) % order.length;
          apply(order[index]);
        }, 2600);
      }
    });
    io.observe(card);
  }
}

// --- Mode comparison slider -----------------------------------------------------------------
for (const compare of document.querySelectorAll("[data-compare]")) {
  const frame = compare.querySelector(".compare-frame");
  const range = compare.querySelector(".compare-range");
  const set = () => frame.style.setProperty("--pos", `${range.value}%`);
  range.addEventListener("input", set);
  set();
}

// --- Bento spotlight follows the pointer ----------------------------------------------------
if (finePointer.matches) {
  for (const card of document.querySelectorAll(".bento-card")) {
    card.addEventListener("pointermove", (e) => {
      const r = card.getBoundingClientRect();
      card.style.setProperty("--mx", `${(((e.clientX - r.left) / r.width) * 100).toFixed(1)}%`);
      card.style.setProperty("--my", `${(((e.clientY - r.top) / r.height) * 100).toFixed(1)}%`);
    });
  }
}

// --- Modes showcase -------------------------------------------------------------------------
const TITLES = {
  view: "Read-only viewing",
  review: "Comment review",
  annotate: "Annotations only",
  fill: "Forms",
  sign: "Forms and signatures",
  organize: "Page organizer",
  edit: "Full authoring",
  full: "Everything",
};

function code(mode) {
  return [
    `<span class="hljs-keyword">const</span> viewer = <span class="hljs-keyword">await</span> FreePdf.<span class="hljs-title function_">init</span>(element, {`,
    `  <span class="hljs-attr">baseUrl</span>: <span class="hljs-string">"/free-pdf-sdk/"</span>,`,
    `  <span class="hljs-attr">mode</span>: <span class="hljs-string">"${esc(mode.key)}"</span>,`,
    mode.key === "annotate" ? `  <span class="hljs-attr">features</span>: { <span class="hljs-attr">stamps</span>: <span class="hljs-literal">false</span> }, <span class="hljs-comment">// fine-tune any mode</span>` : null,
    `  <span class="hljs-attr">document</span>: { <span class="hljs-attr">url</span>: <span class="hljs-string">"/files/document.pdf"</span> },`,
    `});`,
  ]
    .filter(Boolean)
    .join("\n");
}

const showcase = document.getElementById("mode-showcase");
if (showcase && reference.modes) {
  const list = showcase.querySelector(".mode-list");
  const detail = showcase.querySelector(".mode-detail");
  const editing = reference.features.filter((f) => f.edits || ["history", "save", "download", "export"].includes(f.key));
  const select = (key) => {
    const mode = reference.modes.find((m) => m.key === key);
    for (const b of list.querySelectorAll("button")) {
      const selected = b.dataset.mode === key;
      b.setAttribute("aria-selected", String(selected));
      b.tabIndex = selected ? 0 : -1;
    }
    detail.setAttribute("aria-labelledby", `mode-tab-${key}`);
    detail.querySelector(".eyebrow").textContent = `mode: "${mode.key}"`;
    detail.querySelector("h3").textContent = TITLES[mode.key] ?? mode.key;
    detail.querySelector(".desc").textContent = mode.description;
    detail.querySelector(".chips").innerHTML = editing
      .map((f) => `<span class="chip${mode.features.includes(f.key) ? " on" : ""}" title="${esc(f.description)}">${esc(f.title)}</span>`)
      .join("");
    detail.querySelector(".mode-code").innerHTML = code(mode);
  };
  list.innerHTML = reference.modes
    .map((m) => `<button type="button" role="tab" id="mode-tab-${esc(m.key)}" aria-controls="mode-panel" tabindex="-1" data-mode="${esc(m.key)}"><span>${esc(m.key)}</span><small>${esc(TITLES[m.key] ?? "")}</small></button>`)
    .join("");
  const tabs = [...list.querySelectorAll("button")];
  for (const b of tabs) b.addEventListener("click", () => select(b.dataset.mode));
  // Arrow keys, Home and End move between the tabs, as a tab list expects.
  list.addEventListener("keydown", (e) => {
    const i = tabs.indexOf(document.activeElement);
    if (i < 0) return;
    const moves = { ArrowRight: i + 1, ArrowDown: i + 1, ArrowLeft: i - 1, ArrowUp: i - 1, Home: 0, End: tabs.length - 1 };
    if (!(e.key in moves)) return;
    e.preventDefault();
    const next = tabs[(moves[e.key] + tabs.length) % tabs.length];
    select(next.dataset.mode);
    next.focus();
  });
  select("annotate");
}
