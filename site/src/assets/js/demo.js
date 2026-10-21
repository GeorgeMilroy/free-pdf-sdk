// The live demo: the real SDK from /sdk/, configured from the panel.

import { FreePdf, FreePdfError } from "/sdk/free-pdf-sdk.js";

const reference = JSON.parse(document.getElementById("reference").textContent);
const $ = (id) => document.getElementById(id);
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
const params = new URLSearchParams(location.search);

const state = {
  mode: reference.modes.some((m) => m.key === params.get("mode")) ? params.get("mode") : "annotate",
  features: {},
  ui: {},
  panels: {},
  // The viewer starts in the site's theme (light unless the visitor chose dark).
  theme: document.documentElement.dataset.theme === "dark" ? "dark" : "light",
  locale: "en",
  author: "Demo user",
  doc: "/samples/demo.pdf",
};

let viewer = null;
let events = 0;

function log(name, data) {
  events += 1;
  $("event-count").textContent = `(${events})`;
  const line = document.createElement("div");
  const time = new Date().toLocaleTimeString();
  let detail = "";
  try {
    detail = JSON.stringify(data, (k, v) => (v instanceof Uint8Array ? `<${v.byteLength} bytes>` : v));
  } catch {
    detail = String(data);
  }
  line.innerHTML = `<span class="t">${esc(time)}</span> <b>${esc(name)}</b> ${esc(detail ?? "")}`;
  $("log").prepend(line);
}

// --- Controls built from the configuration reference ----------------------------------------------
function modeFeatures() {
  return new Set(reference.modes.find((m) => m.key === state.mode)?.features ?? []);
}

function toggle(id, label, desc, checked, changed, onChange) {
  const el = document.createElement("label");
  el.className = `toggle${changed ? " changed" : ""}`;
  el.innerHTML = `<span>${esc(label)}${desc ? `<small>${esc(desc)}</small>` : ""}</span><span class="switch"><input type="checkbox" id="${esc(id)}"${checked ? " checked" : ""}><i></i></span>`;
  el.querySelector("input").addEventListener("change", (e) => onChange(e.target.checked));
  return el;
}

function renderFeatures() {
  const box = $("features");
  box.textContent = "";
  const preset = modeFeatures();
  for (const category of [...new Set(reference.features.map((f) => f.category))]) {
    const h = document.createElement("h5");
    h.textContent = category;
    box.append(h);
    for (const f of reference.features.filter((x) => x.category === category)) {
      const on = state.features[f.key] ?? preset.has(f.key);
      box.append(
        toggle(`f-${f.key}`, f.title, f.description, on, f.key in state.features, (checked) => {
          if (checked === preset.has(f.key)) delete state.features[f.key];
          else state.features[f.key] = checked;
          renderFeatures();
          void apply("features");
        }),
      );
    }
  }
}

function renderUi() {
  const box = $("ui");
  box.textContent = "";
  for (const u of reference.ui) {
    const on = state.ui[u.key] ?? u.default;
    box.append(
      toggle(`u-${u.key}`, u.key, u.description, on, u.key in state.ui, (checked) => {
        if (checked === u.default) delete state.ui[u.key];
        else state.ui[u.key] = checked;
        renderUi();
        void apply("ui");
      }),
    );
  }
  const panels = $("panels");
  panels.querySelectorAll(".toggle").forEach((t) => t.remove());
  for (const p of reference.panels) {
    const on = state.panels[p.key] ?? true;
    panels.append(
      toggle(`p-${p.key}`, p.key, p.description, on, p.key in state.panels, (checked) => {
        if (checked) delete state.panels[p.key];
        else state.panels[p.key] = false;
        renderUi();
        void apply("ui");
      }),
    );
  }
}

function uiOptions() {
  const ui = { ...state.ui };
  if (Object.keys(state.panels).length) ui.panels = { ...state.panels };
  return ui;
}

// --- Code sample ----------------------------------------------------------------------------
function codeSample(flavour) {
  const q = (s) => JSON.stringify(String(s));
  const attr = (s) => String(s).replace(/&/g, "&amp;").replace(/"/g, "&quot;");
  const locale = state.locale !== "en" ? state.locale : null;
  const features = Object.keys(state.features).length ? state.features : null;
  const ui = Object.keys(uiOptions()).length ? uiOptions() : null;
  const json = (v) => JSON.stringify(v).replace(/"([a-zA-Z]+)":/g, "$1: ").replace(/,/g, ", ");
  if (flavour === "react") {
    return [
      `<FreePdfViewer`,
      `  baseUrl="/free-pdf-sdk/"`,
      `  mode="${state.mode}"`,
      features && `  features={${json(features)}}`,
      ui && `  ui={${json(ui)}}`,
      `  theme="${state.theme}"`,
      locale && `  locale="${locale}"`,
      `  author={${q(state.author)}}`,
      `  document={{ url: "/files/document.pdf" }}`,
      `  onAnnotationsChanged={({ count }) => console.log(count)}`,
      `  style={{ height: "80vh" }}`,
      `/>`,
    ].filter(Boolean).join("\n");
  }
  if (flavour === "vue") {
    return [
      `<FreePdfViewer`,
      `  base-url="/free-pdf-sdk/"`,
      `  mode="${state.mode}"`,
      features && `  :features="${json(features).replace(/"/g, "'")}"`,
      ui && `  :ui="${json(ui).replace(/"/g, "'")}"`,
      `  theme="${state.theme}"`,
      locale && `  locale="${locale}"`,
      `  author="${attr(state.author)}"`,
      `  :document="{ url: '/files/document.pdf' }"`,
      `  @annotations-changed="(e) => console.log(e.count)"`,
      `  style="height: 80vh"`,
      `/>`,
    ].filter(Boolean).join("\n");
  }
  if (flavour === "element") {
    return [
      `<free-pdf-viewer`,
      `  base-url="/free-pdf-sdk/"`,
      `  mode="${state.mode}"`,
      features && `  features='${JSON.stringify(features)}'`,
      ui && `  ui='${JSON.stringify(ui)}'`,
      `  theme="${state.theme}"`,
      locale && `  locale="${locale}"`,
      `  author="${attr(state.author)}"`,
      `  src="/files/document.pdf"`,
      `  style="height: 80vh"></free-pdf-viewer>`,
    ].filter(Boolean).join("\n");
  }
  return [
    `import { FreePdf } from "free-pdf-sdk";`,
    ``,
    `const viewer = await FreePdf.init(document.getElementById("viewer"), {`,
    `  baseUrl: "/free-pdf-sdk/",`,
    `  mode: "${state.mode}",`,
    features && `  features: ${json(features)},`,
    ui && `  ui: ${json(ui)},`,
    `  theme: "${state.theme}",`,
    `  locale: ${q(state.locale)},`,
    `  author: ${q(state.author)},`,
    `  document: { url: "/files/document.pdf" },`,
    `});`,
  ].filter((l) => l !== null && l !== false && l !== undefined).join("\n");
}

function renderCode() {
  const flavour = $("code-flavour").value;
  $("code-lang").textContent = { js: "ts", react: "tsx", vue: "vue", element: "html" }[flavour];
  $("code").textContent = codeSample(flavour);
}

// --- Applying changes ------------------------------------------------------------------------
async function apply(what) {
  renderCode();
  if (!viewer) return;
  try {
    if (what === "mode") await viewer.setMode(state.mode);
    if (what === "features") await viewer.setFeatures({ ...state.features });
    if (what === "ui") await viewer.setUi({ ...Object.fromEntries(reference.ui.map((u) => [u.key, state.ui[u.key] ?? u.default])), panels: Object.fromEntries(reference.panels.map((p) => [p.key, state.panels[p.key] ?? true])) });
    if (what === "theme") await viewer.setTheme(state.theme);
    if (what === "locale") await viewer.setLocale(state.locale);
    if (what === "author") await viewer.setAuthor(state.author);
  } catch (e) {
    log("error", e instanceof FreePdfError ? { code: e.code, message: e.message } : String(e));
  }
}

async function openDocument(source) {
  if (!viewer) return;
  try {
    const result = await viewer.load(source);
    $("unlock-row").hidden = !result.permissions.canUnlock;
  } catch (e) {
    log("error", e instanceof FreePdfError ? { code: e.code, message: e.message } : String(e));
  }
}

function download(name, data, type) {
  const url = URL.createObjectURL(new Blob([data], { type }));
  const a = Object.assign(document.createElement("a"), { href: url, download: name });
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

async function action(name) {
  if (!viewer) return;
  try {
    if (name === "import") {
      const xfdf = await (await fetch("/samples/review.xfdf")).text();
      log("importXfdf", { imported: await viewer.importXfdf(xfdf) });
    }
    if (name === "export") {
      const xfdf = await viewer.exportXfdf();
      log("exportXfdf", { characters: xfdf.length });
      download("comments.xfdf", xfdf, "application/vnd.adobe.xfdf");
    }
    if (name === "list") log("getAnnotations", await viewer.getAnnotations());
    if (name === "delete") log("deleteAllAnnotations", { deleted: await viewer.deleteAllAnnotations() });
    if (name === "save") {
      const pdf = await viewer.save();
      log("save", { bytes: pdf.byteLength });
      download("document.pdf", pdf, "application/pdf");
    }
    if (name === "config") log("getConfig", await viewer.getConfig());
  } catch (e) {
    log("error", e instanceof FreePdfError ? { code: e.code, message: e.message } : String(e));
  }
}

// --- Wiring ---------------------------------------------------------------------------------
function init() {
  const modeSelect = $("mode");
  modeSelect.innerHTML = reference.modes.map((m) => `<option value="${esc(m.key)}">${esc(m.key)}</option>`).join("");
  modeSelect.value = state.mode;
  const describe = () => ($("mode-desc").textContent = reference.modes.find((m) => m.key === state.mode)?.description ?? "");
  describe();
  modeSelect.addEventListener("change", () => {
    state.mode = modeSelect.value;
    describe();
    renderFeatures();
    void apply("mode");
  });
  $("reset-features").addEventListener("click", () => {
    state.features = {};
    renderFeatures();
    void apply("features");
  });
  $("theme").value = state.theme;
  $("theme").addEventListener("change", (e) => {
    state.theme = e.target.value;
    void apply("theme");
  });
  // The site's theme toggle switches the viewer too.
  document.addEventListener("fps-site-theme", (e) => {
    state.theme = e.detail;
    $("theme").value = state.theme;
    void apply("theme");
  });
  $("locale").addEventListener("change", (e) => {
    state.locale = e.target.value;
    void apply("locale");
  });
  $("author").addEventListener("change", (e) => {
    state.author = e.target.value.trim() || "Demo user";
    void apply("author");
  });
  $("doc").addEventListener("change", (e) => {
    if (e.target.value === "file") {
      // Back to the open document's entry, so choosing "Open your own PDF…" again works.
      e.target.value = state.doc;
      $("file").value = "";
      $("file").click();
      return;
    }
    state.doc = e.target.value;
    void openDocument({ url: state.doc });
  });
  $("file").addEventListener("change", (e) => {
    const file = e.target.files?.[0];
    if (file) void openDocument({ data: file, name: file.name });
    e.target.value = "";
  });
  $("unlock").addEventListener("click", async () => {
    try {
      const permissions = await viewer.unlock($("password").value);
      log("unlock", permissions);
      $("unlock-row").hidden = permissions.canAnnotate;
    } catch (e) {
      log("error", e instanceof FreePdfError ? { code: e.code, message: e.message } : String(e));
    }
  });
  for (const b of document.querySelectorAll("[data-action]")) b.addEventListener("click", () => void action(b.dataset.action));
  $("code-flavour").addEventListener("change", renderCode);
  for (const b of document.querySelectorAll(".bottom-tabs button[data-pane]")) {
    b.addEventListener("click", () => {
      for (const x of document.querySelectorAll(".bottom-tabs button[data-pane]")) x.setAttribute("aria-selected", String(x === b));
      for (const p of document.querySelectorAll(".bottom-body .pane")) p.hidden = p.dataset.pane !== b.dataset.pane;
      $("bottom").classList.remove("collapsed");
    });
  }
  $("collapse").addEventListener("click", () => $("bottom").classList.toggle("collapsed"));

  // Drag a PDF onto the viewer to open it.
  const wrap = $("viewer-wrap");
  wrap.addEventListener("dragover", (e) => {
    if ([...(e.dataTransfer?.items ?? [])].some((i) => i.kind === "file")) {
      e.preventDefault();
      wrap.classList.add("dragging");
    }
  });
  wrap.addEventListener("dragleave", () => wrap.classList.remove("dragging"));
  wrap.addEventListener("drop", (e) => {
    e.preventDefault();
    wrap.classList.remove("dragging");
    const file = e.dataTransfer?.files?.[0];
    if (file) void openDocument({ data: file, name: file.name });
  });

  renderFeatures();
  renderUi();
  renderCode();
}

async function start() {
  init();
  try {
    viewer = await FreePdf.init($("viewer"), {
      baseUrl: "/sdk/",
      mode: state.mode,
      theme: state.theme,
      author: state.author,
      document: { url: state.doc },
      listeners: {
        ready: (d) => log("ready", d),
        documentLoaded: (d) => log("documentLoaded", d),
        documentError: (d) => log("documentError", d),
        annotationsChanged: (d) => log("annotationsChanged", d),
        dirtyChanged: (d) => log("dirtyChanged", d),
        error: (d) => log("error", d),
        saveRequested: (d) => {
          log("saveRequested", d);
          download(d.name || "document.pdf", d.data, "application/pdf");
        },
      },
    });
    window.fpsDemo = viewer;
  } catch (e) {
    log("error", e instanceof FreePdfError ? { code: e.code, message: e.message } : String(e));
    $("loading").textContent = "The viewer could not start. It needs WebGL; try another browser.";
    $("loading").hidden = false;
  }
}

void start();
