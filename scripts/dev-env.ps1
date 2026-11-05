<#
.SYNOPSIS
  Sets up this PowerShell process for building Free PDF SDK: pdfcraft's interface translations,
  and optionally a local pdfcraft clone for the fork commit.

.DESCRIPTION
  Dot-source it: `. scripts/dev-env.ps1 [-LocalFork D:\Developments\storytold\pdfcraft]`.

  Translations: pdfcraft embeds its interface catalogs from a checkout of storytold/craft-i18n
  named by CRAFT_I18N_DIR; without one the viewer is English only. This pins the checkout to the
  commit in craft-i18n.rev (the one the pinned pdfcraft names) and sets CRAFT_I18N_REQUIRED=1, so
  a build without the translations fails instead. An existing CRAFT_I18N_DIR is used when it is at
  that commit (CI checks it out there); otherwise target/craft-i18n is cloned or moved to it.

  -LocalFork: CARGO_NET_GIT_FETCH_WITH_CLI=true and a git
  `url.<local>.insteadOf=https://github.com/GeorgeMilroy/pdfcraft` rewrite through the
  GIT_CONFIG_COUNT/GIT_CONFIG_KEY_n/GIT_CONFIG_VALUE_n variables, for this process only.
  Cargo.toml and Cargo.lock keep the real GitHub URL and commit, so nothing changes once the
  commit is pushed. With no -LocalFork (or an empty one) it leaves git alone.
#>
param([string]$LocalFork = "")

$devEnvRepo = Split-Path -Parent $PSScriptRoot

# Translations ------------------------------------------------------------------------------
$i18nRev = (Get-Content (Join-Path $devEnvRepo "craft-i18n.rev") -Raw).Trim()
if ($i18nRev -notmatch '^[0-9a-f]{40}$') { throw "craft-i18n.rev must hold one 40-hex commit" }
$i18nGit = (Get-Command git -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
function Get-I18nHead([string]$dir) {
  if (-not (Test-Path (Join-Path $dir ".git"))) { return "" }
  $saved = $ErrorActionPreference; $ErrorActionPreference = "Continue"
  try { $out = & $i18nGit -C $dir rev-parse HEAD 2>$null } finally { $ErrorActionPreference = $saved }
  return ($out | Out-String).Trim()
}
$i18nDir = $env:CRAFT_I18N_DIR
if (-not $i18nDir -or (Get-I18nHead $i18nDir) -ne $i18nRev) {
  $i18nDir = Join-Path (Join-Path $devEnvRepo "target") "craft-i18n"
  $saved = $ErrorActionPreference; $ErrorActionPreference = "Continue"
  try {
    if (-not (Test-Path (Join-Path $i18nDir ".git"))) {
      & $i18nGit clone --quiet https://github.com/storytold/craft-i18n $i18nDir
      if ($LASTEXITCODE -ne 0) { throw "cloning storytold/craft-i18n failed" }
    }
    if ((Get-I18nHead $i18nDir) -ne $i18nRev) {
      & $i18nGit -C $i18nDir cat-file -e "$i18nRev^{commit}" 2>$null
      if ($LASTEXITCODE -ne 0) { & $i18nGit -C $i18nDir fetch --quiet origin }
      & $i18nGit -C $i18nDir -c advice.detachedHead=false checkout --quiet $i18nRev
      if ($LASTEXITCODE -ne 0) { throw "checking out craft-i18n $i18nRev failed" }
    }
  } finally { $ErrorActionPreference = $saved }
}
# Absolute, not canonicalized: pdfcraft's build script needs an absolute path, and include_str!
# can't read Windows' \\?\ form.
$env:CRAFT_I18N_DIR = (Resolve-Path $i18nDir).ProviderPath
$env:CRAFT_I18N_REQUIRED = "1"
Write-Host "Translations from craft-i18n $($i18nRev.Substring(0, 8)) in $env:CRAFT_I18N_DIR"

# Local fork --------------------------------------------------------------------------------
if ($LocalFork) {
  $full = (Resolve-Path $LocalFork).Path
  if (-not (Test-Path (Join-Path $full ".git"))) { throw "$full is not a git clone" }
  $url = "file:///" + ($full -replace '\\', '/')
  $env:CARGO_NET_GIT_FETCH_WITH_CLI = "true"
  $env:GIT_CONFIG_COUNT = "1"
  $env:GIT_CONFIG_KEY_0 = "url.$url.insteadOf"
  $env:GIT_CONFIG_VALUE_0 = "https://github.com/GeorgeMilroy/pdfcraft"
  Write-Host "Cargo fetches https://github.com/GeorgeMilroy/pdfcraft from $url (this process only)"
}
