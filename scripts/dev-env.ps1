<#
.SYNOPSIS
  Lets Cargo fetch the pdfcraft fork commit from a local clone before it is pushed.

.DESCRIPTION
  Dot-source it: `. scripts/dev-env.ps1 -LocalFork D:\Developments\storytold\pdfcraft`.
  It sets, for this PowerShell process only, CARGO_NET_GIT_FETCH_WITH_CLI=true and a git
  `url.<local>.insteadOf=https://github.com/GeorgeMilroy/pdfcraft` rewrite through the
  GIT_CONFIG_COUNT/GIT_CONFIG_KEY_n/GIT_CONFIG_VALUE_n variables. Cargo.toml and Cargo.lock keep
  the real GitHub URL and commit, so nothing changes once the commit is pushed.
  With no -LocalFork (or an empty one) it does nothing.
#>
param([string]$LocalFork = "")

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
