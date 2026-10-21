#!/usr/bin/env node
// Free PDF SDK command line: copies the viewer files (viewer/, version.json and the licences)
// from this package into a folder your web server serves, such as public/free-pdf-sdk.
//
//   npx @mecorelabs/free-pdf-sdk copy-viewer public/free-pdf-sdk
//
// Run it after every install or upgrade (for example in a "postinstall" or "prebuild" script),
// so the viewer always matches the wrapper version.

import { cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const pkg = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const [command, target] = process.argv.slice(2);

function usage(code) {
  console.log("Usage: free-pdf-sdk copy-viewer <folder>\n\nCopies the viewer files into <folder>; then use that folder's URL as baseUrl.");
  process.exit(code);
}

if (command === "--help" || command === "-h") usage(0);
if (command !== "copy-viewer" || !target) usage(1);

const source = join(pkg, "viewer");
if (!existsSync(source)) {
  console.error(`free-pdf-sdk: ${source} is missing; reinstall the package`);
  process.exit(1);
}
const dest = resolve(process.cwd(), target);
mkdirSync(dest, { recursive: true });
// A new version has new hashed file names: start clean so old ones don't pile up.
rmSync(join(dest, "viewer"), { recursive: true, force: true });
cpSync(source, join(dest, "viewer"), { recursive: true });
for (const file of ["version.json", "NOTICE", "LICENSE-MIT", "LICENSE-APACHE"]) {
  if (existsSync(join(pkg, file))) cpSync(join(pkg, file), join(dest, file));
}
if (existsSync(join(pkg, "third-party"))) cpSync(join(pkg, "third-party"), join(dest, "third-party"), { recursive: true });
console.log(`free-pdf-sdk: copied the viewer to ${dest}`);
