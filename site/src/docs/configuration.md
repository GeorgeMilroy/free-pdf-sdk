---
title: Configuration
description: Choose what the viewer offers with a mode, fine-tune it with features, shape the interface with ui options and hide single commands with disabledElements.
section: Configure
order: 30
---

# Configuration

Free PDF SDK shows your users only what you allow. You describe that with four options, each
one more specific than the last:

1. **`mode`**: a preset that says what the viewer is for (`view`, `review`, `annotate`, `fill`,
   `sign`, `organize`, `edit` or `full`). The default is `view`, which is read-only.
2. **`features`**: switches that turn individual features on or off on top of the mode.
3. **`ui`**: which parts of the interface are shown (the Tools panel, the rails, the menu,
   right-hand panels).
4. **`disabledElements`**: single commands or tools to hide, such as one markup tool.

```ts
import { FreePdf } from "free-pdf-sdk";

const viewer = await FreePdf.init(document.getElementById("viewer")!, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/files/contract.pdf" },
  mode: "annotate",                              // 1. what the viewer is for
  features: { stamps: false, measurement: false }, // 2. adjust the preset
  ui: { quickToolbar: false, panels: { layers: false } }, // 3. interface
  disabledElements: ["tool:squiggly"],           // 4. single commands
});
```

Start with the mode closest to what you need, then adjust it with `features`. Reach for `ui`
when the features are right but the layout isn't, and for `disabledElements` only when you need
to remove one command while keeping the rest of its feature.

## What "hidden" means

A feature that is off is hidden **everywhere** in the viewer: menus, toolbars, the Tools panel,
right-hand panels, context menus and keyboard shortcuts. If an action is reached anyway, the
viewer refuses it. You don't need CSS or extra code to make a feature disappear.

When no feature that changes the document is on, the viewer is **read-only**: nothing can
change the document, and `getConfig()` reports `readOnly: true`. That is the case for `view`,
and for any other configuration where you turn every editing feature off.

## Modes

<!-- reference:modes -->

### Choose a mode

| You want users to... | Mode |
|---|---|
| Read, search and print, nothing else | `view` |
| Leave review comments: notes, text boxes, highlights | `review` |
| Mark up the document with any kind of annotation, including Fill & Sign, without changing anything else | `annotate` |
| Fill in a form | `fill` |
| Fill in a form and sign it with a digital ID | `sign` |
| Rearrange, insert, delete or rotate pages and edit bookmarks | `organize` |
| Author documents: content, forms, pages, accessibility | `edit` |
| Use everything the viewer offers | `full` |

Every mode includes the viewing basics: search, copy, print, document properties, help, the
command palette, preferences and the signature panel. Every mode except `view` also includes
undo/redo (`history`) and the viewer's own Save button (`save`).

`full` leaves `javascript` off. Turn it on explicitly if your documents need it:
`features: { javascript: true }`.

## Features

<!-- reference:features -->

### Annotation

`comments`, `markup`, `shapes`, `ink`, `stamps` and `measurement` are the annotation tools.
`comments` also covers replies, review status and the comment list. Annotations are stored in
the PDF and travel as XFDF (see [Annotations and XFDF](/docs/annotations-and-xfdf/)).

### Forms & signing

`fillSign` places typed text, marks and drawn or typed signatures on the page (it works on any
PDF, not only forms). `formFill` fills real form fields, `formDesign` creates and edits them.
`digitalSignatures` signs with a certificate; `signaturePanel` only shows and validates
existing signatures and does not change the document.

### Pages & content

`pages` and `bookmarks` organize the document; `editContent` edits its text and images, links,
headers and footers, watermarks and properties. `flatten`, `redaction`,
`accessibility` and `standards` are specialised tools. `javascript` runs the document's own
scripts and is off in every mode.

### Security

`security` adds or removes passwords and permissions. It is only in `full`.

### Editing

`history` is undo and redo. It does not allow any change by itself: it undoes and redoes the
changes the other features allow.

### Output

`save` is the viewer's own Save button: the saved PDF goes to your page in the
`saveRequested` event, and the viewer never stores it anywhere. `download` (Save As and
copies), `export` (other formats, page extraction) and `print` control the other ways a
document leaves the viewer. `viewer.save()` from your code always works, whatever these
switches say.

### Viewing and Interface

`search`, `copy` and `documentInfo` are viewing aids; `help`, `palette` and `preferences` are
parts of the viewer's own interface. None of them changes the document.

## Recipes

### Annotations only, no other edits

`annotate` gives every kind of annotation and Fill & Sign, with undo/redo and Save, and nothing
that changes pages, content, forms or security:

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/files/artwork.pdf" },
  mode: "annotate",
  author: "Priya Raman",
});
```

To also remove stamps and measurement (and, if you like, Fill & Sign), switch them off:

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/files/artwork.pdf" },
  mode: "annotate",
  features: { stamps: false, measurement: false, fillSign: false },
});
```

The tools disappear from the Tools panel, the floating tool bar, the menus and the shortcuts.

### Review comments only

Notes, text boxes, callouts and text markup with replies and review status:

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/files/draft.pdf" },
  mode: "review",
  author: currentUser.name,
});
```

For sticky notes and replies without highlighting, add `features: { markup: false }`.

### Read-only, with print and download disabled

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/files/confidential.pdf" },
  mode: "view",
  features: { print: false, download: false, export: false, copy: false },
});
```

`download` and `export` are already off in `view`; naming them keeps them off if you later
switch mode, because overrides survive `setMode`. Remember that the viewer runs in the
user's browser: hiding these commands controls the viewer, it is not a way to keep a file from
someone who already has it.

### Forms only

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/forms/application.pdf" },
  mode: "fill",
  features: { fillSign: false },
});

viewer.on("saveRequested", async ({ name, data }) => {
  await fetch(`/api/forms/${encodeURIComponent(name)}`, { method: "PUT", body: data });
});
```

Without `fillSign:false`, users can also type and sign anywhere on the page, which is what you
want for PDFs that are not real forms.

### Page organizer

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/files/report.pdf" },
  mode: "organize",
  ui: { panels: { pages: true } },
});
```

`organize` includes `export` (extracting and splitting pages); add `features: { export: false }`
to keep users to rearranging.

### Everything

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/files/anything.pdf" },
  mode: "full",
  features: { javascript: true }, // only if you trust the documents' scripts
});
```

## Change the configuration at runtime

The mode, features, interface options and hidden elements can change after `init`, without
reloading the viewer or the document (so can the theme, author and language, with `setTheme`,
`setAuthor` and `setLocale`):

| Method | What it does |
|---|---|
| `setMode(mode)` | Switches the preset. Your feature overrides still apply on top of the new mode. |
| `setFeatures(features)` | **Replaces** the feature overrides. Pass `{}` to use the mode's features as they are. |
| `setUi(ui)` | **Merges**: options you leave out keep their value. Panels you name are changed, the others are kept. |
| `setDisabledElements(ids)` | Replaces the list of hidden elements. |
| `getConfig()` | Resolves to the configuration in effect. |

```ts
// Start read-only, let the user switch to annotating.
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/files/artwork.pdf" },
  mode: "view",
  features: { print: false },
});

editButton.onclick = async () => {
  await viewer.setMode("annotate");        // print stays off: the override is kept
  await viewer.setFeatures({ print: false, stamps: false }); // the full list of overrides
  await viewer.setUi({ quickToolbar: false });               // other ui options unchanged

  const config = await viewer.getConfig();
  console.log(config.mode);              // "annotate"
  console.log(config.features.stamps);   // false
  console.log(config.features.ink);      // true
  console.log(config.readOnly);          // false
};
```

`getConfig()` resolves to a `ViewerConfig`: the `mode`, `features` with **every** feature
`true` or `false` (the mode's preset with your overrides applied), the full `ui` options with
the `panels` you set, `disabledElements`, and `readOnly`.

Because `setFeatures` replaces the overrides, keep your overrides in one place and always pass
the complete set:

```ts
const overrides = { print: false };
await viewer.setFeatures({ ...overrides, stamps: false });
```

The framework integrations do this for you: changing the `mode`, `features`, `ui` or
`disabledElements` prop calls the matching method (see [React](/docs/react/)).

Unknown feature keys, `ui` options, panel names and element ids are rejected with
`BAD_REQUEST`, both in `init` and in the `set*` methods.

## Interface options (`ui`)

<!-- reference:ui -->

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  mode: "annotate",
  ui: {
    toolsPanel: true,
    quickToolbar: false,   // no floating tool bar beside the page
    searchBox: false,      // no "Find tools and commands" box
    lockLayout: true,      // panels stay where they are
    externalLinks: false,  // links to web pages in the document don't open
  },
});
```

A few options depend on others: `toolsRail` only matters while `toolsPanel` is on, and
`searchBox` only shows when the `palette` feature is on.

### Panels

`ui.panels` shows or hides the right-hand panels. A panel you don't list shows when its
feature is on; listing it as `false` hides it even then.

<!-- reference:panels -->

```ts
await viewer.setUi({ panels: { layers: false, attachments: false, comments: true } });
```

Setting a panel to `true` does not turn its feature on: the signatures panel still needs
`signaturePanel`, the accessibility and tags panels need `accessibility`.

## Hide single commands (`disabledElements`)

`disabledElements` hides individual commands and tools. Each id stands for one or more
commands in the viewer:

<!-- reference:elements -->

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  mode: "annotate",
  disabledElements: ["tool:squiggly", "tool:caret", "tool:replace", "settings"],
});

// Later: the list is replaced, not extended.
await viewer.setDisabledElements(["tool:squiggly"]);
```

### Features or disabledElements?

Prefer **features** whenever one matches what you want to remove:

- A feature removes a whole capability everywhere and the viewer refuses it if it is reached
  anyway. `features: { stamps: false }` is the reliable way to forbid stamps.
- A feature that is off changes what the viewer considers an edit: with every editing feature
  off, the viewer becomes read-only.
- `disabledElements` hides the listed commands from the interface and nothing else. Use it to
  remove one tool inside a feature you keep, for example squiggly underline while keeping the
  other markup tools, or the viewer's Save command while keeping undo.

Neither affects your own code: `disabledElements: ["import"]` hides the viewer's Import
commands, but `viewer.importXfdf()` still works. The API refuses `importXfdf` and
`deleteAllAnnotations` with `READ_ONLY` only when no annotation feature is on.

## Next steps

- [API reference](/docs/api/): every option, method, event and error code.
- [Annotations and XFDF](/docs/annotations-and-xfdf/): store and restore comments.
