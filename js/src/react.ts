/**
 * Free PDF SDK for React (17, 18 and 19; Next.js in a client component).
 *
 * ```tsx
 * import { FreePdfViewer, type FreePdfViewerHandle } from "free-pdf-sdk/react";
 *
 * const ref = useRef<FreePdfViewerHandle>(null);
 * <FreePdfViewer ref={ref} baseUrl="/free-pdf-sdk/" mode="annotate"
 *   document={{ url: "/files/contract.pdf" }} features={{ stamps: false }}
 *   onAnnotationsChanged={({ count }) => setCount(count)} style={{ height: "80vh" }} />
 * // later: await ref.current?.viewer()?.exportXfdf()
 * ```
 *
 * Changing a prop calls the matching API method; changing `baseUrl` mounts a new viewer.
 *
 * @packageDocumentation
 */

import { type CSSProperties, createElement, forwardRef, useEffect, useImperativeHandle, useRef } from "react";

import { EVENT_NAMES, FreePdfController, type ViewerOptions } from "./controller.js";
import type { EventMap, EventName, FreePdfViewer as Api } from "./free-pdf-sdk.js";

type Handlers = {
  [K in EventName as `on${Capitalize<K>}`]?: (data: EventMap[K]) => void;
};

/** Props: the viewer options, event handlers (`onReady`, `onDocumentLoaded`...) and the container's `className`/`style`. */
export type FreePdfViewerProps = ViewerOptions &
  Handlers & {
    /** Class of the element the viewer fills. */
    className?: string;
    /** Style of the element the viewer fills; give it a height. */
    style?: CSSProperties;
  };

/** What a `ref` to {@link FreePdfViewer} gives. */
export interface FreePdfViewerHandle {
  /** The viewer's API once ready, else `null`. */
  viewer(): Api | null;
  /** Resolves with the viewer's API once it is ready. */
  whenReady(): Promise<Api>;
}

const handlerName = (name: EventName): keyof Handlers => `on${name.charAt(0).toUpperCase()}${name.slice(1)}` as keyof Handlers;

function options(props: FreePdfViewerProps): ViewerOptions {
  const { className: _c, style: _s, ...rest } = props;
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(rest)) {
    if (!(k.startsWith("on") && typeof v === "function")) out[k] = v;
  }
  return out as unknown as ViewerOptions;
}

/** The Free PDF SDK viewer as a React component. */
export const FreePdfViewer = forwardRef<FreePdfViewerHandle, FreePdfViewerProps>(function FreePdfViewer(props, ref) {
  const container = useRef<HTMLDivElement>(null);
  const controller = useRef<FreePdfController | null>(null);
  const latest = useRef(props);
  latest.current = props;

  useImperativeHandle(
    ref,
    () => ({
      viewer: () => controller.current?.current ?? null,
      whenReady: () => controller.current?.ready ?? Promise.reject(new Error("the viewer is not mounted")),
    }),
    [],
  );

  // Mount once; later prop changes go through update() (a new baseUrl remounts inside it).
  useEffect(() => {
    const el = container.current;
    if (!el) return undefined;
    const c = new FreePdfController(el, options(latest.current), (name, data) => {
      const handler = latest.current[handlerName(name)] as ((d: typeof data) => void) | undefined;
      handler?.(data);
    });
    c.ready.catch(() => undefined);
    controller.current = c;
    return () => {
      controller.current = null;
      void c.destroy();
    };
  }, []);

  useEffect(() => {
    controller.current?.update(options(props));
  });

  return createElement("div", { ref: container, className: props.className, style: { position: "relative", minHeight: 200, ...props.style } });
});

/** Event names the component forwards, for reference. */
export const REACT_EVENT_PROPS: readonly string[] = EVENT_NAMES.map(handlerName);
