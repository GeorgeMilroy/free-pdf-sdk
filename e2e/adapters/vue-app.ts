// The Vue integration under test: built from js/dist, mounted with Vue 3.
import { createApp, h, ref } from "vue";

import { FreePdfViewer } from "../../js/dist/vue.js";

const mode = ref("annotate");
const stamps = ref(true);
const events: string[] = [];
const viewerRef = ref<{ viewer(): unknown; whenReady(): Promise<unknown> } | null>(null);
(window as unknown as Record<string, unknown>).fpsTest = { events, mode, stamps, viewerRef };

createApp({
  setup: () => () =>
    h(FreePdfViewer, {
      ref: viewerRef,
      baseUrl: "/sdk/",
      mode: mode.value,
      features: { stamps: stamps.value },
      document: { url: "/fixtures/plain-3p.pdf" },
      style: "height: 600px",
      onReady: () => events.push("ready"),
      onDocumentLoaded: () => events.push("documentLoaded"),
      onAnnotationsChanged: (e: { count: number }) => events.push(`annotationsChanged:${e.count}`),
    }),
}).mount("#app");
