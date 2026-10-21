---
title: Annotations and XFDF
description: Export, import, count, list and delete comments, set their author, and store them as XFDF in your database, including XFDF written by other PDF tools.
section: Guides
order: 50
---

# Annotations and XFDF

Comments live in the PDF the viewer has open. Most apps don't save a new PDF for every comment:
they keep the original file and store the comments separately as **XFDF**, an XML format for
PDF annotations. Free PDF SDK exports and imports XFDF, so you can save comments to your
database and restore them on top of the original PDF later.

Everything happens in the browser: the SDK sends nothing to a server. Where the XFDF goes is up
to your code.

## Export comments

```ts
const xfdf = await viewer.exportXfdf();
// <?xml version="1.0" encoding="UTF-8"?> <xfdf ...><annots>...</annots></xfdf>
```

`exportXfdf()` resolves to a string with every comment of the open document, replies
included. Each comment's author is in its `title` attribute, its page in a 0-based `page`
attribute and its id in `name`.

The export always contains an `<annots>` element, even when there are no comments. Don't test
`xfdf.includes("<annots")` to find out whether there are comments; count them instead (see
[Count and list comments](#count-and-list-comments)).

## Import comments

```ts
const imported = await viewer.importXfdf(xfdf);
console.log(`${imported} comments imported`);
```

`importXfdf()` accepts a string or a `Blob` (for example a `File` from an `<input>`), adds the
comments to the open document and resolves to the number of top-level comments imported.
Replies are imported with their comment but not counted. Comments on pages the document doesn't
have are skipped. An XFDF with no comments imports 0 without an error.

### Replace the existing comments

To show exactly the stored set, for example when the user picks an earlier revision, pass
`replace: true`:

```ts
await viewer.importXfdf(storedXfdf, { replace: true });
```

The existing comments are deleted first, in the same call. The XFDF is checked before anything
is deleted: if it is invalid, the call rejects with `XFDF_INVALID` and the document keeps its
comments.

### What import needs

`importXfdf` changes the document, so it needs:

- **An annotation feature.** At least one of `comments`, `markup`, `shapes`, `ink`, `stamps`,
  `measurement` or `fillSign` must be on, otherwise it rejects with `READ_ONLY`. `view` and
  `organize` have none; every other mode has at least one (`fill` and `sign` have `fillSign`).
- **Permission from the document.** If the PDF's security forbids comments, it rejects with
  `NOT_PERMITTED` until you unlock it with the owner password (see
  [Protected documents](#protected-documents)).
- **Comments only.** XFDF that carries form field values is refused with `XFDF_INVALID`.

Turning off a single annotation feature does not filter what you import: with
`features: { stamps: false }` users can't add stamps, but stamps in the XFDF are still
imported and shown.

### Show stored comments read-only

`view` mode refuses imports, because nothing may change the document there. To show a stored
comment set that users can't change, import in an annotating mode and then switch to `view`:

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: pdfUrl },
  mode: "annotate",
});
await viewer.importXfdf(storedXfdf, { replace: true });
await viewer.setMode("view");
```

The comments stay on screen; every editing tool disappears. The PDF on your server is
unchanged: imported comments live in the viewer until you call `save()`.

## Count and list comments

```ts
const annotations = await viewer.getAnnotations();
const hasComments = annotations.length > 0;
```

`getAnnotations()` lists the document's top-level comments, each with its replies. Its length
is the exact number of comments, counted the same way as `exportXfdf()` writes them, so you can
rely on it to decide whether there is anything to store.

Each entry is an `Annotation`:

```ts
{
  id: "8f1c2b9e-...",          // XFDF name
  type: "highlight",           // text, freetext, square, circle, line, ink, polygon, polyline,
                               // highlight, underline, squiggly, strikeout, stamp, caret,
                               // fileattachment, sound, redact
  author: "Priya Raman",       // XFDF title
  subject: "Highlight",
  contents: "Check this figure",
  date: "2026-10-21T09:14:03Z",// ISO 8601: last modified, else created
  pageIndex: 2,                // 0-based
  color: "#FFD400",
  replies: [
    { id: "...", author: "Sam Lee", contents: "Fixed in v3", date: "2026-10-21T10:02:41Z" },
  ],
}
```

`contents` is plain text. Replies are listed oldest first.

To follow changes as they happen, listen to `annotationsChanged`. It fires when comments are
added, changed or removed, by the user or by your code, with the current count:

```ts
viewer.on("annotationsChanged", ({ count }) => {
  badge.textContent = String(count);
});
```

## Delete all comments

```ts
const deleted = await viewer.deleteAllAnnotations();
```

Deletes every comment, locked ones included, and resolves to how many there were. Like
`importXfdf`, it rejects with `READ_ONLY` when no annotation feature is on and with
`NOT_PERMITTED` when the document's security forbids comments.

## Set the author of new comments

New comments carry the author's name in their XFDF `title` attribute. Set it when the viewer
starts:

```ts
const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  mode: "annotate",
  author: currentUser.displayName,
});
```

or change it later, for example when a different user signs in:

```ts
await viewer.setAuthor(currentUser.displayName);
```

The name you set wins over any name saved in the viewer's preferences and stays in effect when
you load another document. It is trimmed, must not be empty, and can be up to 200 characters.

Changing the author affects new comments only: existing comments, including imported ones,
keep the author they were written with.

## Store comments in a database

A typical flow: load the original PDF, import the stored XFDF, and save the XFDF back when the
user is done.

```ts
import { FreePdf, FreePdfError } from "free-pdf-sdk";

const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: `/files/${fileId}.pdf` },
  mode: "annotate",
  author: currentUser.displayName,
  features: { save: false }, // your page has its own Save button
});

// Restore.
const res = await fetch(`/api/files/${fileId}/comments`);
if (res.ok) {
  await viewer.importXfdf(await res.text(), { replace: true });
}

// Save.
saveButton.onclick = async () => {
  const annotations = await viewer.getAnnotations();
  const xfdf = await viewer.exportXfdf();
  await fetch(`/api/files/${fileId}/comments`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      xfdf,
      count: annotations.length,
      authors: [...new Set(annotations.map((a) => a.author).filter(Boolean))],
    }),
  });
};
```

Tips:

- **Store the XFDF as text** next to a reference to the PDF. It is usually much smaller than
  the PDF. If your database limits document size, keep large XFDF in file storage instead.
- **Store a summary for queries.** `getAnnotations()` gives you the count, authors, pages and
  dates without parsing XML on the server.
- **Save on change.** To save automatically, listen to `annotationsChanged` and debounce:

  ```ts
  let timer: ReturnType<typeof setTimeout> | undefined;
  viewer.on("annotationsChanged", () => {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      const xfdf = await viewer.exportXfdf();
      await fetch(`/api/files/${fileId}/comments`, { method: "PUT", body: xfdf });
    }, 1000);
  });
  ```

- **Keep revisions.** Store each saved XFDF as a new row; showing an older revision is one
  `importXfdf(xfdf, { replace: true })`.
- **Rename authors on the server if you need to.** The author is a plain `title="..."`
  attribute, so rewriting it (for example from an old placeholder name) is a string
  replacement.

### Save a PDF with the comments in it

When you do want a file with the comments embedded, for download or archiving, call `save()`:

```ts
const pdf = await viewer.save(); // Uint8Array
```

Or let users press Save in the viewer and receive the file in `saveRequested`:

```ts
viewer.on("saveRequested", async ({ name, data }) => {
  await fetch(`/api/files/${fileId}/pdf`, { method: "PUT", body: data });
});
```

## Protected documents

Some PDFs are encrypted with an owner password that forbids commenting. Check before offering
annotation tools:

```ts
const permissions = await viewer.getPermissions();
if (!permissions.canAnnotate && permissions.canUnlock) {
  await viewer.unlock(ownerPassword); // rejects with WRONG_PASSWORD
}
```

Or pass the password when loading: `FreePdf.init(element, { ..., password })` or
`viewer.load(source, { password })`. Unlocking only lifts the restrictions in the viewer: the
file itself stays protected, and a PDF saved with `save()` keeps its protection.

## XFDF compatibility

The SDK reads standard XFDF (ISO 19444-1) as other PDF tools write it: authors in `title`,
0-based `page`, `name` ids, `inreplyto` replies, colours, ink paths, vertices, callouts and
pop-ups. It writes the author in `title` too, so XFDF you have already stored keeps working,
and server code that reads or rewrites `title` attributes doesn't need to change. Annotation
`type` names in `getAnnotations()` are the PDF subtypes in lowercase (`text`, `freetext`,
`square`, `circle`, `line`, `ink`, and so on).

### Known losses

Importing XFDF keeps every annotation with its author, page, colour, contents, replies,
rectangle, ink, vertices, callouts and pop-ups. These details are not kept:

| Lost | Effect |
|---|---|
| Measurement dictionaries (`<measure>`) | Measurements import as plain lines and polygons with their text (for example "105.83 mm"); the scale is not kept. |
| Intents other than callouts (`LineDimension`, `PolygonCloud`, `FreeTextTypeWriter`) | Shown as plain lines, polygons and text boxes. |
| Rich text (`<contents-richtext>`) | The plain `<contents>` is used; fonts, colours and sizes inside the text are lost. |
| Appearance streams (`<appearance>`) | Stamps are redrawn by the viewer: standard stamps keep their name, custom stamp artwork is lost. |
| File attachment and sound data | The annotation stays, its file does not. |
| Line leader extras, dash patterns, cloud borders | Drawn without them. |

### Check your stored XFDF

Before switching production data, round-trip a sample of your stored XFDF covering every
annotation type your users make (including measurements and signatures). A small page can do it:

```ts
import { FreePdf } from "free-pdf-sdk";

const viewer = await FreePdf.init(element, { baseUrl: "/free-pdf-sdk/", mode: "annotate" });

for (const sample of samples) { // [{ pdfUrl, xfdf }] from your database
  await viewer.load({ url: sample.pdfUrl });
  const imported = await viewer.importXfdf(sample.xfdf, { replace: true });
  const list = await viewer.getAnnotations();
  const reimported = await viewer.importXfdf(await viewer.exportXfdf(), { replace: true });
  console.table({
    pdf: sample.pdfUrl,
    imported,
    listed: list.length,
    roundTrip: reimported,
    authors: [...new Set(list.map((a) => a.author))].join(", "),
    pages: [...new Set(list.map((a) => a.pageIndex))].join(", "),
  });
}
```

Compare the counts, authors and pages with what your previous viewer showed for the same records.
