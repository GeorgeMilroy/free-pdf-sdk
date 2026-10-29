// Renders the site's images into site/src/assets/img/: favicons and the share card from brand/,
// and the hero screenshot of the real viewer (annotate mode, sample comments). Needs the site
// built (node site/build.mjs) and served (node scripts/serve.mjs --root site/public).
//
//   node site/make-images.mjs [http://127.0.0.1:8780]

import { mkdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "@playwright/test";

const site = dirname(fileURLToPath(import.meta.url));
const repo = resolve(site, "..");
const out = join(site, "src/assets/img");
const base = process.argv[2] ?? "http://127.0.0.1:8780";
mkdirSync(out, { recursive: true });

const mark = readFileSync(join(repo, "brand/mark.svg"), "utf8");
const logoDark = readFileSync(join(repo, "brand/logo-dark.svg"), "utf8");
const browser = await chromium.launch({ args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"] });

async function render(html, width, height, file, transparent = false) {
  const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
  await page.setContent(`<!doctype html><html><body style="margin:0">${html}</body></html>`);
  await page.screenshot({ path: join(out, file), omitBackground: transparent });
  await page.close();
  console.log(file);
}

const icon = (size, pad) =>
  `<div style="width:${size}px;height:${size}px;display:grid;place-items:center;background:#fff;border-radius:${Math.round(size * 0.2)}px">${mark.replace(/width="100" height="100"/, `width="${size - pad * 2}" height="${size - pad * 2}"`)}</div>`;
await render(icon(32, 2), 32, 32, "favicon-32.png", true);
await render(icon(180, 22), 180, 180, "apple-touch-icon.png");

await render(
  `<div style="width:1200px;height:630px;box-sizing:border-box;padding:72px 80px;color:#fff;font-family:Inter,system-ui,sans-serif;background:radial-gradient(circle at 85% 15%,rgba(255,8,1,.55),transparent 55%),#080B0C;display:flex;flex-direction:column;justify-content:space-between">
    <div>${logoDark.replace(/width="349" height="100"/, 'width="520" height="149"')}</div>
    <div><div style="font-size:58px;font-weight:800;letter-spacing:-1.5px;line-height:1.05;max-width:980px">The embeddable PDF viewer and annotator for the web</div>
    <div style="margin-top:22px;font-size:26px;color:#c6ccd3">Free · Open source · Runs in the browser · React, Vue, Angular, Svelte</div></div>
  </div>`,
  1200,
  630,
  "og-image.png",
);

// Screenshots of the real viewer on the demo document, once per site theme (the demo's viewer
// follows the site's theme, which the site reads from localStorage):
//   viewer-layers: annotate mode with the Layers panel open (the hero);
//   viewer-full / viewer-view: the two ends of the mode comparison slider.
const shots = [
  { file: "viewer-layers", mode: "annotate", layers: true },
  { file: "viewer-full", mode: "full" },
  { file: "viewer-view", mode: "view" },
];
for (const theme of ["light", "dark"]) {
  for (const shot of shots) {
    const file = theme === "dark" ? `${shot.file}-dark.png` : `${shot.file}.png`;
    const page = await browser.newPage({ viewport: { width: 1430, height: 960 }, deviceScaleFactor: 1 });
    await page.emulateMedia({ colorScheme: theme });
    await page.addInitScript((t) => {
      localStorage.setItem("fps-theme", t);
      // A recorded choice keeps the consent banner (site/src/assets/js/consent.js) out of the
      // shot; declined, so nothing is sent to analytics. `v` follows consent.js's VERSION.
      localStorage.setItem("fps-consent", JSON.stringify({ v: 1, analytics: false, gpc: false, at: new Date().toISOString() }));
    }, theme);
    await page.goto(`${base}/demo/?mode=${shot.mode}`);
    await page.waitForFunction(() => window.fpsDemo !== undefined, undefined, { timeout: 120000 });
    await page.waitForTimeout(2000);
    await page.evaluate(() => window.fpsDemo.setZoom("fit-width"));
    await page.waitForTimeout(2000);
    if (shot.layers) {
      // The viewer is a canvas, so by position: close the Tools panel (its ×), open the Layers
      // panel from the right-hand rail, then hide the Furniture layer (its second row).
      const box = await page.locator("#viewer-wrap").boundingBox();
      await page.mouse.click(box.x + 248, box.y + 74);
      await page.waitForTimeout(600);
      await page.mouse.click(box.x + box.width - 23, box.y + 226);
      await page.waitForTimeout(900);
      await page.mouse.click(box.x + box.width - 310, box.y + 153);
      await page.waitForTimeout(1200);
    }
    await page.locator("#viewer-wrap").screenshot({ path: join(out, file) });
    await page.close();
    console.log(file);
  }
}
await browser.close();
