// Screenshots of the example in view and annotate mode, light and dark, after importing the
// ComPDF review fixture. Usage: node e2e/screenshots.mjs [outDir] (needs scripts/serve.mjs running on 8780).
import { chromium } from "@playwright/test";
import { mkdirSync } from "node:fs";

const out = process.argv[2] ?? "e2e/screenshots";
const base = process.argv[3] ?? "http://127.0.0.1:8780";
mkdirSync(out, { recursive: true });
const browser = await chromium.launch({ args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"] });
for (const theme of ["light", "dark"]) {
  for (const mode of ["annotate", "view"]) {
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    page.on("pageerror", (e) => console.log(`[${mode}/${theme}] pageerror ${e}`));
    await page.goto(`${base}/?mode=annotate&theme=${theme}${process.env.FPS_SHOT_QUERY ?? ""}`);
    await page.waitForFunction(() => window.fps !== undefined, undefined, { timeout: 120000 });
    await page.evaluate(async (mode) => {
      const v = window.fps.viewer;
      const xfdf = await (await fetch("/fixtures/compdf-review.xfdf")).text();
      await v.importXfdf(xfdf, { replace: true });
      await v.setZoom("fit-width");
      await v.setMode(mode);
    }, mode);
    await page.waitForTimeout(2500);
    await page.screenshot({ path: `${out}/${mode}-${theme}.png` });
    console.log(`${out}/${mode}-${theme}.png`);
    await page.close();
  }
}
await browser.close();
