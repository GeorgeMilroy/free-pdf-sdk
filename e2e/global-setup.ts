// Bundles the framework integration test pages into e2e/.out (served at /e2e-pages/).
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const here = dirname(fileURLToPath(import.meta.url));

export default async function globalSetup(): Promise<void> {
  const require = createRequire(join(here, "../js/package.json"));
  const esbuild = require("esbuild") as typeof import("esbuild");
  const out = join(here, ".out");
  mkdirSync(out, { recursive: true });
  for (const name of ["react", "vue"]) {
    await esbuild.build({
      entryPoints: [join(here, "adapters", `${name}-app.ts`)],
      bundle: true,
      format: "esm",
      outfile: join(out, `${name}.js`),
      nodePaths: [join(here, "../js/node_modules")],
      define: { "process.env.NODE_ENV": '"production"', __VUE_OPTIONS_API__: "true", __VUE_PROD_DEVTOOLS__: "false", __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "false" },
      logLevel: "warning",
    });
    writeFileSync(join(out, `${name}.html`), `<!doctype html><meta charset="utf-8"><title>${name}</title><div id="app"></div><script type="module" src="/e2e-pages/${name}.js"></script>\n`);
  }
  writeFileSync(
    join(out, "element.html"),
    `<!doctype html><meta charset="utf-8"><title>element</title>
<script type="module">import { defineFreePdfViewer } from "/sdk/element.js"; defineFreePdfViewer(); window.fpsEvents = [];
for (const n of ["fps-ready", "fps-document-loaded", "fps-annotations-changed"]) document.addEventListener(n, (e) => window.fpsEvents.push(n + ":" + JSON.stringify(e.detail)));</script>
<free-pdf-viewer id="v" base-url="/sdk/" src="/fixtures/plain-3p.pdf" mode="annotate" features='{"stamps": false}' style="height:600px"></free-pdf-viewer>\n`,
  );
  writeFileSync(
    join(out, "svelte.html"),
    `<!doctype html><meta charset="utf-8"><title>svelte action</title><div id="node" style="height:600px"></div>
<script type="module">import { freePdf } from "/sdk/svelte.js"; window.fpsEvents = [];
const node = document.getElementById("node");
node.addEventListener("fps-document-loaded", () => window.fpsEvents.push("documentLoaded"));
window.fpsAction = freePdf(node, { baseUrl: "/sdk/", mode: "review", document: { url: "/fixtures/plain-3p.pdf" }, onViewer: (v) => (window.fpsViewer = v) });</script>\n`,
  );
}
