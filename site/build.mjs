// Builds the Free PDF SDK website into site/public (deployed with Firebase Hosting).
//
//   node site/build.mjs            (after scripts/build.ps1, which builds the SDK into dist/)
//
// Pages: the home page, the live demo, and the docs (site/src/docs/*.md) as HTML for people
// and as Markdown for AI agents (/docs/<slug>.md, /llms.txt, /llms-full.txt), with tables
// generated from docs/reference/config.json so they never drift from the code. The built SDK
// is copied to /sdk/ so the demo runs the real thing, from the same origin.

import { execFileSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import hljs from "highlight.js";
import { Marked } from "marked";

const site = dirname(fileURLToPath(import.meta.url));
const repo = resolve(site, "..");
const src = join(site, "src");
const out = join(site, "public");
const cfg = JSON.parse(readFileSync(join(site, "site.config.json"), "utf8"));
const version = JSON.parse(readFileSync(join(repo, "js/package.json"), "utf8")).version;
const reference = JSON.parse(readFileSync(join(repo, "docs/reference/config.json"), "utf8"));
const siteUrl = (process.env.SITE_URL ?? cfg.url).replace(/\/$/, "");
// Google Analytics loads only after the visitor consents (assets/js/consent.js); the privacy
// notice has to say who to contact, so the build refuses to run without it.
const analyticsId = cfg.analytics?.measurementId ?? "";
if (analyticsId && !/^G-[A-Z0-9]+$/.test(analyticsId)) throw new Error(`site.config.json: analytics.measurementId ${analyticsId} is not a GA4 id (G-...)`);
for (const k of ["controller", "email", "updated"]) {
  if (!cfg.legal?.[k]) throw new Error(`site.config.json: legal.${k} is required for the privacy notice`);
}
// The sitemap's <lastmod>: the date of the last commit touching these files, or nothing outside a
// git checkout. A shallow clone would give every page the same date.
const lastmod = (files) => {
  try {
    return execFileSync("git", ["log", "-1", "--format=%cs", "--", ...files], { cwd: repo, encoding: "utf8" }).trim() || null;
  } catch {
    return null;
  }
};

const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
// JSON-LD inside <script>: escape "<" so a "</script>" or "<!--" in a value cannot break out.
const ldJson = (data) => `<script type="application/ld+json">${JSON.stringify(data).replace(/</g, "\\u003c")}</script>`;
const slugify = (s) =>
  String(s)
    .toLowerCase()
    .replace(/<[^>]+>/g, "")
    .replace(/&[a-z]+;/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

// --- Markdown -------------------------------------------------------------------------------
function highlight(code, lang) {
  const language = { vue: "xml", svelte: "xml", html: "xml", tsx: "typescript", jsx: "javascript", sh: "bash", shell: "bash" }[lang] ?? lang;
  if (language && hljs.getLanguage(language)) return hljs.highlight(code, { language }).value;
  return esc(code);
}

function markdown(toc) {
  const md = new Marked({ gfm: true });
  md.use({
    renderer: {
      heading(text, depth, raw) {
        const id = slugify(raw);
        if (depth === 2 || depth === 3) toc.push({ depth, id, text: raw });
        if (depth === 1) return `<h1>${text}</h1>\n`;
        return `<h${depth} id="${id}"><a class="anchor" href="#${id}" aria-hidden="true">#</a>${text}</h${depth}>\n`;
      },
      code(code, lang) {
        const l = (lang ?? "").trim().split(/\s+/)[0];
        return `<div class="code"><div class="code-bar"><span>${esc(l || "text")}</span><button class="copy" type="button" aria-label="Copy code">Copy</button></div><pre><code class="hljs language-${esc(l)}">${highlight(code, l)}</code></pre></div>\n`;
      },
      table(header, body) {
        return `<div class="table-wrap"><table><thead>${header}</thead><tbody>${body}</tbody></table></div>\n`;
      },
      link(href, title, text) {
        const external = /^https?:\/\//.test(href ?? "");
        return `<a href="${esc(href)}"${title ? ` title="${esc(title)}"` : ""}${external ? ' rel="noopener" target="_blank"' : ""}>${text}</a>`;
      },
    },
  });
  return md;
}

// --- Reference tables (from docs/reference/config.json) ------------------------------------------
const cell = (s) => String(s).replace(/\|/g, "\\|");
const tables = {
  modes: () =>
    "| Mode | What it is for | Features on |\n|---|---|---|\n" +
    reference.modes.map((m) => `| \`${m.key}\` | ${cell(m.description)} | ${m.features.map((f) => `\`${f}\``).join(", ")} |`).join("\n"),
  features: () => {
    const cats = [...new Set(reference.features.map((f) => f.category))];
    return cats
      .map(
        (c) =>
          `**${c}**\n\n| Feature | What it controls | Changes the document | In modes |\n|---|---|---|---|\n` +
          reference.features
            .filter((f) => f.category === c)
            .map((f) => `| \`${f.key}\` | ${cell(f.description)} | ${f.edits ? "yes" : "no"} | ${f.modes.map((m) => `\`${m}\``).join(", ") || "none (opt in)"} |`)
            .join("\n"),
      )
      .join("\n\n");
  },
  ui: () => "| Option | Default | What it does |\n|---|---|---|\n" + reference.ui.map((u) => `| \`${u.key}\` | \`${u.default}\` | ${cell(u.description)} |`).join("\n"),
  panels: () => "| Panel | What it shows |\n|---|---|\n" + reference.panels.map((p) => `| \`${p.key}\` | ${cell(p.description)} |`).join("\n"),
  elements: () =>
    "| Element id | Hides |\n|---|---|\n" +
    reference.disabledElements.map((e) => `| \`${e.id}\` | ${e.hides.map((h) => `\`${h}\``).join(", ")} |`).join("\n"),
};
function expandReferences(text, file) {
  return text.replace(/^<!-- reference:([a-z]+) -->$/gm, (_, name) => {
    if (!tables[name]) throw new Error(`${file}: unknown reference table ${name}`);
    return tables[name]();
  });
}

// --- Docs -----------------------------------------------------------------------------------
function frontMatter(text, file) {
  const m = /^---\r?\n([\s\S]*?)\r?\n---\r?\n/.exec(text);
  if (!m) throw new Error(`${file}: missing front matter`);
  const meta = {};
  for (const line of m[1].split(/\r?\n/)) {
    const i = line.indexOf(":");
    if (i > 0) meta[line.slice(0, i).trim()] = line.slice(i + 1).trim().replace(/^"(.*)"$/, "$1");
  }
  for (const k of ["title", "description", "section", "order"]) if (!meta[k]) throw new Error(`${file}: front matter needs ${k}`);
  return { meta: { ...meta, order: Number(meta.order) }, body: text.slice(m[0].length) };
}

const SECTIONS = ["Get started", "Configure", "Frameworks", "Guides", "Reference"];

function loadDocs() {
  const docs = [];
  for (const file of readdirSync(join(src, "docs")).filter((f) => f.endsWith(".md") && f !== "README.md")) {
    const { meta, body } = frontMatter(readFileSync(join(src, "docs", file), "utf8"), file);
    if (!SECTIONS.includes(meta.section)) throw new Error(`${file}: unknown section ${meta.section}`);
    docs.push({ slug: file.replace(/\.md$/, ""), ...meta, body: expandReferences(body, file) });
  }
  // The changelog page comes from CHANGELOG.md.
  const changelog = readFileSync(join(repo, "CHANGELOG.md"), "utf8").replace(/^# Changelog\s*/, "");
  docs.push({ slug: "changelog", title: "Changelog", description: "What changed in each version of Free PDF SDK.", section: "Reference", order: 90, body: `# Changelog\n\n${changelog}` });
  docs.sort((a, b) => SECTIONS.indexOf(a.section) - SECTIONS.indexOf(b.section) || a.order - b.order);
  return docs;
}

// --- Layout ---------------------------------------------------------------------------------
const NAV = [
  ["Docs", "/docs/introduction/"],
  ["Configuration", "/docs/configuration/"],
  ["Frameworks", "/docs/react/"],
  ["API", "/docs/api/"],
  ["Live demo", "/demo/"],
];

function layout({ title, description, path, body, bodyClass = "", head = "", scripts = "", jsonLd }) {
  const full = title === cfg.name ? (cfg.homeTitle ?? `${cfg.name}: ${cfg.tagline}`) : `${title} | ${cfg.name}`;
  const exact = NAV.some(([, href]) => href === path);
  const nav = NAV.map(([label, href]) => {
    const current = path === href || (!exact && label === "Docs" && path.startsWith("/docs/"));
    return `<a href="${href}"${current ? ' aria-current="page"' : ""}>${label}</a>`;
  }).join("");
  const github = cfg.github ? `<a class="icon-link" href="${esc(cfg.github)}" rel="noopener" target="_blank" aria-label="Source code">GitHub</a>` : "";
  return `<!doctype html>
<html lang="en" data-theme="light">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(full)}</title>
<meta name="description" content="${esc(description)}">
<meta name="robots" content="max-image-preview:large">
<link rel="canonical" href="${esc(siteUrl + path)}">
<meta property="og:type" content="website">
<meta property="og:site_name" content="${esc(cfg.name)}">
<meta property="og:title" content="${esc(full)}">
<meta property="og:description" content="${esc(description)}">
<meta property="og:url" content="${esc(siteUrl + path)}">
<meta property="og:image" content="${esc(siteUrl)}/assets/img/og-image.png">
<meta name="twitter:card" content="summary_large_image">
<meta name="theme-color" content="#080B0C">
<link rel="icon" href="/assets/brand/mark.svg" type="image/svg+xml">
<link rel="icon" href="/assets/img/favicon-32.png" sizes="32x32" type="image/png">
<link rel="apple-touch-icon" href="/assets/img/apple-touch-icon.png">
<link rel="alternate" type="text/plain" title="Documentation for AI agents" href="/llms.txt">
${analyticsId ? `<meta name="fps-analytics" content="${esc(analyticsId)}">\n` : ""}<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap">
<link rel="stylesheet" href="/assets/css/site.css?v=${version}">
<script>try{var t=localStorage.getItem("fps-theme");if(t)document.documentElement.dataset.theme=t}catch(e){}</script>
${jsonLd ? ldJson(jsonLd) + "\n" : ""}${head}
</head>
<body class="${bodyClass}">
<a class="skip" href="#main">Skip to content</a>
<header class="site-header">
  <div class="container header-inner">
    <a class="brand" href="/" aria-label="${esc(cfg.name)} home">
      <img class="logo logo-light" src="/assets/brand/logo-light.svg" alt="${esc(cfg.name)}" width="140" height="40">
      <img class="logo logo-dark" src="/assets/brand/logo-dark.svg" alt="" width="140" height="40">
    </a>
    <nav class="main-nav" aria-label="Main">${nav}</nav>
    <div class="header-actions">
      <span class="version" title="Latest version">v${esc(version)}</span>
      ${github}
      <button class="theme-toggle" type="button" aria-label="Switch colour theme"><span class="sun">☀</span><span class="moon">☾</span></button>
      <button class="menu-toggle" type="button" aria-label="Menu" aria-expanded="false">☰</button>
    </div>
  </div>
</header>
<main id="main">
${body}
</main>
<footer class="site-footer">
  <div class="container footer-grid">
    <div class="footer-brand">
      <img class="logo logo-light" src="/assets/brand/logo-light.svg" alt="${esc(cfg.name)}" width="140" height="40">
      <img class="logo logo-dark" src="/assets/brand/logo-dark.svg" alt="" width="140" height="40">
      <p>${esc(cfg.tagline)}</p>
    </div>
    <div><h3>Product</h3><a href="/demo/">Live demo</a><a href="/docs/introduction/">Overview</a><a href="/docs/configuration/">Modes &amp; features</a><a href="/docs/changelog/">Changelog</a></div>
    <div><h3>Developers</h3><a href="/docs/getting-started/">Getting started</a><a href="/docs/api/">API reference</a><a href="/docs/react/">Frameworks</a><a href="/docs/ai-agents/">Docs for AI agents</a></div>
    <div><h3>Open source</h3><a href="/licenses/">Licences &amp; notices</a><a href="/llms.txt">llms.txt</a><a href="/privacy/">Privacy</a>${analyticsId ? `<button class="footer-link" type="button" data-consent-open hidden>Cookie settings</button>` : ""}</div>
  </div>
  <div class="container footer-bottom">
    <p>© ${new Date().getUTCFullYear()} ${esc(cfg.name)}. Released under MIT OR Apache-2.0.</p>
    <p class="based-on">Built on the open-source pdfcraft engine.</p>
  </div>
</footer>
<script type="module" src="/assets/js/site.js?v=${version}"></script>
${analyticsId ? `<script type="module" src="/assets/js/consent.js?v=${version}"></script>\n` : ""}${scripts}
</body>
</html>
`;
}

function docPage(doc, docs, index) {
  const toc = [];
  const html = markdown(toc).parse(doc.body);
  const sidebar = SECTIONS.map((section) => {
    const items = docs.filter((d) => d.section === section);
    if (!items.length) return "";
    return `<div class="side-section"><h4>${esc(section)}</h4>${items.map((d) => `<a href="/docs/${d.slug}/"${d.slug === doc.slug ? ' aria-current="page"' : ""}>${esc(d.title)}</a>`).join("")}</div>`;
  }).join("");
  const prev = docs[index - 1];
  const next = docs[index + 1];
  const onPage = toc.length
    ? `<nav class="on-page" aria-label="On this page"><h4>On this page</h4>${toc.map((t) => `<a class="d${t.depth}" href="#${t.id}">${esc(t.text.replace(/`/g, ""))}</a>`).join("")}</nav>`
    : "";
  const body = `<div class="container docs-layout">
  <aside class="docs-side" aria-label="Documentation"><div class="side-search"><input type="search" placeholder="Filter pages" aria-label="Filter pages"></div>${sidebar}</aside>
  <article class="prose">
    <div class="doc-meta"><span>${esc(doc.section)}</span><a href="/docs/${doc.slug}.md" title="This page as Markdown, for AI agents">View as Markdown</a></div>
    ${html}
    <nav class="pager">${prev ? `<a class="prev" href="/docs/${prev.slug}/"><small>Previous</small>${esc(prev.title)}</a>` : "<span></span>"}${next ? `<a class="next" href="/docs/${next.slug}/"><small>Next</small>${esc(next.title)}</a>` : ""}</nav>
  </article>
  ${onPage}
</div>`;
  return layout({ title: doc.title, description: doc.description, path: `/docs/${doc.slug}/`, body, bodyClass: "docs" });
}

// --- Machine-readable outputs ----------------------------------------------------------------------
function llmsTxt(docs) {
  const lines = [`# ${cfg.name}`, "", `> ${cfg.description}`, ""];
  lines.push(
    "Free PDF SDK is installed with `npm install @mecorelabs/free-pdf-sdk`, and its viewer files are copied into the host site with `npx @mecorelabs/free-pdf-sdk copy-viewer <folder>`; they must be served from the page's own origin. Configure what the viewer offers with `mode`, `features`, `ui` and `disabledElements`; every method is async and rejects with a typed `FreePdfError`.",
    "",
  );
  for (const section of SECTIONS) {
    const items = docs.filter((d) => d.section === section);
    if (!items.length) continue;
    lines.push(`## ${section}`, "");
    for (const d of items) lines.push(`- [${d.title}](${siteUrl}/docs/${d.slug}.md): ${d.description}`);
    lines.push("");
  }
  lines.push("## Optional", "", `- [All documentation in one file](${siteUrl}/llms-full.txt): every page above, concatenated.`, `- [Options JSON Schema](${siteUrl}/schema/viewer-options.schema.json): the shape of the options object.`, `- [Configuration reference](${siteUrl}/reference/config.json): modes, features, interface options and element ids as data.`, "");
  return lines.join("\n");
}

function optionsSchema() {
  const bool = (description) => ({ type: "boolean", description });
  return {
    $schema: "https://json-schema.org/draft/2020-12/schema",
    $id: `${siteUrl}/schema/viewer-options.schema.json`,
    title: "Free PDF SDK viewer options",
    description: "The options of FreePdf.init(element, options), without the callbacks (listeners).",
    type: "object",
    required: ["baseUrl"],
    additionalProperties: false,
    properties: {
      baseUrl: { type: "string", description: "URL of the folder holding viewer/ and version.json, on the page's own origin." },
      document: {
        description: "A PDF to open once ready: { url } (fetched by the page) or { data } (bytes, in JavaScript only).",
        type: "object",
        properties: { url: { type: "string" }, name: { type: "string" } },
      },
      mode: { enum: reference.modes.map((m) => m.key), default: "view", description: "A preset of features." },
      features: {
        type: "object",
        description: "Turn features on or off on top of the mode's preset.",
        additionalProperties: false,
        properties: Object.fromEntries(reference.features.map((f) => [f.key, bool(f.description)])),
      },
      ui: {
        type: "object",
        description: "Show or hide parts of the interface.",
        additionalProperties: false,
        properties: {
          ...Object.fromEntries(reference.ui.map((u) => [u.key, { ...bool(u.description), default: u.default }])),
          panels: { type: "object", additionalProperties: false, properties: Object.fromEntries(reference.panels.map((p) => [p.key, bool(p.description)])) },
        },
      },
      disabledElements: { type: "array", items: { enum: reference.disabledElements.map((e) => e.id) }, uniqueItems: true, maxItems: 256 },
      author: { type: "string", maxLength: 200, description: "Author of new comments." },
      theme: { enum: ["light", "dark", "system"], default: "system" },
      password: { type: "string", maxLength: 1024, description: "Opens a password-protected document and lifts owner restrictions." },
      locale: { type: "string", maxLength: 35, description: "Interface language, such as en, de or zh-Hans." },
      timeoutMs: { type: "integer", minimum: 1, default: 60000 },
    },
  };
}

// --- Build ----------------------------------------------------------------------------------
function write(path, text) {
  mkdirSync(dirname(join(out, path)), { recursive: true });
  writeFileSync(join(out, path), text);
}

function page(name) {
  return readFileSync(join(src, "pages", name), "utf8")
    .replaceAll("{{VERSION}}", version)
    .replace(/<code class="language-([a-z]+)">([\s\S]*?)<\/code>/g, (_, lang, code) => {
      const raw = code.replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&amp;/g, "&");
      return `<code class="hljs language-${lang}">${highlight(raw.replace(/^\n/, ""), lang)}</code>`;
    });
}

function main() {
  const sdk = join(repo, "dist", `free-pdf-sdk-${version}`);
  if (!existsSync(join(sdk, "viewer", "index.html"))) {
    throw new Error(`${sdk} is missing: build the SDK first (powershell -File scripts/build.ps1)`);
  }
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });

  // Assets, brand, samples, the SDK itself.
  cpSync(join(src, "assets"), join(out, "assets"), { recursive: true });
  for (const f of ["logo-light.svg", "logo-dark.svg", "mark.svg"]) cpSync(join(repo, "brand", f), join(out, "assets/brand", f));
  cpSync(sdk, join(out, "sdk"), { recursive: true });
  for (const f of ["demo.pdf", "protected-aes256-owner.pdf", "review.xfdf"]) cpSync(join(repo, "tests/fixtures", f), join(out, "samples", f));
  write("reference/config.json", JSON.stringify(reference, null, 2) + "\n");
  write("schema/viewer-options.schema.json", JSON.stringify(optionsSchema(), null, 2) + "\n");

  // Docs.
  const docs = loadDocs();
  docs.forEach((doc, i) => {
    write(`docs/${doc.slug}/index.html`, docPage(doc, docs, i));
    write(`docs/${doc.slug}.md`, `---\ntitle: ${doc.title}\ndescription: ${doc.description}\nurl: ${siteUrl}/docs/${doc.slug}/\n---\n\n${doc.body.trim()}\n`);
  });
  write("docs/index.html", `<!doctype html><meta charset="utf-8"><meta http-equiv="refresh" content="0; url=/docs/introduction/"><link rel="canonical" href="/docs/introduction/"><title>Docs</title>`);
  write("llms.txt", llmsTxt(docs));
  write(
    "llms-full.txt",
    `# ${cfg.name} documentation\n\n> ${cfg.description}\n\nSource: ${siteUrl}/llms.txt\n\n` + docs.map((d) => `<!-- ${siteUrl}/docs/${d.slug}/ -->\n\n${d.body.trim()}\n`).join("\n---\n\n"),
  );

  // Pages.
  // Raw text inside <script type="application/json">: only "<" needs escaping.
  const refJson = JSON.stringify({ modes: reference.modes, features: reference.features, ui: reference.ui, panels: reference.panels }).replace(/</g, "\\u003c");
  write(
    "index.html",
    layout({
      title: cfg.name,
      description: cfg.homeDescription ?? cfg.description,
      path: "/",
      // WebSite gives Google the site name for search results (on the home page only, with the
      // canonical home URL); Organization gives the publisher and its logo.
      jsonLd: {
        "@context": "https://schema.org",
        "@graph": [
          { "@type": "WebSite", "@id": `${siteUrl}/#website`, name: cfg.name, alternateName: "FreePDFSDK", url: `${siteUrl}/`, publisher: { "@id": `${siteUrl}/#organization` } },
          {
            "@type": "Organization",
            "@id": `${siteUrl}/#organization`,
            name: cfg.legal.controller,
            url: `${siteUrl}/`,
            logo: `${siteUrl}/assets/img/apple-touch-icon.png`,
            ...(cfg.github ? { sameAs: [cfg.github] } : {}),
          },
        ],
      },
      body: page("home.html").replace("{{REFERENCE_JSON}}", () => refJson),
      bodyClass: "home",
      head: `<link rel="stylesheet" href="/assets/css/home.css?v=${version}"><script>document.documentElement.classList.add("js")</script>`,
      // The Ko-fi donate button: on the home page only (not the docs, the demo or the SDK), drawn
      // once the page has loaded so it never holds the page up.
      scripts: `<script type="module" src="/assets/js/home.js?v=${version}"></script>
<script src="https://storage.ko-fi.com/cdn/scripts/overlay-widget.js" defer></script>
<script src="https://storage.ko-fi.com/cdn/widget/Widget_2.js" defer></script>
<script>
  addEventListener("load", () => {
    // The support section's button: Ko-fi's widget (its getHTML, as draw() uses document.write),
    // replacing the plain link it falls back to.
    const slot = document.getElementById("kofi-button");
    if (slot && window.kofiwidget2) {
      kofiwidget2.init("Donate on Ko-fi", "#d9534f", "K4V228W28L");
      slot.innerHTML = kofiwidget2.getHTML();
    }
    if (!window.kofiWidgetOverlay) return;
    kofiWidgetOverlay.draw("mecorelabs", {
      type: "floating-chat",
      "floating-chat.donateButton.text": "Donate",
      "floating-chat.donateButton.background-color": "#d9534f",
      "floating-chat.donateButton.text-color": "#fff",
    });
  });
</script>`,
    }),
  );
  write(
    "demo/index.html",
    layout({
      title: "Live demo",
      description: "Try Free PDF SDK in your browser: switch modes and features and watch the interface follow. Your files never leave your device.",
      path: "/demo/",
      body: page("demo.html").replace("{{REFERENCE_JSON}}", () => refJson),
      bodyClass: "demo-page",
      scripts: `<script type="module" src="/assets/js/demo.js?v=${version}"></script>`,
    }),
  );
  const licences = readFileSync(join(sdk, "third-party", "README.md"), "utf8");
  const licToc = [];
  write(
    "licenses/index.html",
    layout({
      title: "Licences and notices",
      description: "Free PDF SDK's licence and the open-source software it includes.",
      path: "/licenses/",
      body: `<div class="container narrow prose"><h1>Licences and notices</h1><p>Free PDF SDK is released under <strong>MIT OR Apache-2.0</strong>. It includes the open-source software below; the full licence texts ship with the SDK in <code>third-party/</code> and are served with the SDK: <a href="/sdk/NOTICE">NOTICE</a> and <a href="/sdk/third-party/README.md">third-party/README.md</a>.</p>${markdown(licToc).parse(licences.replace(/^# .*$/m, ""))}</div>`,
    }),
  );
  const privacy = readFileSync(join(src, "pages", "privacy.md"), "utf8").replace(/\{\{([A-Z_]+)\}\}/g, (m, key) => {
    const value = { SITE_NAME: cfg.name, SITE_URL: siteUrl, CONTROLLER: cfg.legal.controller, EMAIL: cfg.legal.email, UPDATED: cfg.legal.updated, MEASUREMENT_ID: analyticsId }[key];
    if (value === undefined) throw new Error(`privacy.md: unknown placeholder ${m}`);
    return value;
  });
  write(
    "privacy/index.html",
    layout({
      title: "Privacy notice",
      description: "What the Free PDF SDK website collects, why, and the choices you have.",
      path: "/privacy/",
      body: `<div class="container narrow prose">${markdown([]).parse(privacy)}<p><button class="btn" type="button" data-consent-open hidden>Cookie settings</button></p></div>`,
    }),
  );
  write("404.html", layout({ title: "Page not found", description: "This page does not exist.", path: "/404.html", body: `<div class="container narrow center-block"><p class="eyebrow">404</p><h1>Page not found</h1><p>The page you are looking for doesn't exist or has moved.</p><p><a class="btn primary" href="/">Home</a> <a class="btn" href="/docs/introduction/">Docs</a></p></div>` }));
  write("robots.txt", `User-agent: *\nAllow: /\nSitemap: ${siteUrl}/sitemap.xml\n`);
  // <lastmod>: the last commit to each page's content sources (not the templates, CSS or JS).
  const sources = {
    "/": ["site/src/pages/home.html", "site/site.config.json", "js/package.json"],
    "/demo/": ["site/src/pages/demo.html"],
    "/licenses/": ["third-party/README.md"],
    "/privacy/": ["site/src/pages/privacy.md", "site/site.config.json"],
    ...Object.fromEntries(docs.map((d) => [`/docs/${d.slug}/`, d.slug === "changelog" ? ["CHANGELOG.md"] : [`site/src/docs/${d.slug}.md`, "docs/reference/config.json"]])),
  };
  const sitemap = Object.entries(sources).map(([u, files]) => {
    const m = lastmod(files);
    return `  <url><loc>${esc(siteUrl + u)}</loc>${m ? `<lastmod>${m}</lastmod>` : ""}</url>`;
  });
  write("sitemap.xml", `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${sitemap.join("\n")}\n</urlset>\n`);

  checkLinks();
  console.log(`site built: ${out} (${docs.length} docs pages, SDK ${version})`);
}

/** Every internal link and asset in the built HTML points at a file that exists. */
function checkLinks() {
  const missing = [];
  const walk = (dir) => {
    for (const e of readdirSync(dir, { withFileTypes: true })) {
      const p = join(dir, e.name);
      if (e.isDirectory()) {
        if (p !== join(out, "sdk")) walk(p);
      } else if (e.name.endsWith(".html")) {
        const html = readFileSync(p, "utf8");
        for (const m of html.matchAll(/(?:href|src)="(\/[^"#?]*)/g)) {
          const url = m[1];
          const target = join(out, url);
          const ok = existsSync(target) && (!url.endsWith("/") || existsSync(join(target, "index.html")));
          if (!ok && !existsSync(target + "index.html")) missing.push(`${p.slice(out.length)} -> ${url}`);
        }
      }
    }
  };
  walk(out);
  if (missing.length) throw new Error(`broken links:\n${[...new Set(missing)].join("\n")}`);
}

main();
