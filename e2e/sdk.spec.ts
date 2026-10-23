// End-to-end tests of the public API on examples/vanilla, in headless Chromium.
//
// The example page exposes `window.fps = { viewer, events }` once FreePdf.init resolves; the
// tests drive the viewer through that handle and check the events it fires.

import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const fixture = (name: string): string => readFileSync(fileURLToPath(new URL(`../tests/fixtures/${name}`, import.meta.url)), "utf8");

type Handle = { error?: { code: string; message: string } };

async function open(page: Page, query = ""): Promise<void> {
  await page.goto(`/${query}`);
  await page.waitForFunction(() => (window as unknown as { fps?: unknown }).fps !== undefined, undefined, { timeout: 120_000 });
  const err = await page.evaluate(() => (window as unknown as { fps: Handle }).fps.error);
  expect(err, "init succeeded").toBeUndefined();
}

/** Calls viewer[method](...args) in the page; returns { ok, value } or { ok: false, code }. */
async function call(page: Page, method: string, ...args: unknown[]): Promise<{ ok: boolean; value?: unknown; code?: string; message?: string }> {
  return page.evaluate(
    async ({ method, args }) => {
      const v = (window as unknown as { fps: { viewer: Record<string, (...a: unknown[]) => Promise<unknown>> } }).fps.viewer;
      try {
        const value = await v[method]!(...args);
        return { ok: true, value: value instanceof Uint8Array ? { bytes: value.byteLength, head: String.fromCharCode(...value.slice(0, 5)) } : value };
      } catch (e) {
        const err = e as { code?: string; message?: string };
        return { ok: false, code: err.code, message: err.message };
      }
    },
    { method, args },
  );
}

async function events(page: Page): Promise<Array<{ name: string; data: unknown }>> {
  return page.evaluate(() => (window as unknown as { fps: { events: Array<{ name: string; data: unknown }> } }).fps.events);
}

test("init, load, events and version", async ({ page }) => {
  await open(page);
  const names = (await events(page)).map((e) => e.name);
  expect(names).toContain("ready");
  expect(names).toContain("documentLoaded");
  const loaded = (await events(page)).find((e) => e.name === "documentLoaded")?.data as { pageCount: number; permissions: { canAnnotate: boolean } };
  expect(loaded.pageCount).toBe(3);
  expect(loaded.permissions.canAnnotate).toBe(true);
  expect((await call(page, "getPageCount")).value).toBe(3);
  const version = (await call(page, "version")).value as { sdk: string; pdfcraftRev: string };
  expect(version.sdk).toBe("0.2.2");
  expect(version.pdfcraftRev).toMatch(/^[0-9a-f]{40}$/);
  expect((await call(page, "goToPage", 2)).ok).toBe(true);
  expect((await call(page, "goToPage", 9)).code).toBe("BAD_REQUEST");
  expect((await call(page, "setZoom", 150)).ok).toBe(true);
  expect((await call(page, "setZoom", "fit-width")).ok).toBe(true);
  expect((await call(page, "setZoom", 5)).code).toBe("BAD_REQUEST");
  expect((await call(page, "isDirty")).value).toBe(false);
});

test("owner-protected AES-256: locked, unlock, import, export, list, delete, save", async ({ page }) => {
  await open(page, "?doc=protected-aes256-owner.pdf");
  let p = (await call(page, "getPermissions")).value as Record<string, boolean>;
  expect(p).toMatchObject({ canAnnotate: false, isEncrypted: true, canUnlock: true });

  const review = fixture("review.xfdf");
  expect((await call(page, "importXfdf", review)).code).toBe("NOT_PERMITTED");
  expect((await call(page, "unlock", "wrong")).code).toBe("WRONG_PASSWORD");
  p = (await call(page, "unlock", "fps-owner-secret")).value as Record<string, boolean>;
  expect(p).toMatchObject({ canAnnotate: true, isEncrypted: true, canUnlock: false });

  expect((await call(page, "importXfdf", review)).value).toBe(6);
  const list = (await call(page, "getAnnotations")).value as Array<{ id: string; author: string; pageIndex: number; replies: unknown[] }>;
  expect(list).toHaveLength(6);
  expect(list.map((a) => a.author).sort()).toEqual(["Daniel Ochieng", "Daniel Ochieng", "Priya Raman", "Priya Raman", "Priya Raman", "Priya Raman"]);
  expect(list.find((a) => a.id.endsWith("0006"))?.replies).toHaveLength(2);
  expect(list.map((a) => a.pageIndex).sort()).toEqual([0, 0, 0, 1, 1, 2]);

  const xfdf = (await call(page, "exportXfdf")).value as string;
  expect(xfdf).toContain('title="Priya Raman"');
  await expect.poll(async () => (await events(page)).some((e) => e.name === "annotationsChanged")).toBe(true);
  await expect.poll(async () => (await events(page)).some((e) => e.name === "dirtyChanged" && (e.data as { dirty: boolean }).dirty)).toBe(true);
  expect((await call(page, "isDirty")).value).toBe(true);

  const saved = (await call(page, "save")).value as { bytes: number; head: string };
  expect(saved.head).toBe("%PDF-");
  expect((await call(page, "isDirty")).value).toBe(false);

  expect((await call(page, "deleteAllAnnotations")).value).toBe(6);
  expect(((await call(page, "getAnnotations")).value as unknown[]).length).toBe(0);
});

test("XFDF fixtures import with exact counts; replace and empty XFDF", async ({ page }) => {
  await open(page);
  expect((await call(page, "importXfdf", fixture("markup.xfdf"))).value).toBe(4);
  expect((await call(page, "importXfdf", fixture("guest.xfdf"))).value).toBe(2);
  expect(((await call(page, "getAnnotations")).value as unknown[]).length).toBe(6);
  expect((await call(page, "importXfdf", fixture("review.xfdf"), { replace: true })).value).toBe(6);
  expect(((await call(page, "getAnnotations")).value as unknown[]).length).toBe(6);
  expect((await call(page, "importXfdf", fixture("empty.xfdf"))).value).toBe(0);
  expect((await call(page, "importXfdf", "<not xfdf")).code).toBe("XFDF_INVALID");
});

test("view mode is read-only through the API", async ({ page }) => {
  await open(page, "?mode=view");
  expect((await call(page, "importXfdf", fixture("review.xfdf"))).code).toBe("READ_ONLY");
  expect((await call(page, "deleteAllAnnotations")).code).toBe("READ_ONLY");
  expect((await call(page, "setMode", "annotate")).ok).toBe(true);
  expect((await call(page, "importXfdf", fixture("review.xfdf"))).value).toBe(6);
  expect((await call(page, "setMode", "view")).ok).toBe(true);
  expect((await call(page, "deleteAllAnnotations")).code).toBe("READ_ONLY");
  expect(((await call(page, "getAnnotations")).value as unknown[]).length).toBe(6);
});

test("view mode: shortcuts and clicks in the viewer change nothing", async ({ page }) => {
  await open(page);
  expect((await call(page, "importXfdf", fixture("review.xfdf"))).value).toBe(6);
  await call(page, "save"); // clean
  expect((await call(page, "setMode", "view")).ok).toBe(true);
  const frame = page.locator("iframe").first();
  const box = await frame.boundingBox();
  if (!box) throw new Error("no iframe");
  // Select a comment, then try to delete it, undo the import, and save.
  await page.mouse.click(box.x + box.width * 0.45, box.y + box.height * 0.45);
  for (const key of ["Delete", "Backspace", "Control+z", "Control+s", "Control+p", "n"]) await page.keyboard.press(key);
  await page.waitForTimeout(500);
  expect(((await call(page, "getAnnotations")).value as unknown[]).length).toBe(6);
  expect((await call(page, "isDirty")).value).toBe(false);
  expect((await events(page)).some((e) => e.name === "saveRequested")).toBe(false);
});

test("init with the owner password unlocks; a wrong one opens restricted and fires error", async ({ page }) => {
  await open(page, "?doc=protected-aes256-owner.pdf&password=fps-owner-secret");
  const loaded = (await events(page)).find((e) => e.name === "documentLoaded")?.data as { permissions: { canAnnotate: boolean } };
  expect(loaded.permissions.canAnnotate).toBe(true);
  expect(((await call(page, "getPermissions")).value as { canAnnotate: boolean }).canAnnotate).toBe(true);

  await open(page, "?doc=protected-aes256-owner.pdf&password=not-it");
  const err = (await events(page)).find((e) => e.name === "error")?.data as { code: string };
  expect(err.code).toBe("WRONG_PASSWORD");
  expect(((await call(page, "getPermissions")).value as { canAnnotate: boolean }).canAnnotate).toBe(false);
});

test("the viewer's own Save fires saveRequested with the PDF", async ({ page }) => {
  await open(page);
  expect((await call(page, "importXfdf", fixture("guest.xfdf"))).value).toBe(2);
  await expect.poll(async () => (await call(page, "isDirty")).value).toBe(true);
  const box = await page.locator("iframe").first().boundingBox();
  if (!box) throw new Error("no iframe");
  await page.mouse.click(box.x + box.width * 0.6, box.y + box.height * 0.85);
  await page.keyboard.press("Control+s");
  await expect.poll(async () => (await events(page)).find((e) => e.name === "saveRequested")?.data as { data?: number } | undefined).toBeTruthy();
  const saved = (await events(page)).find((e) => e.name === "saveRequested")?.data as { name: string; data: number };
  expect(saved.name).toBe("plain-3p.pdf");
  expect(saved.data).toBeGreaterThan(1000);
  await expect.poll(async () => (await call(page, "isDirty")).value).toBe(false);
});

test("a document that can't be fetched fires documentError", async ({ page }) => {
  await open(page, "?doc=none");
  const code = await page.evaluate(async () => {
    const v = (window as unknown as { fps: { viewer: { load: (d: unknown) => Promise<unknown> } } }).fps.viewer;
    try {
      await v.load({ url: "/fixtures/missing.pdf" });
      return "opened";
    } catch (e) {
      return (e as { code: string }).code;
    }
  });
  expect(code).toBe("FETCH_FAILED");
  const ev = (await events(page)).find((e) => e.name === "documentError")?.data as { code: string };
  expect(ev.code).toBe("FETCH_FAILED");
});

test("bad requests are typed errors", async ({ page }) => {
  await open(page);
  expect((await call(page, "setTheme", "blue")).code).toBe("BAD_REQUEST");
  expect((await call(page, "setMode", "draw")).code).toBe("BAD_REQUEST");
  expect((await call(page, "setAuthor", "   ")).code).toBe("BAD_REQUEST");
  expect((await call(page, "setDisabledElements", ["nope"])).code).toBe("BAD_REQUEST");
  expect((await call(page, "setDisabledElements", ["print", "tool:stamp"])).ok).toBe(true);
  expect((await call(page, "unlock", "x")).ok).toBe(true); // nothing to unlock: the permissions
  const raw = await page.evaluate(async () => {
    // A request with another protocol version, straight through postMessage.
    const v = (window as unknown as { fps: { viewer: { iframe: HTMLIFrameElement } } }).fps.viewer;
    const ch = new URL(v.iframe.src).hash.slice("#ch=".length);
    return new Promise((resolve) => {
      const onMessage = (e: MessageEvent): void => {
        if (e.data?.kind === "res" && e.data.id === 4242) {
          window.removeEventListener("message", onMessage);
          resolve(e.data.error?.code);
        }
      };
      window.addEventListener("message", onMessage);
      v.iframe.contentWindow?.postMessage({ fps: 99, ch, kind: "req", id: 4242, method: "version", params: {} }, location.origin);
    });
  });
  expect(raw).toBe("UNSUPPORTED_PROTOCOL");
});

test("documents that fail to open", async ({ page }) => {
  await open(page, "?doc=none");
  const bad = await page.evaluate(async () => {
    const v = (window as unknown as { fps: { viewer: { load: (d: unknown) => Promise<unknown> } } }).fps.viewer;
    try {
      await v.load({ data: new TextEncoder().encode("not a pdf at all"), name: "x.pdf" });
      return "opened";
    } catch (e) {
      return (e as { code: string }).code;
    }
  });
  expect(bad).toBe("LOAD_FAILED");
  expect((await events(page)).some((e) => e.name === "documentError")).toBe(true);
  expect((await call(page, "getPageCount")).code).toBe("NO_DOCUMENT");
});

test("two viewers on one page are isolated; destroy rejects pending calls", async ({ page }) => {
  await open(page);
  const result = await page.evaluate(async () => {
    const { FreePdf } = (window as unknown as { fps: { FreePdf: { init: (el: HTMLElement, o: unknown) => Promise<Record<string, (...a: unknown[]) => Promise<unknown>>> } } }).fps;
    const host = document.createElement("div");
    host.style.cssText = "position:fixed;right:0;bottom:0;width:400px;height:300px";
    document.body.appendChild(host);
    const second = await FreePdf.init(host, { baseUrl: "/sdk/", mode: "annotate", document: { url: "/fixtures/plain-3p.pdf" } });
    const first = (window as unknown as { fps: { viewer: Record<string, (...a: unknown[]) => Promise<unknown>> } }).fps.viewer;
    const xfdf = await (await fetch("/fixtures/guest.xfdf")).text();
    await second.importXfdf!(xfdf);
    const counts = [((await first.getAnnotations!()) as unknown[]).length, ((await second.getAnnotations!()) as unknown[]).length];
    const pending = second.getAnnotations!();
    await second.destroy!();
    let code = "";
    try {
      await pending;
    } catch (e) {
      code = (e as { code: string }).code;
    }
    let after = "";
    try {
      await second.version!();
    } catch (e) {
      after = (e as { code: string }).code;
    }
    return { counts, code, after, frames: document.querySelectorAll("iframe").length };
  });
  expect(result.counts).toEqual([0, 2]);
  expect(result.code).toBe("DESTROYED");
  expect(result.after).toBe("DESTROYED");
  expect(result.frames).toBe(1);
});

test("init rejects a cross-origin baseUrl", async ({ page }) => {
  await open(page, "?doc=none");
  const code = await page.evaluate(async () => {
    const { FreePdf } = (window as unknown as { fps: { FreePdf: { init: (el: HTMLElement, o: unknown) => Promise<unknown> } } }).fps;
    try {
      await FreePdf.init(document.body, { baseUrl: "https://example.com/sdk/" });
      return "ok";
    } catch (e) {
      return (e as { code: string }).code;
    }
  });
  expect(code).toBe("BAD_REQUEST");
});
