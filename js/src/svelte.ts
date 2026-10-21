/**
 * Free PDF SDK for Svelte (4 and 5, and SvelteKit in the browser): an action.
 *
 * ```svelte
 * <script>
 *   import { freePdf } from "free-pdf-sdk/svelte";
 *   let mode = "annotate";
 * </script>
 * <div style="height: 80vh"
 *   use:freePdf={{ baseUrl: "/free-pdf-sdk/", mode, document: { url: "/files/contract.pdf" } }}
 *   on:fps-annotations-changed={(e) => console.log(e.detail.count)}></div>
 * ```
 *
 * The node receives the same `fps-*` DOM events as `<free-pdf-viewer>`; `options.onViewer`
 * hands over the API once the viewer is ready. Changed options call the matching API methods.
 *
 * @packageDocumentation
 */

import { ELEMENT_EVENTS, FreePdfController, type ViewerOptions } from "./controller.js";
import type { FreePdfViewer as Api } from "./free-pdf-sdk.js";

/** The action's options: the viewer options and an optional callback with the API. */
export type FreePdfActionOptions = ViewerOptions & {
  /** Called with the viewer's API once it is ready. */
  onViewer?: (viewer: Api) => void;
};

const viewerOptions = ({ onViewer: _o, ...rest }: FreePdfActionOptions): ViewerOptions => rest;

/**
 * Mounts a viewer in `node`.
 *
 * @param node - The element the viewer fills; give it a height.
 * @param options - Viewer options.
 */
export function freePdf(node: HTMLElement, options: FreePdfActionOptions): { update(options: FreePdfActionOptions): void; destroy(): void } {
  let latest = options;
  const controller = new FreePdfController(node, viewerOptions(options), (name, data) => {
    node.dispatchEvent(new CustomEvent(ELEMENT_EVENTS[name], { detail: data }));
  });
  // Hands over the API of every viewer the action mounts (a new baseUrl mounts a new one).
  let handed = controller.ready;
  handed.then((v) => latest.onViewer?.(v)).catch(() => undefined);
  return {
    update(next: FreePdfActionOptions): void {
      latest = next;
      controller.update(viewerOptions(next));
      if (controller.ready !== handed) {
        handed = controller.ready;
        handed.then((v) => latest.onViewer?.(v)).catch(() => undefined);
      }
    },
    destroy(): void {
      void controller.destroy();
    },
  };
}
