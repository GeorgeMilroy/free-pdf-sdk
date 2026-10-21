---
title: Vue
description: Embed the viewer in Vue 3 and Nuxt with the FreePdfViewer component, its props, kebab-case events and template ref API.
section: Frameworks
order: 42
---

# Vue

`free-pdf-sdk/vue` exports `FreePdfViewer`, a Vue 3 component. Props configure the viewer,
emitted events report what happens, and a template ref gives you the full API.

```vue
<script setup lang="ts">
import { FreePdfViewer } from "free-pdf-sdk/vue";
</script>

<template>
  <FreePdfViewer base-url="/free-pdf-sdk/" :document="{ url: '/files/contract.pdf' }" style="height: 80vh" />
</template>
```

## Install

```bash
npm install free-pdf-sdk
npx free-pdf-sdk copy-viewer public/free-pdf-sdk
```

The second command copies the viewer files (`viewer/`, `version.json`, the licences) into
`public/free-pdf-sdk`, which Vite and Nuxt serve at `/free-pdf-sdk/`. That URL is `baseUrl`, and
it must be on the same origin as your app. Run the command again after every upgrade, for example
from a `postinstall` script:

```json
{
  "scripts": {
    "postinstall": "free-pdf-sdk copy-viewer public/free-pdf-sdk"
  }
}
```

The component needs Vue 3.3 or later.

## Props

| Prop | Type | Notes |
|---|---|---|
| `base-url` | `string` | Required. URL of the viewer folder, on the page's origin. |
| `document` | `{ url, name?, fetchInit? }` or `{ data, name? }` | The PDF to open. `data` is an `ArrayBuffer`, `Uint8Array` or `Blob`. |
| `mode` | `Mode` | `view` (default, read-only), `review`, `annotate`, `fill`, `sign`, `organize`, `edit` or `full`. |
| `features` | `FeatureFlags` | Feature switches on top of the mode, such as `{ stamps: false }`. |
| `ui` | `UiOptions` | Interface options, such as `{ toolsPanel: false }`. |
| `disabled-elements` | `ElementId[]` | Parts of the interface to hide, such as `['download', 'print']`. |
| `theme` | `'light' \| 'dark' \| 'system'` | Default `system`. |
| `author` | `string` | Author of new comments. |
| `locale` | `string` | Interface language, such as `en`, `de` or `zh-Hans`. |
| `password` | `string` | Used when `document` is opened: opens a protected file and lifts owner restrictions. |
| `timeout-ms` | `number` | How long requests wait for a reply (default 60000). Read once, at mount. |

Props can be written in kebab-case or camelCase (`base-url` or `baseUrl`).

## Size

The component renders a `div` with `position: relative`, `width: 100%`, `height: 100%` and a
`min-height` of 200px; the viewer fills it. Either give the parent a height, or set one on the
component: `class` and `style` fall through to the `div`.

```vue
<FreePdfViewer class="viewer" base-url="/free-pdf-sdk/" :document="doc" />

<style scoped>
.viewer { height: calc(100vh - 64px); }
</style>
```

## Events

The component emits every viewer event. Listen in kebab-case:

| Event | Payload | When |
|---|---|---|
| `@ready` | `{ sdk, pdfcraftRev }` | The viewer is configured and ready for requests. |
| `@document-loaded` | `{ pageCount, permissions }` | A document opened. |
| `@document-error` | `{ code, message }` | A document could not be opened. |
| `@annotations-changed` | `{ count }` | Comments were added, changed or removed, by the user or the API. |
| `@dirty-changed` | `{ dirty }` | The document gained or lost unsaved changes. |
| `@error` | `{ code, message }` | Something failed outside a request: the viewer could not start, a prop change was refused. |
| `@save-requested` | `{ name, saveAs, data }` | The user pressed Save in the viewer. `data` is the PDF as a `Uint8Array`; the viewer now treats it as saved. |

```vue
<FreePdfViewer
  base-url="/free-pdf-sdk/"
  mode="annotate"
  :document="{ url: '/files/contract.pdf' }"
  @document-loaded="({ pageCount }) => (pages = pageCount)"
  @annotations-changed="({ count }) => (comments = count)"
  @error="({ code, message }) => console.error(code, message)"
/>
```

## Call the API with a template ref

The component exposes two methods:

- `viewer()` returns the viewer's API once it is ready, else `null`.
- `whenReady()` resolves with the API once the viewer is ready and its first document has loaded (or failed to).

```vue
<script setup lang="ts">
import { ref } from "vue";
import type { FreePdfViewer as Api } from "free-pdf-sdk";
import { FreePdfViewer } from "free-pdf-sdk/vue";

type ViewerHandle = { viewer(): Api | null; whenReady(): Promise<Api> };
const viewerRef = ref<ViewerHandle | null>(null);

async function firstPage() {
  const viewer = await viewerRef.value?.whenReady();
  await viewer?.goToPage(1);
}
</script>

<template>
  <button @click="firstPage">First page</button>
  <FreePdfViewer ref="viewerRef" base-url="/free-pdf-sdk/" :document="{ url: '/files/report.pdf' }" style="height: 80vh" />
</template>
```

Every API method is async and rejects with a `FreePdfError` that has a `code` and a `message`
(import `FreePdfError` from `free-pdf-sdk`). Don't store the API object in reactive state; keep it
in a local variable, or use `markRaw` if you must.

## Reactive props

Props are watched deeply. A change calls the matching API method on the running viewer, in this
order: `setMode`, `setFeatures`, `setUi`, `setDisabledElements`, `setTheme`, `setAuthor`,
`setLocale`, then `load` for a new `document`. A refused change is reported through `@error`.
Reactive objects are copied to plain objects before they are sent to the viewer, so you can pass
`reactive()` state directly.

```vue
<script setup lang="ts">
import { reactive, ref } from "vue";
import type { Mode } from "free-pdf-sdk";
import { FreePdfViewer } from "free-pdf-sdk/vue";

const mode = ref<Mode>("review");
const features = reactive({ stamps: false, measurement: false });
const ui = reactive({ quickToolbar: true, panels: { bookmarks: false } });
const dark = ref(false);
</script>

<template>
  <label><input type="checkbox" v-model="features.stamps" /> Stamps</label>
  <label><input type="checkbox" v-model="ui.quickToolbar" /> Quick toolbar</label>
  <label><input type="checkbox" v-model="dark" /> Dark</label>
  <select v-model="mode">
    <option value="view">View</option>
    <option value="review">Review</option>
    <option value="annotate">Annotate</option>
  </select>

  <FreePdfViewer
    base-url="/free-pdf-sdk/"
    :mode="mode"
    :features="features"
    :ui="ui"
    :theme="dark ? 'dark' : 'light'"
    :document="{ url: '/files/contract.pdf' }"
    style="height: 80vh"
  />
</template>
```

How changes are detected:

- `features`, `ui` and `disabled-elements` are compared by value.
- `document` with a `url` is compared by `url` and `name`; with `data`, by the identity of the
  bytes. Keep the same `ArrayBuffer`, `Uint8Array` or `Blob` to avoid reloads.
- `features` replaces the overrides: clearing the prop goes back to the mode's own features.
  `ui` options you leave out keep their value, and clearing `mode`, `theme`, `author` or
  `locale` keeps the current value.
- `password` is only used when a document is opened; change it together with `document`.
- Changing `base-url` destroys the viewer and mounts a new one.

Calls you make through the API don't change the props. If you call `viewer.setMode()` yourself,
keep your `mode` prop in step.

## Example: annotate and save comments as XFDF

```vue
<script setup lang="ts">
import { ref } from "vue";
import type { FreePdfViewer as Api } from "free-pdf-sdk";
import { FreePdfViewer } from "free-pdf-sdk/vue";

const props = defineProps<{ contractId: string; userName: string }>();
const viewerRef = ref<{ viewer(): Api | null; whenReady(): Promise<Api> } | null>(null);
const count = ref(0);
const status = ref("");

async function restore() {
  const viewer = await viewerRef.value?.whenReady();
  if (!viewer) return;
  const res = await fetch(`/api/contracts/${props.contractId}/annotations`);
  if (res.ok) await viewer.importXfdf(await res.text(), { replace: true });
}

async function save() {
  const viewer = viewerRef.value?.viewer();
  if (!viewer) return;
  const xfdf = await viewer.exportXfdf();
  await fetch(`/api/contracts/${props.contractId}/annotations`, {
    method: "PUT",
    headers: { "Content-Type": "application/vnd.adobe.xfdf" },
    body: xfdf,
  });
  status.value = "Saved";
}

async function upload({ data }: { data: Uint8Array }) {
  await fetch(`/api/contracts/${props.contractId}/pdf`, { method: "PUT", body: data });
  status.value = "PDF saved";
}
</script>

<template>
  <div class="page">
    <header>
      {{ count }} comments
      <button @click="save">Save comments</button>
      {{ status }}
    </header>
    <FreePdfViewer
      ref="viewerRef"
      class="viewer"
      base-url="/free-pdf-sdk/"
      mode="annotate"
      :author="userName"
      :disabled-elements="['download']"
      :document="{ url: `/api/contracts/${contractId}/pdf` }"
      @document-loaded="restore"
      @annotations-changed="({ count: n }) => (count = n)"
      @save-requested="upload"
      @error="({ code, message }) => (status = `${code}: ${message}`)"
    />
  </div>
</template>

<style scoped>
.page { display: flex; flex-direction: column; height: 100vh; }
.viewer { flex: 1; }
</style>
```

`@document-loaded` can fire before `viewer()` is set for the first document, so `restore` waits
with `whenReady()`. In `annotate` mode comments can be imported; without an annotation feature
(`view` or `organize` mode, say) `importXfdf` rejects with `READ_ONLY`.

## Nuxt

The viewer runs only in the browser. Wrap the component in `<ClientOnly>`:

```vue
<!-- pages/contracts/[id].vue -->
<script setup lang="ts">
import { FreePdfViewer } from "free-pdf-sdk/vue";

const route = useRoute();
</script>

<template>
  <ClientOnly>
    <FreePdfViewer
      base-url="/free-pdf-sdk/"
      mode="review"
      :document="{ url: `/api/contracts/${route.params.id}/pdf` }"
      style="height: calc(100vh - 64px)"
    />
    <template #fallback>
      <p>Loading viewer…</p>
    </template>
  </ClientOnly>
</template>
```

Copy the viewer into Nuxt's public folder (`public/` in Nuxt 3 and 4):

```bash
npx free-pdf-sdk copy-viewer public/free-pdf-sdk
```

If the app is served under a base URL (`app.baseURL`), include it in `base-url`.

## Things to know

- Unmounting the component destroys the viewer; pending calls reject with `DESTROYED`.
- If a document can't be opened, `@document-error` reports why and the viewer keeps running:
  set another `document` to try again.
- The viewer is an iframe on your origin that talks to the page over `postMessage`. PDFs are
  processed in the browser with WebAssembly and are not uploaded anywhere by the SDK. It needs
  WebGL; if it can't start, `@error` reports `TIMEOUT` (usually a wrong `base-url`) or
  `INTERNAL` (for example without WebGL).
- See [Vanilla JavaScript](/docs/vanilla-js/) for the core `FreePdf.init` API the component
  wraps.
