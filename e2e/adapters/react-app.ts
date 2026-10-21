// The React integration under test: built from js/dist, mounted with React 18.
import { createElement, useRef, useState } from "react";
import { createRoot } from "react-dom/client";

import { FreePdfViewer, type FreePdfViewerHandle } from "../../js/dist/react.js";
import type { Mode } from "../../js/dist/free-pdf-sdk.js";

declare global {
  interface Window {
    fpsTest: { events: string[]; handle: FreePdfViewerHandle | null; setMode(m: Mode): void; setStamps(on: boolean): void };
  }
}

function App() {
  const ref = useRef<FreePdfViewerHandle>(null);
  const [mode, setMode] = useState<Mode>("annotate");
  const [stamps, setStamps] = useState(true);
  window.fpsTest = { ...window.fpsTest, handle: null, setMode, setStamps };
  return createElement(FreePdfViewer, {
    ref: (h: FreePdfViewerHandle | null) => {
      (ref as { current: FreePdfViewerHandle | null }).current = h;
      window.fpsTest.handle = h;
    },
    baseUrl: "/sdk/",
    mode,
    features: { stamps },
    document: { url: "/fixtures/plain-3p.pdf" },
    onReady: () => window.fpsTest.events.push("ready"),
    onDocumentLoaded: () => window.fpsTest.events.push("documentLoaded"),
    onAnnotationsChanged: ({ count }) => window.fpsTest.events.push(`annotationsChanged:${count}`),
    style: { height: "600px" },
  });
}

window.fpsTest = { events: [], handle: null, setMode: () => undefined, setStamps: () => undefined };
const el = document.getElementById("app");
if (el) createRoot(el).render(createElement(App));
