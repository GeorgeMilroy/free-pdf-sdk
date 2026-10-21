/**
 * The shared engine of the framework adapters: mounts a viewer in a container, turns later
 * option changes into the matching API calls (in order), and tears it down. The Web Component,
 * React, Vue and Svelte integrations are thin layers over it.
 *
 * @packageDocumentation
 */

import {
  type DocumentSource,
  type EventMap,
  type EventName,
  type FreePdfViewer,
  type InitOptions,
  type Listener,
  FreePdfError,
  init,
} from "./free-pdf-sdk.js";

/** The DOM event name for each SDK event (`<free-pdf-viewer>` and the Svelte action). */
export const ELEMENT_EVENTS: Record<EventName, string> = {
  ready: "fps-ready",
  documentLoaded: "fps-document-loaded",
  documentError: "fps-document-error",
  annotationsChanged: "fps-annotations-changed",
  dirtyChanged: "fps-dirty-changed",
  error: "fps-error",
  saveRequested: "fps-save-requested",
};

/** Every event name, for adapters that forward them all. */
export const EVENT_NAMES: readonly EventName[] = ["ready", "documentLoaded", "documentError", "annotationsChanged", "dirtyChanged", "error", "saveRequested"];

/** What a controller mounts: the {@link InitOptions} without `listeners`. */
export type ViewerOptions = Omit<InitOptions, "listeners">;

/** Receives every event of the viewer. */
export type EventSink = <K extends EventName>(name: K, data: EventMap[K]) => void;

const same = (a: unknown, b: unknown): boolean => JSON.stringify(a ?? null) === JSON.stringify(b ?? null);

function sameDocument(a: DocumentSource | undefined, b: DocumentSource | undefined): boolean {
  if (!a || !b) return a === b;
  if ("url" in a && "url" in b) return a.url === b.url && a.name === b.name;
  if ("data" in a && "data" in b) return a.data === b.data && a.name === b.name;
  return false;
}

/** A viewer whose options can change after it mounted. */
export class FreePdfController {
  private viewer: FreePdfViewer | null = null;
  private options: ViewerOptions;
  private queue: Promise<unknown> = Promise.resolve();
  private generation = 0;
  private destroyed = false;
  /** The iframe of a mount still starting, removed at once if it is abandoned. */
  private starting: Element | null = null;
  /** Resolves with the viewer once it is ready and its first document has loaded or failed to (a failure is a `documentError` event). */
  ready: Promise<FreePdfViewer>;

  /**
   * @param container - The element the viewer fills; give it a size.
   * @param options - Initial options.
   * @param sink - Called with every event.
   */
  constructor(
    private readonly container: HTMLElement,
    options: ViewerOptions,
    private readonly sink: EventSink,
  ) {
    this.options = options;
    this.ready = this.mount();
  }

  /** The mounted viewer, once ready. */
  get current(): FreePdfViewer | null {
    return this.viewer;
  }

  private mount(): Promise<FreePdfViewer> {
    const generation = ++this.generation;
    const listeners: { [K in EventName]?: Listener<K> } = {};
    for (const name of EVENT_NAMES) {
      (listeners as Record<EventName, Listener<EventName>>)[name] = (data) => {
        if (generation === this.generation && !this.destroyed) this.sink(name, data);
      };
    }
    // The viewer starts without the document: a document that fails to open must not take the
    // viewer with it (init would remove it), so later options can still load another one.
    const { document: doc, password, ...config } = this.options;
    const started = init(this.container, { ...config, listeners });
    // init appends its iframe synchronously; remember it so an abandoned start can remove it.
    this.starting = this.container.lastElementChild;
    return started.then(
      async (viewer) => {
        this.starting = null;
        if (generation !== this.generation || this.destroyed) {
          await viewer.destroy();
          throw new FreePdfError("DESTROYED", "the viewer was replaced before it was ready");
        }
        this.viewer = viewer;
        if (doc) {
          // The viewer reports a failure itself (documentError); ready still resolves.
          await viewer.load(doc, password === undefined ? undefined : { password }).catch(() => undefined);
        }
        return viewer;
      },
      (e: unknown) => {
        this.starting = null;
        const err = e instanceof FreePdfError ? e : new FreePdfError("INTERNAL", String(e));
        if (generation === this.generation && !this.destroyed && err.code !== "DESTROYED") this.sink("error", { code: err.code, message: err.message });
        throw err;
      },
    );
  }

  /**
   * Applies new options: what changed becomes API calls (`setMode`, `setFeatures`, `setUi`,
   * `setTheme`, `setAuthor`, `setLocale`, `setDisabledElements`, `load`); a new `baseUrl`
   * mounts a new viewer. Failures are reported as `error` events.
   */
  update(next: ViewerOptions): void {
    const prev = this.options;
    this.options = next;
    if (this.destroyed) return;
    if (next.baseUrl !== prev.baseUrl) {
      const old = this.viewer;
      this.viewer = null;
      this.queue = Promise.resolve();
      this.starting?.remove();
      this.starting = null;
      void old?.destroy();
      this.ready = this.mount();
      this.ready.catch(() => undefined);
      return;
    }
    const calls: Array<(v: FreePdfViewer) => Promise<unknown>> = [];
    if (next.mode !== undefined && next.mode !== prev.mode) calls.push((v) => v.setMode(next.mode ?? "view"));
    if (!same(next.features, prev.features)) calls.push((v) => v.setFeatures(next.features ?? {}));
    if (!same(next.ui, prev.ui) && next.ui) calls.push((v) => v.setUi(next.ui ?? {}));
    if (!same(next.disabledElements, prev.disabledElements)) calls.push((v) => v.setDisabledElements(next.disabledElements ?? []));
    if (next.theme !== undefined && next.theme !== prev.theme) calls.push((v) => v.setTheme(next.theme ?? "system"));
    if (next.author !== undefined && next.author !== prev.author) calls.push((v) => v.setAuthor(next.author ?? ""));
    if (next.locale !== undefined && next.locale !== prev.locale) calls.push((v) => v.setLocale(next.locale ?? "en"));
    if (next.document && !sameDocument(next.document, prev.document)) {
      const doc = next.document;
      calls.push((v) => v.load(doc, next.password === undefined ? undefined : { password: next.password }));
    }
    if (calls.length === 0) return;
    const generation = this.generation;
    this.queue = this.queue
      .then(() => this.ready)
      .then(async (v) => {
        // Each call on its own: one refused call (a bad locale) must not drop the rest (the load).
        for (const call of calls) {
          if (generation !== this.generation || this.destroyed) return;
          try {
            await call(v);
          } catch (e) {
            if (generation !== this.generation || this.destroyed) return;
            const err = e instanceof FreePdfError ? e : new FreePdfError("INTERNAL", String(e));
            if (err.code !== "DESTROYED") this.sink("error", { code: err.code, message: err.message });
          }
        }
      })
      .catch(() => undefined);
  }

  /** Removes the viewer. Safe to call twice. */
  async destroy(): Promise<void> {
    if (this.destroyed) return;
    this.destroyed = true;
    this.generation++;
    const v = this.viewer;
    this.viewer = null;
    // A start still under way: take its iframe away now (it is destroyed when init settles).
    this.starting?.remove();
    this.starting = null;
    if (v) await v.destroy();
    else await this.ready.then((late) => late.destroy()).catch(() => undefined);
  }
}
