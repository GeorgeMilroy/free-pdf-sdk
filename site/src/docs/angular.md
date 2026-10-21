---
title: Angular
description: Use Free PDF SDK in Angular through the <free-pdf-viewer> Web Component, with property bindings, fps-* events and the viewer API.
section: Frameworks
order: 43
---

# Angular

Angular uses Free PDF SDK through its Web Component, `<free-pdf-viewer>`. You register the
element once, allow custom elements in your component, and then bind to it like any other
element: attributes and properties in, `fps-*` events out.

## Install

```bash
npm install @mecorelabs/free-pdf-sdk
npx @mecorelabs/free-pdf-sdk copy-viewer public/free-pdf-sdk
```

The second command copies the viewer files (`viewer/`, `version.json`, the licences) into a
folder your app serves. Projects created with Angular 18 or later serve `public/` at the site root, so the viewer is
at `/free-pdf-sdk/`. In an older project that uses `src/assets/`, copy it to
`src/assets/free-pdf-sdk` and use `/assets/free-pdf-sdk/`. That URL is the `base-url`, and it
must be on the same origin as the app.

Run the command again after every upgrade, for example from a `postinstall` script:

```json
{
  "scripts": {
    "postinstall": "free-pdf-sdk copy-viewer public/free-pdf-sdk"
  }
}
```

## Register the element

Call `defineFreePdfViewer()` in `main.ts`, before bootstrapping:

```ts
// src/main.ts
import { bootstrapApplication } from "@angular/platform-browser";
import { defineFreePdfViewer } from "@mecorelabs/free-pdf-sdk/element";
import { appConfig } from "./app/app.config";
import { AppComponent } from "./app/app.component";

defineFreePdfViewer();

bootstrapApplication(AppComponent, appConfig).catch((err) => console.error(err));
```

## Use it in a standalone component

Add `CUSTOM_ELEMENTS_SCHEMA` so Angular accepts `<free-pdf-viewer>` and its bindings:

```ts
// src/app/contract-viewer.component.ts
import { CUSTOM_ELEMENTS_SCHEMA, Component } from "@angular/core";

@Component({
  selector: "app-contract-viewer",
  standalone: true,
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  template: `
    <free-pdf-viewer
      base-url="/free-pdf-sdk/"
      src="/files/contract.pdf"
      mode="review"
    ></free-pdf-viewer>
  `,
  styles: [`free-pdf-viewer { display: block; height: calc(100vh - 64px); }`],
})
export class ContractViewerComponent {}
```

The element is a block with a `min-height` of 200px; the viewer fills it. Give it a height in
your component's styles, as above.

## Bind options

Static values can be plain attributes. For values from your component, bind properties where
the element has them and attributes otherwise.

The element has these properties: `baseUrl`, `mode`, `features`, `ui` and `theme`, plus
`options`, which takes several options at once (`document`, `author`, `locale`, `password`,
`disabledElements`, `timeoutMs`, and the ones above). Objects such as `features` and `ui` are best
bound as properties, so no JSON is involved.

```ts
import { CUSTOM_ELEMENTS_SCHEMA, Component, signal } from "@angular/core";
import type { FeatureFlags, Mode, UiOptions } from "@mecorelabs/free-pdf-sdk";

@Component({
  selector: "app-review",
  standalone: true,
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  template: `
    <select [value]="mode()" (change)="mode.set($any($event.target).value)">
      <option value="view">View</option>
      <option value="review">Review</option>
      <option value="annotate">Annotate</option>
    </select>

    <free-pdf-viewer
      base-url="/free-pdf-sdk/"
      [attr.src]="pdfUrl"
      [mode]="mode()"
      [features]="features"
      [ui]="ui"
      [options]="{ author: userName, disabledElements: ['download', 'print'] }"
      [attr.theme]="dark ? 'dark' : 'light'"
    ></free-pdf-viewer>
  `,
  styles: [`free-pdf-viewer { display: block; height: 80vh; }`],
})
export class ReviewComponent {
  pdfUrl = "/files/contract.pdf";
  userName = "Priya Raman";
  dark = false;
  mode = signal<Mode>("review");
  features: FeatureFlags = { stamps: false, measurement: false };
  ui: UiOptions = { quickToolbar: false };
}
```

Every change calls the matching API method on the running viewer (`mode` calls `setMode`, a new
`src` opens that document, and so on), batched per task. A refused change is reported as an
`fps-error` event. Changing `base-url` mounts a new viewer.

Keep these in mind:

- Assign a new object to change `features` or `ui`; Angular only re-binds when the reference
  changes. `features` replaces the overrides (relative to the mode); `ui` options you leave out
  keep their value.
- `options` merges into what was set before, and a value set through a property wins over the
  attribute of the same name from then on.
- To open a document from bytes (an upload, a `Blob` from your API), set it through `options`:
  `this.pdf.nativeElement.options = { document: { data: file, name: file.name } }` (with the
  `ViewChild` shown below), or bind `[options]="{ document: { data: fileBytes, name: 'scan.pdf' } }"`.
- Attributes are strings. Use `[attr.disabled-elements]="'download print'"` (comma or space
  separated) and `[attr.features]` only with JSON text; properties are simpler.

## Listen to events

The element dispatches DOM `CustomEvent`s with the data in `detail`. Bind them with Angular's
event syntax:

| Event | `detail` |
|---|---|
| `(fps-ready)` | `{ sdk, pdfcraftRev }` |
| `(fps-document-loaded)` | `{ pageCount, permissions }` |
| `(fps-document-error)` | `{ code, message }` |
| `(fps-annotations-changed)` | `{ count }` |
| `(fps-dirty-changed)` | `{ dirty }` |
| `(fps-error)` | `{ code, message }` |
| `(fps-save-requested)` | `{ name, saveAs, data }` (`data` is the saved PDF as a `Uint8Array`) |

```ts
@Component({
  selector: "app-counter",
  standalone: true,
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  template: `
    <p>{{ count }} comments</p>
    <free-pdf-viewer
      base-url="/free-pdf-sdk/"
      src="/files/contract.pdf"
      mode="annotate"
      (fps-annotations-changed)="onAnnotationsChanged($event)"
      (fps-error)="onError($event)"
    ></free-pdf-viewer>
  `,
  styles: [`free-pdf-viewer { display: block; height: 80vh; }`],
})
export class CounterComponent {
  count = 0;

  onAnnotationsChanged(event: Event) {
    this.count = (event as CustomEvent<{ count: number }>).detail.count;
  }

  onError(event: Event) {
    const { code, message } = (event as CustomEvent<{ code: string; message: string }>).detail;
    console.error(code, message);
  }
}
```

## Call the API

Get the element with `ViewChild` and an `ElementRef`. Its `whenReady()` resolves with the
viewer's full API once the viewer is ready and the document from `src` is open; its `viewer`
property returns the API synchronously, or `null` before then.

```ts
import { CUSTOM_ELEMENTS_SCHEMA, Component, ElementRef, ViewChild } from "@angular/core";
import { FreePdfError } from "@mecorelabs/free-pdf-sdk";
import type { FreePdfViewerElement } from "@mecorelabs/free-pdf-sdk/element";

@Component({
  selector: "app-contract-review",
  standalone: true,
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  template: `
    <header>
      {{ count }} comments
      <button (click)="saveComments()">Save comments</button>
      <button (click)="downloadPdf()">Download PDF</button>
      {{ status }}
    </header>
    <free-pdf-viewer
      #pdf
      base-url="/free-pdf-sdk/"
      [attr.src]="'/api/contracts/' + contractId + '/pdf'"
      mode="annotate"
      [options]="{ author: userName }"
      (fps-document-loaded)="restoreComments()"
      (fps-annotations-changed)="count = $any($event).detail.count"
    ></free-pdf-viewer>
  `,
  styles: [
    `:host { display: flex; flex-direction: column; height: 100vh; }
     free-pdf-viewer { display: block; flex: 1; }`,
  ],
})
export class ContractReviewComponent {
  @ViewChild("pdf") pdf!: ElementRef<FreePdfViewerElement>;

  contractId = "42";
  userName = "Priya Raman";
  count = 0;
  status = "";

  async restoreComments() {
    const viewer = await this.pdf.nativeElement.whenReady();
    const res = await fetch(`/api/contracts/${this.contractId}/annotations`);
    if (res.ok) await viewer.importXfdf(await res.text(), { replace: true });
  }

  async saveComments() {
    try {
      const viewer = await this.pdf.nativeElement.whenReady();
      const xfdf = await viewer.exportXfdf();
      await fetch(`/api/contracts/${this.contractId}/annotations`, {
        method: "PUT",
        headers: { "Content-Type": "application/vnd.adobe.xfdf" },
        body: xfdf,
      });
      this.status = "Saved";
    } catch (e) {
      this.status = e instanceof FreePdfError ? `Could not save (${e.code})` : "Could not save";
    }
  }

  async downloadPdf() {
    const viewer = await this.pdf.nativeElement.whenReady();
    const bytes = await viewer.save();
    const url = URL.createObjectURL(new Blob([bytes], { type: "application/pdf" }));
    const a = document.createElement("a");
    a.href = url;
    a.download = "contract.pdf";
    a.click();
    URL.revokeObjectURL(url);
  }
}
```

`whenReady()` can be called at any time, including in `ngAfterViewInit`: it resolves once the
viewer is ready and its first document has loaded (or failed to).

## Cleanup is automatic

You don't need `ngOnDestroy`. When Angular removes `<free-pdf-viewer>` from the DOM (the
component is destroyed, an `@if` turns false, the route changes), the element destroys its viewer
and pending calls reject with `DESTROYED`. When it is rendered again, a new viewer mounts.

## Server-side rendering

The element only works in the browser. `defineFreePdfViewer()` in `main.ts` runs only in the
browser build; with Angular SSR, keep `@mecorelabs/free-pdf-sdk/element` out of `main.server.ts` and of code
that runs on the server. On the server, `<free-pdf-viewer>` renders as an empty element and
mounts once the app is running in the browser.

## Things to know

- The viewer runs in an iframe served from `base-url`, on your origin, and talks to the page over
  `postMessage`. PDFs are processed in the browser with WebAssembly and are not uploaded anywhere
  by the SDK.
- The viewer needs WebGL. If it can't start, `fps-error` reports `TIMEOUT` (usually a wrong
  `base-url`) or `INTERNAL` (the viewer loaded but could not start, for example without WebGL).
- If a document can't be opened, `(fps-document-error)` reports why and the viewer keeps running;
  bind another `src` to try again.
- [Web Component](/docs/web-component/) describes every attribute, property and event of the
  element in full.
