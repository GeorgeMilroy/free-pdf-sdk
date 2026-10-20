/**
 * Free PDF SDK: an embeddable PDF viewer and annotator for the web, based on pdfcraft.
 *
 * ```ts
 * import { FreePdf } from "free-pdf-sdk";
 * const viewer = await FreePdf.init(document.getElementById("viewer")!, {
 *   baseUrl: "/free-pdf-sdk/",
 *   mode: "annotate",
 *   theme: "system",
 *   document: { url: "/files/artwork.pdf" },
 * });
 * viewer.on("annotationsChanged", ({ count }) => console.log(count));
 * ```
 *
 * Each viewer is an iframe on the host's own origin (the SDK folder must be served from the same
 * origin as the page). The wrapper talks to it over a versioned postMessage protocol; see
 * docs/api.md and docs/architecture.md.
 *
 * @packageDocumentation
 */

/** The SDK version this wrapper belongs to. */
export const SDK_VERSION = "0.1.0";

/** The postMessage protocol version this wrapper speaks. */
export const PROTOCOL_VERSION = 1;

/** Viewer mode: `view` is read-only; `annotate` allows comments, shapes, ink, Fill & Sign and measurement. */
export type Mode = "view" | "annotate";

/** Colour theme; `system` follows the browser's preference. */
export type Theme = "light" | "dark" | "system";

/** Zoom: a percentage (10–6400) or a fit. */
export type Zoom = number | "fit-width" | "fit-page";

/**
 * Ids accepted by `disabledElements` and {@link FreePdfViewer.setDisabledElements}.
 * docs/api.md lists what each one hides.
 */
export type ElementId =
  | "download"
  | "save"
  | "print"
  | "open"
  | "flatten"
  | "settings"
  | "import"
  | "tool:note"
  | "tool:freetext"
  | "tool:callout"
  | "tool:highlight"
  | "tool:underline"
  | "tool:strikeout"
  | "tool:squiggly"
  | "tool:caret"
  | "tool:replace"
  | "tool:ink"
  | "tool:line"
  | "tool:arrow"
  | "tool:square"
  | "tool:circle"
  | "tool:polygon"
  | "tool:polyline"
  | "tool:cloud"
  | "tool:attach"
  | "tool:stamp"
  | "tool:image"
  | "tool:link"
  | "tool:measure"
  | "tool:fillsign"
  | "tool:signature";

/** A PDF to open: fetched from a URL, or given as bytes. */
export type DocumentSource =
  | {
      /** URL of the PDF (same-origin, or CORS-enabled such as a Firebase Storage download URL). */
      url: string;
      /** File name shown in the viewer; defaults to the URL's last path segment. */
      name?: string;
      /** Options for `fetch` (headers, credentials). */
      fetchInit?: RequestInit;
    }
  | {
      /** The PDF's bytes. They are copied, never detached from the caller. */
      data: ArrayBuffer | Uint8Array | Blob;
      /** File name shown in the viewer; defaults to the Blob's name or `document.pdf`. */
      name?: string;
    };

/** Options for {@link FreePdf.init}. */
export interface InitOptions {
  /** URL of the SDK folder (the one holding `viewer/` and `version.json`), on the page's origin. */
  baseUrl: string;
  /** A document to open once the viewer is ready. */
  document?: DocumentSource;
  /** `view` (read-only, the default) or `annotate`. */
  mode?: Mode;
  /** Author of new comments; it wins over a name saved in the viewer's settings. */
  author?: string;
  /** Colour theme (default `system`). */
  theme?: Theme;
  /** Password: opens a document that needs one, and lifts owner restrictions (as {@link FreePdfViewer.unlock}). */
  password?: string;
  /** Parts of the interface to hide. */
  disabledElements?: ElementId[];
  /** Interface language, such as `en`, `de` or `zh-Hans` (closest available; English otherwise). */
  locale?: string;
  /** Milliseconds to wait for a reply before rejecting with `TIMEOUT` (default 60000; loads get twice that). */
  timeoutMs?: number;
  /**
   * Listeners attached before the viewer starts, so they also hear events fired while `init`
   * is still running (`ready`, the first `documentLoaded`). `ready` also reaches listeners
   * added later with `on`.
   */
  listeners?: { [K in EventName]?: Listener<K> };
}

/** What the open document's security allows. */
export interface Permissions {
  /** Comments may be added, changed, deleted and imported. */
  canAnnotate: boolean;
  /** The document may be printed. */
  canPrint: boolean;
  /** Text and images may be copied. */
  canCopy: boolean;
  /** The document is encrypted. */
  isEncrypted: boolean;
  /** An owner password would lift restrictions ({@link FreePdfViewer.unlock}). */
  canUnlock: boolean;
}

/** A reply in a comment thread. */
export interface AnnotationReply {
  /** The reply's id (XFDF `name`). */
  id: string;
  /** Author (XFDF `title`). */
  author: string | null;
  /** Text. */
  contents: string | null;
  /** Last modified, ISO 8601. */
  date: string | null;
}

/** A comment, as {@link FreePdfViewer.getAnnotations} lists it. */
export interface Annotation {
  /** The annotation's id (XFDF `name`). */
  id: string;
  /** Lowercase subtype as ComPDF names it: `text` (note), `freetext`, `square`, `circle`, `line`, `ink`, `polygon`, `polyline`, `highlight`, `underline`, `squiggly`, `strikeout`, `stamp`, `caret`, `fileattachment`, `sound`, `redact`. */
  type: string;
  /** Author (XFDF `title`). */
  author: string | null;
  /** Subject. */
  subject: string | null;
  /** Text. */
  contents: string | null;
  /** Last modified (else created), ISO 8601. */
  date: string | null;
  /** 0-based page index. */
  pageIndex: number;
  /** Colour, `#RRGGBB`. */
  color: string | null;
  /** Replies, oldest first. */
  replies: AnnotationReply[];
}

/** A newly opened document. */
export interface DocumentInfo {
  /** Number of pages. */
  pageCount: number;
  /** What its security allows. */
  permissions: Permissions;
}

/** The result of {@link FreePdfViewer.load}. */
export interface LoadResult extends DocumentInfo {
  /** The `password` lifted owner restrictions. */
  unlocked: boolean;
}

/** Versions of the SDK and of pdfcraft. */
export interface VersionInfo {
  /** The SDK version (semver). */
  sdk: string;
  /** The pdfcraft commit the viewer was built from. */
  pdfcraftRev: string;
}

/** Error codes. `TIMEOUT`, `DESTROYED` and `FETCH_FAILED` come from the wrapper; the rest from the viewer. */
export type ErrorCode =
  | "BAD_REQUEST"
  | "UNKNOWN_METHOD"
  | "UNSUPPORTED_PROTOCOL"
  | "NOT_READY"
  | "NO_DOCUMENT"
  | "LOAD_FAILED"
  | "NEEDS_PASSWORD"
  | "WRONG_PASSWORD"
  | "NOT_PERMITTED"
  | "READ_ONLY"
  | "XFDF_INVALID"
  | "TOO_LARGE"
  | "BUSY"
  | "INTERNAL"
  | "TIMEOUT"
  | "DESTROYED"
  | "FETCH_FAILED";

/** An error as data: what events carry. */
export interface ErrorInfo {
  /** Machine-readable code. */
  code: ErrorCode;
  /** Human-readable message (English). */
  message: string;
}

/** Every failure of the SDK: a typed code and a message. */
export class FreePdfError extends Error implements ErrorInfo {
  /** Machine-readable code. */
  readonly code: ErrorCode;

  /**
   * @param code - Machine-readable code.
   * @param message - Human-readable message.
   */
  constructor(code: ErrorCode, message: string) {
    super(message);
    this.name = "FreePdfError";
    this.code = code;
  }
}

/** Events and the data each one carries. */
export interface EventMap {
  /** The viewer is configured and ready for requests. */
  ready: VersionInfo;
  /** A document opened. */
  documentLoaded: DocumentInfo;
  /** A document could not be opened. */
  documentError: ErrorInfo;
  /** Comments were added, changed or removed (by the user or the API). */
  annotationsChanged: { count: number };
  /** The document gained or lost unsaved changes. */
  dirtyChanged: { dirty: boolean };
  /** Something failed outside a request (for example a wrong `password` at load). */
  error: ErrorInfo;
  /** The user pressed Save in the viewer: here is the saved PDF. The viewer treats it as saved. */
  saveRequested: { name: string; saveAs: boolean; data: Uint8Array };
}

/** An event name. */
export type EventName = keyof EventMap;

/** A listener for event `K`. */
export type Listener<K extends EventName> = (data: EventMap[K]) => void;

/** A mounted viewer. Every method is async and rejects with {@link FreePdfError}. */
export interface FreePdfViewer {
  /** Opens a document, replacing the one shown. */
  load(document: DocumentSource, options?: { password?: string }): Promise<LoadResult>;
  /** Lifts owner restrictions with the owner password; resolves to the new permissions. */
  unlock(password: string): Promise<Permissions>;
  /** What the open document's security allows. */
  getPermissions(): Promise<Permissions>;
  /** Imports comments from XFDF (ComPDF's included); `replace` deletes the existing ones first. Resolves to the number imported. */
  importXfdf(xfdf: string | Blob, options?: { replace?: boolean }): Promise<number>;
  /** The document's comments as XFDF (author in `title`). */
  exportXfdf(): Promise<string>;
  /** The document's comments with their replies; the length is the exact count. */
  getAnnotations(): Promise<Annotation[]>;
  /** Deletes every comment (one undo step); resolves to how many there were. */
  deleteAllAnnotations(): Promise<number>;
  /** Sets the author of new comments. */
  setAuthor(author: string): Promise<void>;
  /** Sets the colour theme. */
  setTheme(theme: Theme): Promise<void>;
  /** Switches between `view` (read-only) and `annotate`. */
  setMode(mode: Mode): Promise<void>;
  /** Hides parts of the interface (replaces the previous list). */
  setDisabledElements(ids: ElementId[]): Promise<void>;
  /** Sets the interface language; resolves to the language used. */
  setLocale(locale: string): Promise<string>;
  /** Number of pages. */
  getPageCount(): Promise<number>;
  /** Shows page `page` (1-based). */
  goToPage(page: number): Promise<void>;
  /** Sets the zoom: a percentage, `fit-width` or `fit-page`. */
  setZoom(zoom: Zoom): Promise<void>;
  /** Whether there are unsaved changes. */
  isDirty(): Promise<boolean>;
  /** The PDF with every change (comments included); the viewer then treats it as saved. */
  save(): Promise<Uint8Array>;
  /** Versions of the SDK and pdfcraft. */
  version(): Promise<VersionInfo>;
  /** Removes the viewer; pending requests reject with `DESTROYED`. Safe to call twice. */
  destroy(): Promise<void>;
  /** Adds a listener. */
  on<K extends EventName>(name: K, listener: Listener<K>): void;
  /** Removes a listener. */
  off<K extends EventName>(name: K, listener: Listener<K>): void;
  /** The iframe holding the viewer. */
  readonly iframe: HTMLIFrameElement;
}

const HELLO_TIMEOUT_MS = 30_000;
const DEFAULT_TIMEOUT_MS = 60_000;

interface Pending {
  resolve: (value: unknown) => void;
  reject: (error: FreePdfError) => void;
  timer: ReturnType<typeof setTimeout>;
}

type AnyListener = (data: never) => void;

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null;
}

function asErrorInfo(v: unknown): ErrorInfo {
  if (isRecord(v) && typeof v.code === "string" && typeof v.message === "string") {
    return { code: v.code as ErrorCode, message: v.message };
  }
  return { code: "INTERNAL", message: "malformed error from the viewer" };
}

function channelId(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

function nameFromUrl(url: string): string {
  try {
    const last = new URL(url, location.href).pathname.split("/").filter(Boolean).pop();
    return last ? decodeURIComponent(last) : "document.pdf";
  } catch {
    return "document.pdf";
  }
}

/** Copies the document's bytes into a fresh ArrayBuffer (safe to transfer). A URL is fetched within `timeoutMs`. */
async function readDocument(source: DocumentSource, timeoutMs: number): Promise<{ name: string; bytes: ArrayBuffer }> {
  if ("url" in source) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeoutMs);
    const callerSignal = source.fetchInit?.signal;
    const onAbort = (): void => controller.abort();
    callerSignal?.addEventListener("abort", onAbort, { once: true });
    if (callerSignal?.aborted) controller.abort();
    try {
      const response = await fetch(source.url, { ...source.fetchInit, signal: controller.signal });
      if (!response.ok) throw new FreePdfError("FETCH_FAILED", `could not fetch the document: HTTP ${response.status}`);
      return { name: source.name ?? nameFromUrl(source.url), bytes: await response.arrayBuffer() };
    } catch (e) {
      if (e instanceof FreePdfError) throw e;
      const why = controller.signal.aborted && !callerSignal?.aborted ? `no response in ${timeoutMs} ms` : String(e);
      throw new FreePdfError("FETCH_FAILED", `could not fetch the document: ${why}`);
    } finally {
      clearTimeout(timer);
      callerSignal?.removeEventListener("abort", onAbort);
    }
  }
  const data = source.data;
  if (data instanceof Blob) {
    const fileName = typeof File !== "undefined" && data instanceof File ? data.name : undefined;
    return { name: source.name ?? fileName ?? "document.pdf", bytes: await data.arrayBuffer() };
  }
  if (data instanceof ArrayBuffer) return { name: source.name ?? "document.pdf", bytes: data.slice(0) };
  if (data instanceof Uint8Array) {
    const copy = new Uint8Array(data.byteLength);
    copy.set(data);
    return { name: source.name ?? "document.pdf", bytes: copy.buffer };
  }
  throw new FreePdfError("BAD_REQUEST", "document needs a url or data (ArrayBuffer, Uint8Array or Blob)");
}

class Viewer implements FreePdfViewer {
  readonly iframe: HTMLIFrameElement;
  private readonly origin: string;
  private readonly channel = channelId();
  private readonly pending = new Map<number, Pending>();
  private readonly listeners = new Map<EventName, Set<AnyListener>>();
  private readonly timeoutMs: number;
  private nextId = 1;
  private destroyed = false;
  private hello: { resolve: (info: VersionInfo) => void; reject: (e: FreePdfError) => void } | null = null;
  private readyInfo: VersionInfo | null = null;
  private readonly onMessage = (event: MessageEvent): void => this.receive(event);

  constructor(element: HTMLElement, baseUrl: URL, timeoutMs: number) {
    this.origin = baseUrl.origin;
    this.timeoutMs = timeoutMs;
    this.iframe = document.createElement("iframe");
    this.iframe.src = new URL(`viewer/index.html#ch=${this.channel}`, baseUrl).href;
    this.iframe.title = "PDF viewer";
    this.iframe.setAttribute("allow", "clipboard-read; clipboard-write; fullscreen");
    this.iframe.style.cssText = "display:block;width:100%;height:100%;border:0;";
    window.addEventListener("message", this.onMessage);
    element.appendChild(this.iframe);
  }

  /** Waits for the viewer's hello. */
  started(): Promise<VersionInfo> {
    return new Promise<VersionInfo>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.hello = null;
        reject(new FreePdfError("TIMEOUT", "the viewer did not start (is baseUrl right, and WebGL available?)"));
      }, HELLO_TIMEOUT_MS);
      this.hello = {
        resolve: (info) => {
          clearTimeout(timer);
          resolve(info);
        },
        reject: (e) => {
          clearTimeout(timer);
          reject(e);
        },
      };
    });
  }

  private receive(event: MessageEvent): void {
    if (event.origin !== this.origin || event.source !== this.iframe.contentWindow) return;
    const msg: unknown = event.data;
    if (!isRecord(msg) || msg.ch !== this.channel) return;
    if (msg.fps !== PROTOCOL_VERSION) {
      // Another protocol version: fail what is waiting for it now, not after a timeout.
      const err = new FreePdfError("UNSUPPORTED_PROTOCOL", `the viewer speaks protocol ${String(msg.fps)}; this wrapper speaks ${PROTOCOL_VERSION}`);
      if (msg.kind === "hello" && this.hello) {
        this.hello.reject(err);
        this.hello = null;
        return;
      }
      const p = msg.kind === "res" && typeof msg.id === "number" ? this.pending.get(msg.id) : undefined;
      if (p && typeof msg.id === "number") {
        this.pending.delete(msg.id);
        clearTimeout(p.timer);
        p.reject(err);
        return;
      }
      this.emit("error", { code: err.code, message: err.message });
      return;
    }
    switch (msg.kind) {
      case "hello": {
        const info = isRecord(msg.info) ? msg.info : {};
        const version: VersionInfo = { sdk: String(info.sdk ?? ""), pdfcraftRev: String(info.pdfcraftRev ?? "") };
        if (version.sdk !== SDK_VERSION) {
          console.warn(`Free PDF SDK: the wrapper is ${SDK_VERSION} but the viewer is ${version.sdk}; serve matching files`);
        }
        this.hello?.resolve(version);
        this.hello = null;
        return;
      }
      case "res": {
        if (typeof msg.id !== "number") return;
        const p = this.pending.get(msg.id);
        if (!p) return;
        this.pending.delete(msg.id);
        clearTimeout(p.timer);
        if (msg.ok === true) p.resolve(msg.result);
        else {
          const e = asErrorInfo(msg.error);
          p.reject(new FreePdfError(e.code, e.message));
        }
        return;
      }
      case "evt": {
        if (typeof msg.name !== "string") return;
        if (msg.name === "error" && this.hello) {
          const e = asErrorInfo(msg.data);
          this.hello.reject(new FreePdfError(e.code, e.message));
          this.hello = null;
          return;
        }
        if (msg.name === "documentError" || msg.name === "error") {
          this.emit(msg.name, asErrorInfo(msg.data));
        } else if (this.listeners.has(msg.name as EventName)) {
          this.emit(msg.name as EventName, msg.data as never);
        }
        return;
      }
      default:
        return;
    }
  }

  emit<K extends EventName>(name: K, data: EventMap[K]): void {
    if (name === "ready") this.readyInfo = data as VersionInfo;
    for (const listener of [...(this.listeners.get(name) ?? [])]) {
      try {
        (listener as Listener<K>)(data);
      } catch (e) {
        console.error(`Free PDF SDK: a ${name} listener threw`, e);
      }
    }
  }

  request<T>(method: string, params?: Record<string, unknown>, transfer: Transferable[] = [], timeoutMs = this.timeoutMs): Promise<T> {
    if (this.destroyed) return Promise.reject(new FreePdfError("DESTROYED", "the viewer was destroyed"));
    const contentWindow = this.iframe.contentWindow;
    if (!contentWindow) return Promise.reject(new FreePdfError("NOT_READY", "the viewer is not attached to the page"));
    const id = this.nextId;
    this.nextId = this.nextId >= 0x7fffffff ? 1 : this.nextId + 1;
    return new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new FreePdfError("TIMEOUT", `${method} got no reply in ${timeoutMs} ms`));
      }, timeoutMs);
      this.pending.set(id, { resolve: resolve as (v: unknown) => void, reject, timer });
      // The viewer drops the request once the deadline has passed, so a request that timed out
      // here never takes effect later (frames stall in background tabs).
      const deadline = Date.now() + timeoutMs;
      try {
        contentWindow.postMessage({ fps: PROTOCOL_VERSION, ch: this.channel, kind: "req", id, method, params: params ?? {}, deadline }, this.origin, transfer);
      } catch (e) {
        clearTimeout(timer);
        this.pending.delete(id);
        reject(new FreePdfError("BAD_REQUEST", `${method} could not be sent: ${e instanceof Error ? e.message : String(e)}`));
      }
    });
  }

  async load(source: DocumentSource, options?: { password?: string }): Promise<LoadResult> {
    if (this.destroyed) throw new FreePdfError("DESTROYED", "the viewer was destroyed");
    let doc: { name: string; bytes: ArrayBuffer };
    try {
      doc = await readDocument(source, this.timeoutMs * 2);
    } catch (e) {
      // The viewer reports its own load failures; a fetch that failed never reached it.
      const err = e instanceof FreePdfError ? e : new FreePdfError("FETCH_FAILED", String(e));
      this.emit("documentError", { code: err.code, message: err.message });
      throw err;
    }
    const { name, bytes } = doc;
    const params: Record<string, unknown> = { name, data: bytes };
    if (options?.password !== undefined) params.password = options.password;
    return this.request<LoadResult>("load", params, [bytes], this.timeoutMs * 2);
  }

  async unlock(password: string): Promise<Permissions> {
    const r = await this.request<{ permissions: Permissions }>("unlock", { password });
    return r.permissions;
  }

  getPermissions(): Promise<Permissions> {
    return this.request<Permissions>("getPermissions");
  }

  async importXfdf(xfdf: string | Blob, options?: { replace?: boolean }): Promise<number> {
    const text = typeof xfdf === "string" ? xfdf : await xfdf.text();
    return this.request<number>("importXfdf", { xfdf: text, replace: options?.replace ?? false });
  }

  exportXfdf(): Promise<string> {
    return this.request<string>("exportXfdf");
  }

  getAnnotations(): Promise<Annotation[]> {
    return this.request<Annotation[]>("getAnnotations");
  }

  deleteAllAnnotations(): Promise<number> {
    return this.request<number>("deleteAllAnnotations");
  }

  async setAuthor(author: string): Promise<void> {
    await this.request("setAuthor", { author });
  }

  async setTheme(theme: Theme): Promise<void> {
    await this.request("setTheme", { theme });
  }

  async setMode(mode: Mode): Promise<void> {
    await this.request("setMode", { mode });
  }

  async setDisabledElements(ids: ElementId[]): Promise<void> {
    await this.request("setDisabledElements", { ids: [...ids] });
  }

  async setLocale(locale: string): Promise<string> {
    const r = await this.request<{ locale: string }>("setLocale", { locale });
    return r.locale;
  }

  getPageCount(): Promise<number> {
    return this.request<number>("getPageCount");
  }

  async goToPage(page: number): Promise<void> {
    await this.request("goToPage", { page });
  }

  async setZoom(zoom: Zoom): Promise<void> {
    await this.request("setZoom", { zoom });
  }

  isDirty(): Promise<boolean> {
    return this.request<boolean>("isDirty");
  }

  async save(): Promise<Uint8Array> {
    return this.request<Uint8Array>("save", undefined, [], this.timeoutMs * 2);
  }

  version(): Promise<VersionInfo> {
    return this.request<VersionInfo>("version");
  }

  async destroy(): Promise<void> {
    if (this.destroyed) return;
    this.destroyed = true;
    window.removeEventListener("message", this.onMessage);
    const error = new FreePdfError("DESTROYED", "the viewer was destroyed");
    for (const p of this.pending.values()) {
      clearTimeout(p.timer);
      p.reject(error);
    }
    this.pending.clear();
    this.hello?.reject(error);
    this.hello = null;
    this.listeners.clear();
    this.iframe.remove();
  }

  on<K extends EventName>(name: K, listener: Listener<K>): void {
    let set = this.listeners.get(name);
    if (!set) {
      set = new Set();
      this.listeners.set(name, set);
    }
    set.add(listener as AnyListener);
    if (name === "ready" && this.readyInfo) {
      const info = this.readyInfo;
      queueMicrotask(() => {
        if (this.listeners.get(name)?.has(listener as AnyListener)) (listener as Listener<"ready">)(info);
      });
    }
  }

  off<K extends EventName>(name: K, listener: Listener<K>): void {
    this.listeners.get(name)?.delete(listener as AnyListener);
  }
}

/**
 * Mounts a viewer in `element` (which it fills) and resolves once it is ready and, if
 * `options.document` is given, once that document is open. Rejects with {@link FreePdfError}
 * (and removes the iframe) when the viewer cannot start or the document cannot be opened.
 *
 * @param element - The element to mount the viewer in; give it a size.
 * @param options - Where the SDK is hosted, and how the viewer starts.
 */
export async function init(element: HTMLElement, options: InitOptions): Promise<FreePdfViewer> {
  if (!(element instanceof HTMLElement)) throw new FreePdfError("BAD_REQUEST", "init needs an element to mount the viewer in");
  if (!options || typeof options.baseUrl !== "string") throw new FreePdfError("BAD_REQUEST", "init needs options.baseUrl");
  const base = new URL(options.baseUrl.endsWith("/") ? options.baseUrl : `${options.baseUrl}/`, document.baseURI);
  if (base.origin !== location.origin) {
    throw new FreePdfError("BAD_REQUEST", `the SDK must be served from this page's origin (${location.origin}), not ${base.origin}`);
  }
  const viewer = new Viewer(element, base, options.timeoutMs ?? DEFAULT_TIMEOUT_MS);
  for (const [name, listener] of Object.entries(options.listeners ?? {})) {
    if (typeof listener === "function") viewer.on(name as EventName, listener as Listener<EventName>);
  }
  try {
    const info = await viewer.started();
    // Plain copies: a reactive proxy (Vue) can't be posted to the iframe.
    const config: Record<string, unknown> = {
      mode: options.mode ?? "view",
      theme: options.theme ?? "system",
      disabledElements: [...(options.disabledElements ?? [])],
    };
    if (options.author !== undefined) config.author = options.author;
    if (options.locale !== undefined) config.locale = options.locale;
    await viewer.request("configure", config);
    viewer.emit("ready", info);
    if (options.document) {
      await viewer.load(options.document, options.password === undefined ? undefined : { password: options.password });
    }
    return viewer;
  } catch (e) {
    await viewer.destroy();
    throw e instanceof FreePdfError ? e : new FreePdfError("INTERNAL", String(e));
  }
}

/** The SDK entry point. */
export const FreePdf = {
  /** See {@link init}. */
  init,
  /** The wrapper's version. */
  version: SDK_VERSION,
} as const;

export default FreePdf;
