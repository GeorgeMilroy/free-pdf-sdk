// Modes, features and interface options through the API, and every framework integration.
import { expect, test, type Page } from "@playwright/test";

type Api = {
  getConfig(): Promise<{ mode: string; features: Record<string, boolean>; ui: Record<string, unknown>; readOnly: boolean }>;
  setMode(m: string): Promise<void>;
  setFeatures(f: Record<string, boolean>): Promise<void>;
  setUi(u: Record<string, unknown>): Promise<void>;
  importXfdf(x: string, o?: { replace?: boolean }): Promise<number>;
  deleteAllAnnotations(): Promise<number>;
};

async function open(page: Page, query = ""): Promise<void> {
  await page.goto(`/${query}`);
  await page.waitForFunction(() => (window as unknown as { fps?: unknown }).fps !== undefined, undefined, { timeout: 120_000 });
}

test("modes are feature presets; overrides and ui options round-trip through getConfig", async ({ page }) => {
  await open(page, "?mode=annotate");
  const result = await page.evaluate(async () => {
    const v = (window as unknown as { fps: { viewer: Api } }).fps.viewer;
    const annotate = await v.getConfig();
    await v.setFeatures({ stamps: false, pages: true });
    const tuned = await v.getConfig();
    await v.setUi({ quickToolbar: false, panels: { layers: false } });
    const ui = await v.getConfig();
    await v.setMode("view");
    const view = await v.getConfig();
    await v.setFeatures({});
    await v.setMode("organize");
    const organize = await v.getConfig();
    return { annotate, tuned, ui, view, organize };
  });
  expect(result.annotate.mode).toBe("annotate");
  expect(result.annotate.features.comments && result.annotate.features.shapes && result.annotate.features.fillSign).toBe(true);
  expect(result.annotate.features.pages || result.annotate.features.editContent || result.annotate.features.formFill).toBe(false);
  expect(result.tuned.features.stamps).toBe(false);
  expect(result.tuned.features.pages).toBe(true);
  expect(result.ui.ui.quickToolbar).toBe(false);
  expect((result.ui.ui.panels as Record<string, boolean>).layers).toBe(false);
  // Overrides survive a mode change: view + pages override is not read-only.
  expect(result.view.mode).toBe("view");
  expect(result.view.features.pages).toBe(true);
  expect(result.organize.features.pages && !result.organize.features.comments).toBe(true);
  expect(result.organize.readOnly).toBe(false);
});

test("annotation calls are refused without an annotation feature, allowed with one", async ({ page }) => {
  await open(page, "?mode=organize");
  const codes = await page.evaluate(async () => {
    const v = (window as unknown as { fps: { viewer: Api } }).fps.viewer;
    const xfdf = await (await fetch("/fixtures/review.xfdf")).text();
    const tryImport = () => v.importXfdf(xfdf).then((n) => `ok:${n}`, (e: { code: string }) => e.code);
    const organize = await tryImport();
    await v.setFeatures({ comments: true });
    const withComments = await tryImport();
    await v.setFeatures({});
    await v.setMode("view");
    const view = await v.deleteAllAnnotations().then(() => "ok", (e: { code: string }) => e.code);
    return { organize, withComments, view };
  });
  expect(codes.organize).toBe("READ_ONLY");
  expect(codes.withComments).toMatch(/^ok:\d+$/);
  expect(codes.view).toBe("READ_ONLY");
});

test("import needs comments; delete-all is refused up front when a comment's kind is off, deleting nothing", async ({ page }) => {
  await open(page, "?mode=annotate");
  const r = await page.evaluate(async () => {
    const v = (window as unknown as { fps: { viewer: Api & { getAnnotations(): Promise<unknown[]> } } }).fps.viewer;
    const xfdf = await (await fetch("/fixtures/review.xfdf")).text();
    await v.importXfdf(xfdf, { replace: true });
    const before = (await v.getAnnotations()).length;
    await v.setMode("fill");
    const fillImport = await v.importXfdf(xfdf).then(() => "ok", (e: { code: string }) => e.code);
    // Review allows comments and markup, not the shapes and ink the fixture holds.
    await v.setMode("review");
    const reviewDelete = await v.deleteAllAnnotations().then(() => "ok", (e: { code: string }) => e.code);
    const replace = await v.importXfdf(xfdf, { replace: true }).then(() => "ok", (e: { code: string }) => e.code);
    const after = (await v.getAnnotations()).length;
    return { before, fillImport, reviewDelete, replace, after };
  });
  expect(r.fillImport).toBe("READ_ONLY");
  expect(r.reviewDelete).toBe("READ_ONLY");
  expect(r.replace).toBe("READ_ONLY");
  expect(r.after).toBe(r.before);
});

test("unknown features, ui options and panels are typed errors", async ({ page }) => {
  await open(page, "?mode=annotate");
  const codes = await page.evaluate(async () => {
    const v = (window as unknown as { fps: { viewer: Api } }).fps.viewer;
    const code = (p: Promise<unknown>) => p.then(() => "ok", (e: { code: string }) => e.code);
    return [
      await code(v.setFeatures({ teleport: true })),
      await code(v.setUi({ wings: false })),
      await code(v.setUi({ panels: { spaceship: false } })),
      await code(v.setMode("draw")),
      await code(v.setFeatures({ comments: true })),
    ];
  });
  expect(codes).toEqual(["BAD_REQUEST", "BAD_REQUEST", "BAD_REQUEST", "BAD_REQUEST", "ok"]);
});

test("Web Component: attributes configure it, events are DOM events, the API is reachable", async ({ page }) => {
  await page.goto("/e2e-pages/element.html");
  await page.waitForFunction(() => (window as unknown as { fpsEvents: string[] }).fpsEvents.some((e) => e.startsWith("fps-document-loaded")), undefined, { timeout: 120_000 });
  const before = await page.evaluate(async () => {
    const el = document.getElementById("v") as HTMLElement & { whenReady(): Promise<Api> };
    return (await el.whenReady()).getConfig();
  });
  expect(before.mode).toBe("annotate");
  expect(before.features.stamps).toBe(false);
  await page.evaluate(() => document.getElementById("v")?.setAttribute("mode", "fill"));
  await expect
    .poll(async () => page.evaluate(async () => ((await (document.getElementById("v") as HTMLElement & { whenReady(): Promise<Api> }).whenReady()).getConfig()).then((c) => c.mode)))
    .toBe("fill");
  const events = await page.evaluate(() => (window as unknown as { fpsEvents: string[] }).fpsEvents);
  expect(events.some((e) => e.startsWith("fps-ready"))).toBe(true);
});

test("React: props configure it, prop changes call the API, handlers receive events", async ({ page }) => {
  await page.goto("/e2e-pages/react.html");
  await page.waitForFunction(() => (window as unknown as { fpsTest: { events: string[] } }).fpsTest.events.includes("documentLoaded"), undefined, { timeout: 120_000 });
  const config = () => page.evaluate(async () => ((await (window as unknown as { fpsTest: { handle: { whenReady(): Promise<Api> } } }).fpsTest.handle.whenReady()).getConfig()));
  expect((await config()).mode).toBe("annotate");
  await page.evaluate(() => (window as unknown as { fpsTest: { setMode(m: string): void; setStamps(on: boolean): void } }).fpsTest.setMode("review"));
  await expect.poll(async () => (await config()).mode).toBe("review");
  await page.evaluate(() => (window as unknown as { fpsTest: { setStamps(on: boolean): void } }).fpsTest.setStamps(false));
  await expect.poll(async () => (await config()).features.stamps).toBe(false);
  await page.evaluate(async () => {
    const v = await (window as unknown as { fpsTest: { handle: { whenReady(): Promise<Api> } } }).fpsTest.handle.whenReady();
    await v.importXfdf(await (await fetch("/fixtures/review.xfdf")).text());
  });
  await expect.poll(async () => page.evaluate(() => (window as unknown as { fpsTest: { events: string[] } }).fpsTest.events.some((e) => e.startsWith("annotationsChanged:")))).toBe(true);
});

test("Vue: props configure it, reactive changes call the API, emits reach listeners", async ({ page }) => {
  await page.goto("/e2e-pages/vue.html");
  await page.waitForFunction(() => (window as unknown as { fpsTest: { events: string[] } }).fpsTest.events.includes("documentLoaded"), undefined, { timeout: 120_000 });
  const config = () => page.evaluate(async () => ((await (window as unknown as { fpsTest: { viewerRef: { value: { whenReady(): Promise<Api> } } } }).fpsTest.viewerRef.value.whenReady()).getConfig()));
  expect((await config()).mode).toBe("annotate");
  await page.evaluate(() => {
    const t = (window as unknown as { fpsTest: { mode: { value: string }; stamps: { value: boolean } } }).fpsTest;
    t.mode.value = "sign";
    t.stamps.value = false;
  });
  await expect.poll(async () => (await config()).mode).toBe("sign");
  await expect.poll(async () => (await config()).features.stamps).toBe(false);
});

test("Svelte action: mounts, updates and destroys", async ({ page }) => {
  await page.goto("/e2e-pages/svelte.html");
  await page.waitForFunction(() => (window as unknown as { fpsEvents: string[] }).fpsEvents.includes("documentLoaded"), undefined, { timeout: 120_000 });
  await page.evaluate(() => (window as unknown as { fpsAction: { update(o: unknown): void } }).fpsAction.update({ baseUrl: "/sdk/", mode: "edit", document: { url: "/fixtures/plain-3p.pdf" } }));
  await expect.poll(async () => page.evaluate(async () => (await (window as unknown as { fpsViewer: Api }).fpsViewer.getConfig()).mode)).toBe("edit");
  await page.evaluate(() => (window as unknown as { fpsAction: { destroy(): void } }).fpsAction.destroy());
  await expect.poll(async () => page.locator("#node iframe").count()).toBe(0);
});

test("loader: shown while the viewer starts and while a document opens, then hidden; configurable", async ({ page }) => {
  await open(page);
  const r = await page.evaluate(async () => {
    type V = { load(s: { url: string }): Promise<unknown>; destroy(): Promise<void> };
    const { FreePdf } = (window as unknown as { fps: { FreePdf: { init(el: HTMLElement, o: Record<string, unknown>): Promise<V> } } }).fps;
    const mount = (loader: unknown) => {
      const el = document.createElement("div");
      el.style.cssText = "width:400px;height:300px";
      document.body.append(el);
      const opts: Record<string, unknown> = { baseUrl: "/sdk/" };
      if (loader !== undefined) opts.loader = loader;
      return { el, started: FreePdf.init(el, opts) };
    };
    const overlay = (el: HTMLElement) => el.querySelector<HTMLElement>("[data-fps-loader]");
    const shown = (el: HTMLElement) => {
      const o = overlay(el);
      return o !== null && !o.hidden && o.style.opacity !== "0";
    };
    const wait = (ms: number) => new Promise((res) => setTimeout(res, ms));

    const svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><circle cx="5" cy="5" r="4"/></svg>';
    const custom = mount({ svg, size: "96px", background: "rgb(1, 2, 3)", label: "Opening" });
    const img = overlay(custom.el)?.querySelector("img");
    const atStart = {
      shown: shown(custom.el),
      src: img?.src.startsWith("data:image/svg+xml") ?? false,
      alt: img?.alt,
      width: img?.style.width,
      background: overlay(custom.el)?.style.background,
    };
    const v = await custom.started;
    await wait(300);
    const afterReady = shown(custom.el);
    const loading = v.load({ url: "/fixtures/demo.pdf" });
    const duringLoad = shown(custom.el);
    await loading;
    await wait(300);
    const afterLoad = shown(custom.el);
    await v.destroy();

    const gif = mount("/assets/spinner.gif");
    const gifSrc = overlay(gif.el)?.querySelector("img")?.getAttribute("src");
    const none = mount(false);
    const noneOverlay = overlay(none.el) === null;
    const builtIn = mount(undefined);
    const builtInShown = shown(builtIn.el);
    const bad = await FreePdf.init(document.createElement("div"), { baseUrl: "/sdk/", loader: { size: 3 } }).then(() => "ok", (e: { code: string }) => e.code);
    for (const m of [gif, none, builtIn]) await m.started.then((x) => x.destroy(), () => undefined);
    return { atStart, afterReady, duringLoad, afterLoad, gifSrc, noneOverlay, builtInShown, bad };
  });
  expect(r.atStart).toEqual({ shown: true, src: true, alt: "Opening", width: "96px", background: "rgb(1, 2, 3)" });
  expect(r.afterReady).toBe(false);
  expect(r.duringLoad).toBe(true);
  expect(r.afterLoad).toBe(false);
  expect(r.gifSrc).toBe("/assets/spinner.gif");
  expect(r.noneOverlay).toBe(true);
  expect(r.builtInShown).toBe(true);
  expect(r.bad).toBe("BAD_REQUEST");
});
