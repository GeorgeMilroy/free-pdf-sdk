// Prints the raw, gzip (level 9) and brotli (quality 11) sizes of each file given.
import { readFileSync } from "node:fs";
import { basename } from "node:path";
import { brotliCompressSync, constants, gzipSync } from "node:zlib";

const mb = (n) => `${(n / 1024 / 1024).toFixed(2)} MB`;
for (const file of process.argv.slice(2)) {
  const bytes = readFileSync(file);
  const gz = gzipSync(bytes, { level: 9 }).length;
  const br = brotliCompressSync(bytes, {
    params: { [constants.BROTLI_PARAM_QUALITY]: 11, [constants.BROTLI_PARAM_SIZE_HINT]: bytes.length },
  }).length;
  console.log(`${basename(file)}: raw ${mb(bytes.length)}, gzip ${mb(gz)}, brotli ${mb(br)}`);
}
