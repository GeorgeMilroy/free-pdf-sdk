<#
.SYNOPSIS
  Cuts a Free PDF SDK release locally: checks, builds the zip, tags. Never pushes.

.DESCRIPTION
  1. the working tree is clean and on main
  2. Cargo.toml [workspace.package] version, js/package.json version and the wrapper's
     SDK_VERSION agree with -Version, and CHANGELOG.md has a "## [<version>]" section
  3. fmt, clippy, tests, the wasm build (with -Zip) and the e2e tests pass
  4. creates the annotated tag v<version>
  Then it prints the push commands and where the zip is.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$Version,
  [string]$LocalFork = "",
  [int]$Jobs = 2,
  [switch]$SkipE2e
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo

function Run([string]$what, [scriptblock]$block) {
  Write-Host "==> $what" -ForegroundColor Cyan
  $saved = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try { & $block } finally { $ErrorActionPreference = $saved }
  if ($LASTEXITCODE -ne 0) { throw "FAILED: $what (exit $LASTEXITCODE)" }
}

if ($Version -notmatch '^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$') { throw "version $Version is not semver" }
if ((git status --porcelain | Out-String).Trim() -ne "") { throw "the working tree has uncommitted changes" }
$branch = (git rev-parse --abbrev-ref HEAD).Trim()
if ($branch -ne "main") { throw "release from main (on $branch)" }
if ((git tag --list "v$Version" | Out-String).Trim() -ne "") { throw "tag v$Version exists" }

$cargo = Get-Content Cargo.toml -Raw
if ($cargo -notmatch "(?ms)^\[workspace\.package\].*?^version\s*=\s*`"$([regex]::Escape($Version))`"") { throw "Cargo.toml [workspace.package] version is not $Version" }
$pkg = Get-Content js/package.json -Raw | ConvertFrom-Json
if ($pkg.version -ne $Version) { throw "js/package.json version is $($pkg.version), not $Version" }
if ((Get-Content js/src/free-pdf-sdk.ts -Raw) -notmatch "SDK_VERSION = `"$([regex]::Escape($Version))`"") { throw "js/src/free-pdf-sdk.ts SDK_VERSION is not $Version" }
if ((Get-Content CHANGELOG.md -Raw) -notmatch "(?m)^## \[$([regex]::Escape($Version))\]") { throw "CHANGELOG.md has no ## [$Version] section" }

. (Join-Path $PSScriptRoot "dev-env.ps1") -LocalFork $LocalFork
Run "cargo fmt --check" { cargo fmt --all --check }
Run "cargo clippy (native)" { cargo clippy -p fps-core -p fps-brand -p fps-tests --all-targets --locked -j $Jobs -- -D warnings }
Run "cargo clippy (wasm)" { cargo clippy -p fps-viewer --target wasm32-unknown-unknown --locked -j $Jobs -- -D warnings }
Run "cargo test" { cargo test -p fps-core -p fps-brand -p fps-tests --locked -j $Jobs }
Run "typecheck" { npm --prefix js run typecheck }
Run "build" { powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 -LocalFork $LocalFork -Jobs $Jobs -Zip }
if (-not $SkipE2e) { Run "e2e" { npx playwright test -c e2e/playwright.config.ts } }

Run "tag v$Version" { git tag -a "v$Version" -m "Free PDF SDK $Version" }
Write-Host "`nReleased v$Version locally: dist/free-pdf-sdk-$Version/ and dist/free-pdf-sdk-$Version.zip" -ForegroundColor Green
Write-Host @"
Push when ready (the pdfcraft fork commit in Cargo.lock must be pushed first):

  git push origin main
  git push origin v$Version

"@
