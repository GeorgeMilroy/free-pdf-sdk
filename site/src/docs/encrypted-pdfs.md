---
title: Encrypted PDFs
description: Open password-protected PDFs, lift owner restrictions with the owner password, read permissions and handle NEEDS_PASSWORD, WRONG_PASSWORD and NOT_PERMITTED.
section: Guides
order: 51
---

# Encrypted PDFs

PDFs can be protected in two ways, and the SDK handles both:

- A **user password** (open password) is needed to open the file at all. Without it, loading
  fails with `NEEDS_PASSWORD`.
- An **owner password** (permissions password) restricts what can be done with a file that opens
  freely: commenting, printing, copying. Entering it lifts the restrictions.

Encrypted files, including AES-256, are decrypted in the browser. Passwords travel only over the
same-origin channel between your page and the viewer.

The viewer does not show its own password prompt: your page collects the password and passes it
in.

## Open a file that needs a password

Pass `password` with the document. It is used to open the file and, if the file also has owner
restrictions, to lift them:

```ts
import { FreePdf, FreePdfError } from "@mecorelabs/free-pdf-sdk";

const viewer = await FreePdf.init(el, {
  baseUrl: "/free-pdf-sdk/",
  mode: "annotate",
  document: { url: "/files/statement.pdf" },
  password: "s3cret",
});
```

The `password` option only applies to the `document` you pass to `init`. For documents opened
later, pass it to `load`:

```ts
const result = await viewer.load({ url: "/files/statement.pdf" }, { password: "s3cret" });
console.log(result.pageCount, result.permissions, result.unlocked);
```

## Ask the user for a password

`init` with a `document` rejects (and removes the viewer) when the file needs a password you
didn't give. When you don't know in advance whether a file is protected, start the viewer
without a document and open it with `load`, which you can retry:

```ts
import { FreePdf, FreePdfError, type DocumentSource, type FreePdfViewer, type LoadResult } from "@mecorelabs/free-pdf-sdk";

const viewer = await FreePdf.init(el, { baseUrl: "/free-pdf-sdk/", mode: "annotate" });

async function openProtected(viewer: FreePdfViewer, url: string): Promise<LoadResult | null> {
  // Fetch once: `data` is copied for each attempt, so the bytes can be reused.
  const response = await fetch(url);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  const source: DocumentSource = { data: await response.arrayBuffer(), name: url.split("/").pop() };

  let password: string | undefined;
  for (;;) {
    try {
      return await viewer.load(source, password === undefined ? undefined : { password });
    } catch (e) {
      if (!(e instanceof FreePdfError)) throw e;
      if (e.code === "NEEDS_PASSWORD" || e.code === "WRONG_PASSWORD") {
        const prompt = e.code === "WRONG_PASSWORD" ? "Wrong password. Try again:" : "This PDF is protected. Password:";
        const answer = window.prompt(prompt); // use your own dialog in a real app
        if (answer === null) return null;     // the user cancelled
        password = answer;
        continue;
      }
      throw e; // LOAD_FAILED, TOO_LARGE, ...
    }
  }
}

await openProtected(viewer, "/files/statement.pdf");
```

A failed load also fires the `documentError` event with the same `code`, which is convenient
when you open documents through a framework component's `document` prop instead of calling
`load` yourself.

## Lift owner restrictions

A file with an owner password opens normally but may forbid comments, printing or copying.
Check its permissions, then unlock it with the owner password:

```ts
const permissions = await viewer.getPermissions();

if (!permissions.canAnnotate && permissions.canUnlock) {
  try {
    const after = await viewer.unlock(ownerPassword);
    console.log("Can annotate now:", after.canAnnotate);
  } catch (e) {
    if (e instanceof FreePdfError && e.code === "WRONG_PASSWORD") {
      // Ask again, or keep the document restricted.
    }
  }
}
```

`unlock` resolves to the new permissions. On a document with nothing to unlock it changes
nothing and resolves to the current permissions.

You can also unlock while loading: a `password` passed to `init` or `load` that wasn't needed to
open the file is tried as the owner password. The load result says whether it worked:

```ts
const result = await viewer.load({ url: "/files/proof.pdf" }, { password: ownerPassword });
if (!result.unlocked && result.permissions.canUnlock) {
  // The password didn't lift the restrictions.
}
```

A wrong owner password does not fail the load: the document opens, stays restricted, and the
viewer fires an `error` event with `code: "WRONG_PASSWORD"` ("the owner password is incorrect;
the document stays restricted"). With `init`, listen for it through `listeners`, so you hear it
while `init` is still running:

```ts
const viewer = await FreePdf.init(el, {
  baseUrl: "/free-pdf-sdk/",
  mode: "annotate",
  document: { url: "/files/proof.pdf" },
  password: ownerPassword,
  listeners: {
    error: ({ code, message }) => {
      if (code === "WRONG_PASSWORD") console.warn(message);
    },
  },
});
```

### Keep owner passwords on the server

An owner password that you send to the browser is visible to anyone with developer tools. Fetch
it from your server only for users who are allowed to annotate, and remember that owner
restrictions are a convention the PDF reader honours, not encryption of the content: anyone who
can open the file can read it.

## Permissions

`getPermissions()`, the `documentLoaded` event and the result of `load` all describe what the
open document's security allows:

| Field | Meaning |
|---|---|
| `canAnnotate` | Comments may be added, changed, deleted and imported. True only when the document allows both commenting and modification. |
| `canPrint` | The document may be printed. |
| `canCopy` | Text and images may be copied. |
| `isEncrypted` | The document is encrypted. |
| `canUnlock` | An owner password would lift restrictions (`unlock`). |

```ts
viewer.on("documentLoaded", ({ pageCount, permissions }) => {
  if (permissions.isEncrypted && !permissions.canAnnotate) {
    showBanner("This document is protected. Enter the owner password to comment.");
  }
});
```

Document permissions and your configuration are separate. The mode and `features` decide what
the viewer offers; the document's permissions can only restrict it further. For example, in
`annotate` mode a document that forbids commenting still can't be annotated until it is unlocked.

## Errors

| Code | When | What to do |
|---|---|---|
| `NEEDS_PASSWORD` | `init` or `load` without a password, for a file that needs one to open. | Ask for the password and call `load` again with it. |
| `WRONG_PASSWORD` | `init` or `load` with a password that doesn't open the file; `unlock` with a wrong owner password. Also sent as an `error` event when a load's owner password is wrong. | Ask again. |
| `NOT_PERMITTED` | `importXfdf` or `deleteAllAnnotations` on a document whose security forbids comments. | Unlock with the owner password, then retry. |
| `READ_ONLY` | `importXfdf` or `deleteAllAnnotations` while no annotation feature is on (for example in `view` mode). | Not a document problem: choose a mode with annotations. |
| `NO_DOCUMENT` | `unlock` or `getPermissions` before a document is open. | Open a document first. |
| `TOO_LARGE` | A password longer than 1024 characters. | |

```ts
try {
  await viewer.importXfdf(savedXfdf, { replace: true });
} catch (e) {
  if (e instanceof FreePdfError && e.code === "NOT_PERMITTED") {
    const password = await askForOwnerPassword();
    await viewer.unlock(password);
    await viewer.importXfdf(savedXfdf, { replace: true });
  } else {
    throw e;
  }
}
```

## Let users add or remove passwords

Adding or removing passwords and permissions inside the viewer is the `security` feature. It is
part of the `full` mode only; turn it on for another mode with `features`:

```ts
await FreePdf.init(el, {
  baseUrl: "/free-pdf-sdk/",
  mode: "edit",
  features: { security: true },
  document: { url: "/files/report.pdf" },
});
```

Get the protected result with `save()` or the `saveRequested` event, like any other change.
See [Configuration](/docs/configuration/) for every feature.
