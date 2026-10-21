/**
 * `<free-pdf-viewer>`: Free PDF SDK as a Web Component, for any framework that renders HTML
 * (Angular, Svelte, Solid, Lit, plain pages).
 *
 * ```html
 * <script type="module">
 *   import { defineFreePdfViewer } from "@mecorelabs/free-pdf-sdk/element";
 *   defineFreePdfViewer();
 * </script>
 * <free-pdf-viewer base-url="/free-pdf-sdk/" src="/files/contract.pdf" mode="annotate"
 *   features='{"stamps": false}' style="height: 80vh"></free-pdf-viewer>
 * ```
 *
 * Attributes: `base-url` (required), `src`, `mode`, `theme`, `author`, `locale`, `password`,
 * `features` (JSON), `ui` (JSON), `disabled-elements` (comma or space separated). The same
 * options are properties (`baseUrl`, `features`, `ui`...), and `options` sets several at once.
 * Events are DOM `CustomEvent`s with the data in `detail`: `fps-ready`, `fps-document-loaded`,
 * `fps-document-error`, `fps-annotations-changed`, `fps-dirty-changed`, `fps-error`,
 * `fps-save-requested`. `viewer` (once ready) and `whenReady()` give the full API.
 *
 * @packageDocumentation
 */

import { ELEMENT_EVENTS, FreePdfController, type ViewerOptions } from "./controller.js";
import type { ElementId, FeatureFlags, FreePdfViewer, Mode, Theme, UiOptions } from "./free-pdf-sdk.js";

export { ELEMENT_EVENTS };

const ATTRIBUTES = ["base-url", "src", "mode", "theme", "author", "locale", "password", "features", "ui", "disabled-elements", "loader"] as const;

function json<T>(text: string | null, attr: string, el: HTMLElement): T | undefined {
  if (text === null || text.trim() === "") return undefined;
  try {
    return JSON.parse(text) as T;
  } catch {
    el.dispatchEvent(new CustomEvent(ELEMENT_EVENTS.error, { detail: { code: "BAD_REQUEST", message: `the ${attr} attribute is not valid JSON` } }));
    return undefined;
  }
}

/** The `<free-pdf-viewer>` element. */
export class FreePdfViewerElement extends HTMLElement {
  static get observedAttributes(): readonly string[] {
    return ATTRIBUTES;
  }

  private controller: FreePdfController | null = null;
  private readonly container: HTMLDivElement;
  private props: Partial<ViewerOptions> = {};
  private scheduled = false;
  private readyWaiters: Array<{ resolve: (v: FreePdfViewer) => void; reject: (e: unknown) => void }> = [];

  constructor() {
    super();
    const root = this.attachShadow({ mode: "open" });
    const style = document.createElement("style");
    style.textContent = ":host{display:block;position:relative;min-height:200px}div{position:absolute;inset:0}";
    this.container = document.createElement("div");
    root.append(style, this.container);
  }

  /** The viewer's API once ready, else `null`. */
  get viewer(): FreePdfViewer | null {
    return this.controller?.current ?? null;
  }

  /** Resolves with the viewer's API once it is ready. */
  whenReady(): Promise<FreePdfViewer> {
    if (this.controller) return this.controller.ready;
    // Not mounted yet (it mounts a microtask after it is connected and has a base-url): wait.
    return new Promise((resolve, reject) => this.readyWaiters.push({ resolve, reject }));
  }

  private settleWaiters(): void {
    const c = this.controller;
    if (!c) return;
    for (const w of this.readyWaiters.splice(0)) c.ready.then(w.resolve, w.reject);
  }

  /** Several options at once; they win over attributes. */
  get options(): Partial<ViewerOptions> {
    return { ...this.props };
  }

  set options(value: Partial<ViewerOptions>) {
    this.props = { ...this.props, ...value };
    this.schedule();
  }

  get baseUrl(): string | undefined {
    return this.props.baseUrl ?? this.getAttribute("base-url") ?? undefined;
  }
  set baseUrl(v: string) {
    this.options = { baseUrl: v };
  }
  get mode(): Mode | undefined {
    return this.props.mode ?? ((this.getAttribute("mode") as Mode | null) ?? undefined);
  }
  set mode(v: Mode) {
    this.options = { mode: v };
  }
  get features(): FeatureFlags | undefined {
    return this.props.features ?? json<FeatureFlags>(this.getAttribute("features"), "features", this);
  }
  set features(v: FeatureFlags) {
    this.options = { features: v };
  }
  get ui(): UiOptions | undefined {
    return this.props.ui ?? json<UiOptions>(this.getAttribute("ui"), "ui", this);
  }
  set ui(v: UiOptions) {
    this.options = { ui: v };
  }
  get theme(): Theme | undefined {
    return this.props.theme ?? ((this.getAttribute("theme") as Theme | null) ?? undefined);
  }
  set theme(v: Theme) {
    this.options = { theme: v };
  }

  private resolved(): ViewerOptions | null {
    const baseUrl = this.baseUrl;
    if (!baseUrl) return null;
    const opts: ViewerOptions = { baseUrl };
    const src = this.getAttribute("src");
    if (src) opts.document = { url: src };
    const mode = this.mode;
    if (mode) opts.mode = mode;
    const theme = this.theme;
    if (theme) opts.theme = theme;
    const author = this.getAttribute("author");
    if (author) opts.author = author;
    const locale = this.getAttribute("locale");
    if (locale) opts.locale = locale;
    const password = this.getAttribute("password");
    if (password) opts.password = password;
    const features = this.features;
    if (features) opts.features = features;
    const ui = this.ui;
    if (ui) opts.ui = ui;
    const loader = this.getAttribute("loader");
    if (loader) opts.loader = loader === "none" ? false : loader;
    const disabled = this.getAttribute("disabled-elements");
    if (disabled) opts.disabledElements = disabled.split(/[\s,]+/).filter(Boolean) as ElementId[];
    return { ...opts, ...this.props } as ViewerOptions;
  }

  /** Batches attribute and property changes made in the same task. */
  private schedule(): void {
    if (this.scheduled) return;
    this.scheduled = true;
    queueMicrotask(() => {
      this.scheduled = false;
      this.sync();
    });
  }

  private sync(): void {
    if (!this.isConnected) return;
    const opts = this.resolved();
    if (!opts) return;
    if (!this.controller) {
      this.controller = new FreePdfController(this.container, opts, (name, data) => {
        this.dispatchEvent(new CustomEvent(ELEMENT_EVENTS[name], { detail: data, bubbles: true, composed: true }));
      });
      this.controller.ready.catch(() => undefined);
      this.settleWaiters();
    } else {
      this.controller.update(opts);
      this.settleWaiters();
    }
  }

  connectedCallback(): void {
    this.schedule();
  }

  disconnectedCallback(): void {
    const c = this.controller;
    this.controller = null;
    void c?.destroy();
  }

  attributeChangedCallback(): void {
    this.schedule();
  }
}

/**
 * Registers `<free-pdf-viewer>` (or `tag`). Safe to call more than once.
 *
 * @param tag - The custom element name.
 */
export function defineFreePdfViewer(tag = "free-pdf-viewer"): void {
  if (typeof customElements === "undefined" || customElements.get(tag)) return;
  // Each tag needs its own class.
  customElements.define(tag, class extends FreePdfViewerElement {});
}
