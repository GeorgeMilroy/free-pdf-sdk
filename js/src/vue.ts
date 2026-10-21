/**
 * Free PDF SDK for Vue 3 (and Nuxt, inside `<ClientOnly>`).
 *
 * ```vue
 * <script setup lang="ts">
 * import { FreePdfViewer } from "free-pdf-sdk/vue";
 * const viewerRef = ref();
 * </script>
 * <template>
 *   <FreePdfViewer ref="viewerRef" base-url="/free-pdf-sdk/" mode="annotate"
 *     :document="{ url: '/files/contract.pdf' }" :features="{ stamps: false }"
 *     @annotations-changed="({ count }) => (n = count)" style="height: 80vh" />
 * </template>
 * <!-- later: await viewerRef.value.viewer()?.exportXfdf() -->
 * ```
 *
 * Changing a prop calls the matching API method; changing `baseUrl` mounts a new viewer.
 *
 * @packageDocumentation
 */

import { type PropType, defineComponent, h, onBeforeUnmount, onMounted, ref, toRaw, watch } from "vue";

import { EVENT_NAMES, FreePdfController, type ViewerOptions } from "./controller.js";
import type { DocumentSource, ElementId, FeatureFlags, FreePdfViewer as Api, Mode, Theme, UiOptions } from "./free-pdf-sdk.js";

/** A deep, plain copy: Vue's reactive proxies can't be posted to the viewer's iframe. */
function plain<T>(v: T): T {
  if (v === undefined || v === null) return v;
  const raw = toRaw(v);
  if (Array.isArray(raw)) return raw.map(plain) as T;
  // Only plain objects are copied; anything else (Blob, ArrayBuffer, Headers, AbortSignal in
  // fetchInit) is handed over as is.
  const proto = typeof raw === "object" ? Object.getPrototypeOf(raw) : undefined;
  if (proto === Object.prototype || proto === null) {
    const out: Record<string, unknown> = {};
    for (const [k, x] of Object.entries(raw as Record<string, unknown>)) out[k] = plain(x);
    return out as T;
  }
  return raw;
}

/** The Free PDF SDK viewer as a Vue component. Events: `ready`, `document-loaded`, `document-error`, `annotations-changed`, `dirty-changed`, `error`, `save-requested`. */
export const FreePdfViewer = defineComponent({
  name: "FreePdfViewer",
  props: {
    baseUrl: { type: String, required: true },
    document: { type: Object as PropType<DocumentSource>, default: undefined },
    mode: { type: String as PropType<Mode>, default: undefined },
    features: { type: Object as PropType<FeatureFlags>, default: undefined },
    ui: { type: Object as PropType<UiOptions>, default: undefined },
    theme: { type: String as PropType<Theme>, default: undefined },
    author: { type: String, default: undefined },
    locale: { type: String, default: undefined },
    password: { type: String, default: undefined },
    disabledElements: { type: Array as PropType<ElementId[]>, default: undefined },
    timeoutMs: { type: Number, default: undefined },
  },
  emits: [...EVENT_NAMES],
  setup(props, { emit, expose }) {
    const el = ref<HTMLElement | null>(null);
    let controller: FreePdfController | null = null;

    const options = (): ViewerOptions => {
      const out: Record<string, unknown> = {};
      for (const [k, v] of Object.entries(props)) if (v !== undefined) out[k] = plain(v);
      return out as unknown as ViewerOptions;
    };

    onMounted(() => {
      if (!el.value) return;
      controller = new FreePdfController(el.value, options(), (name, data) => emit(name, data));
      controller.ready.catch(() => undefined);
    });
    watch(
      () => options(),
      (next) => controller?.update(next),
      { deep: true },
    );
    onBeforeUnmount(() => {
      const c = controller;
      controller = null;
      void c?.destroy();
    });

    expose({
      /** The viewer's API once ready, else `null`. */
      viewer: (): Api | null => controller?.current ?? null,
      /** Resolves with the viewer's API once it is ready. */
      whenReady: (): Promise<Api> => controller?.ready ?? Promise.reject(new Error("the viewer is not mounted")),
    });

    return () => h("div", { ref: el, style: { position: "relative", minHeight: "200px", width: "100%", height: "100%" } });
  },
});

export default FreePdfViewer;
