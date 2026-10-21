<#
.SYNOPSIS
  Builds the Free PDF SDK into dist/free-pdf-sdk-<version>/.

.DESCRIPTION
  1. cargo build of crates/viewer for wasm32-unknown-unknown (release)
  2. wasm-bindgen --target web
  3. wasm-opt -Oz (size mode)
  4. content-hashed file names, viewer/index.html rendered from the template
  5. the TypeScript wrapper (ESM + .d.ts)
  6. licences, notices, version.json { sdk, pdfcraftRev }
  7. optional zip
  It prints the raw, gzip and brotli size of the .wasm.

.PARAMETER LocalFork
  Path to a local pdfcraft clone that holds the fork commit named in Cargo.toml, for building
  before that commit is pushed. Cargo then fetches https://github.com/GeorgeMilroy/pdfcraft from
  that clone (process-scoped git URL rewriting; nothing global is changed).

.PARAMETER Zip
  Also write dist/free-pdf-sdk-<version>.zip.

.PARAMETER Jobs
  Cargo build jobs (default 2: a full parallel build has locked up this machine before).
#>
[CmdletBinding()]
param(
  [string]$LocalFork = "",
  [switch]$Zip,
  [switch]$SkipWasmOpt,
  [int]$Jobs = 2
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo

function Invoke-Checked([string]$what, [scriptblock]$block) {
  Write-Host "==> $what" -ForegroundColor Cyan
  # Windows PowerShell 5.1 turns a native tool's stderr (cargo's progress) into errors under
  # "Stop"; the exit code is what decides.
  $saved = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try { & $block } finally { $ErrorActionPreference = $saved }
  if ($LASTEXITCODE -ne 0) { throw "$what failed (exit $LASTEXITCODE)" }
}

. (Join-Path $PSScriptRoot "dev-env.ps1") -LocalFork $LocalFork

# --- versions -----------------------------------------------------------------------------------
$cargoToml = Get-Content (Join-Path $repo "Cargo.toml") -Raw
if ($cargoToml -notmatch '(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"') { throw "no [workspace.package] version in Cargo.toml" }
$version = $Matches[1]
$jsPkg = Get-Content (Join-Path $repo "js/package.json") -Raw | ConvertFrom-Json
if ($jsPkg.version -ne $version) { throw "js/package.json version $($jsPkg.version) differs from Cargo.toml $version" }
$lock = Get-Content (Join-Path $repo "Cargo.lock") -Raw
if ($lock -notmatch '(?ms)name = "pdfcraft-engine"\s*\r?\nversion = "[^"]+"\s*\r?\nsource = "git\+[^#"]+#([0-9a-f]{40})"') { throw "Cargo.lock has no git source for pdfcraft-engine" }
$pdfcraftRev = $Matches[1]
Write-Host "Free PDF SDK $version, pdfcraft $pdfcraftRev"

$out = Join-Path $repo "dist/free-pdf-sdk-$version"
$work = Join-Path $repo "build"
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
if (Test-Path $work) { Remove-Item -Recurse -Force $work }
New-Item -ItemType Directory -Force (Join-Path $out "viewer") | Out-Null
New-Item -ItemType Directory -Force $work | Out-Null

# --- wasm ---------------------------------------------------------------------------------------
Invoke-Checked "cargo build (wasm32, release)" { cargo build -p fps-viewer --target wasm32-unknown-unknown --release --locked -j $Jobs }
$bindgenVersion = (wasm-bindgen --version) -replace '^wasm-bindgen\s+', ''
if ($lock -notmatch "(?ms)name = `"wasm-bindgen`"\s*\r?\nversion = `"$([regex]::Escape($bindgenVersion))`"") {
  throw "wasm-bindgen-cli $bindgenVersion does not match Cargo.lock: cargo install wasm-bindgen-cli --version <locked> --locked"
}
$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $repo "target" }
$wasmBuilt = Join-Path $targetDir "wasm32-unknown-unknown/release/fps-viewer.wasm"
Invoke-Checked "wasm-bindgen" { wasm-bindgen $wasmBuilt --out-dir $work --out-name fps-viewer --target web --no-typescript }

$wasmIn = Join-Path $work "fps-viewer_bg.wasm"
$rawBefore = (Get-Item $wasmIn).Length
if (-not $SkipWasmOpt) {
  $wasmOpt = Join-Path $repo "node_modules/.bin/wasm-opt.cmd"
  if (-not (Test-Path $wasmOpt)) { $wasmOpt = Join-Path $repo "node_modules/.bin/wasm-opt" }
  if (-not (Test-Path $wasmOpt)) { throw "wasm-opt not found: run npm install in the repository root" }
  $opt = Join-Path $work "fps-viewer_opt.wasm"
  # The features rustc enables by default for wasm32-unknown-unknown.
  Invoke-Checked "wasm-opt -Oz" { & $wasmOpt $wasmIn -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue -o $opt }
  Move-Item -Force $opt $wasmIn
}

function Get-ShortHash([string]$path) { (Get-FileHash -Algorithm SHA256 $path).Hash.ToLowerInvariant().Substring(0, 10) }
$wasmHash = Get-ShortHash $wasmIn
$wasmName = "fps-viewer-$wasmHash" + "_bg.wasm"
Copy-Item $wasmIn (Join-Path $out "viewer/$wasmName")
$glueIn = Join-Path $work "fps-viewer.js"
$glueHash = Get-ShortHash $glueIn
$glueName = "fps-viewer-$glueHash.js"
Copy-Item $glueIn (Join-Path $out "viewer/$glueName")

$template = Get-Content (Join-Path $repo "viewer/index.html") -Raw
$page = $template.Replace("{{GLUE_JS}}", $glueName).Replace("{{WASM}}", $wasmName)
if ($page -match "\{\{") { throw "viewer/index.html has an unknown placeholder" }
[System.IO.File]::WriteAllText((Join-Path $out "viewer/index.html"), $page, (New-Object System.Text.UTF8Encoding $false))

# --- TypeScript wrapper -------------------------------------------------------------------------
Push-Location (Join-Path $repo "js")
try {
  if (-not (Test-Path "node_modules")) { Invoke-Checked "npm ci (js)" { npm ci --no-audit --no-fund } }
  # tsc never deletes outputs of removed sources: start clean.
  if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" }
  Invoke-Checked "tsc (js)" { npm run build }
} finally { Pop-Location }
Copy-Item -Recurse (Join-Path $repo "js/dist/*") $out

# --- licences, notices, version -----------------------------------------------------------------
foreach ($f in @("LICENSE-MIT", "LICENSE-APACHE", "NOTICE", "README.md", "CHANGELOG.md")) {
  if (Test-Path (Join-Path $repo $f)) { Copy-Item (Join-Path $repo $f) $out }
}
if (Test-Path (Join-Path $repo "third-party")) { Copy-Item -Recurse (Join-Path $repo "third-party") (Join-Path $out "third-party") }
$versionJson = [ordered]@{ sdk = $version; pdfcraftRev = $pdfcraftRev } | ConvertTo-Json
[System.IO.File]::WriteAllText((Join-Path $out "version.json"), $versionJson + "`n", (New-Object System.Text.UTF8Encoding $false))

# --- npm package (dist/npm): the wrapper, its integrations, the CLI and the viewer files --------
$npm = Join-Path $repo "dist/npm"
if (Test-Path $npm) { Remove-Item -Recurse -Force $npm }
New-Item -ItemType Directory -Force (Join-Path $npm "dist") | Out-Null
Copy-Item -Recurse (Join-Path $repo "js/dist/*") (Join-Path $npm "dist")
Copy-Item -Recurse (Join-Path $repo "js/bin") (Join-Path $npm "bin")
foreach ($f in @("js/package.json", "js/README.md")) { Copy-Item (Join-Path $repo $f) $npm }
Copy-Item -Recurse (Join-Path $out "viewer") (Join-Path $npm "viewer")
foreach ($f in @("version.json", "NOTICE", "LICENSE-MIT", "LICENSE-APACHE")) { Copy-Item (Join-Path $out $f) $npm }
Copy-Item -Recurse (Join-Path $out "third-party") (Join-Path $npm "third-party")
Write-Host "npm package staged: $npm (publish with: npm publish $npm)"

# --- sizes --------------------------------------------------------------------------------------
Write-Host "wasm before wasm-opt: $([math]::Round($rawBefore / 1MB, 2)) MB"
Invoke-Checked "sizes" { node (Join-Path $PSScriptRoot "sizes.mjs") (Join-Path $out "viewer/$wasmName") (Join-Path $out "viewer/$glueName") }

if ($Zip) {
  $zipPath = Join-Path $repo "dist/free-pdf-sdk-$version.zip"
  if (Test-Path $zipPath) { Remove-Item -Force $zipPath }
  # .NET's ZipFile writes '/' separators (Compress-Archive 1.0 on Windows PowerShell writes backslashes).
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  [System.IO.Compression.ZipFile]::CreateFromDirectory($out, $zipPath, [System.IO.Compression.CompressionLevel]::Optimal, $true)
  Write-Host "zip: $zipPath"
}
Write-Host "built $out" -ForegroundColor Green
