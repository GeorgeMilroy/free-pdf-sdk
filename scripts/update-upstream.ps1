<#
.SYNOPSIS
  Brings a new pdfcraft upstream into Free PDF SDK. Stops at the first failure; never pushes.

.DESCRIPTION
  1. git fetch upstream (and origin) in the pdfcraft clone
  2. fork main: fast-forward to upstream/main, or merge upstream/main when the fork has its own
     commits (local only; stops on a conflict)
  3. rebase free-pdf-sdk/patches onto upstream/main (in its worktree; stops on a conflict)
  4. scoped pdfcraft tests: cargo fmt --check, clippy and test for the patched crates (-j 2)
  5. tag the new patch-branch commit fps-upstream-<yyyymmdd>[-n] so it stays reachable after the
     branch is force-pushed
  6. bump the pdfcraft revs in Cargo.toml (upstream base and fork patch commit)
  7. cargo update -p for the pdfcraft crates, then this repo's whole test suite and wasm build
  Then it prints the exact push commands for you to run.

.PARAMETER Pdfcraft
  The pdfcraft clone (remotes: origin = your fork, upstream = storytold/pdfcraft).

.PARAMETER Worktree
  The worktree of free-pdf-sdk/patches (created if missing).

.PARAMETER SkipPdfcraftTests
  Skip step 4 (not recommended).
#>
[CmdletBinding()]
param(
  [string]$Pdfcraft = "D:\Developments\storytold\pdfcraft",
  [string]$Worktree = "D:\Developments\storytold\pdfcraft-fps",
  [string]$PatchBranch = "free-pdf-sdk/patches",
  [string[]]$PatchedCrates = @("pdfcraft-ui-egui"),
  [switch]$SkipPdfcraftTests,
  [int]$Jobs = 2
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$repo = Split-Path -Parent $PSScriptRoot

function Step([string]$text) { Write-Host "`n==> $text" -ForegroundColor Cyan }
function Run([string]$what, [scriptblock]$block) {
  $saved = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try { & $block } finally { $ErrorActionPreference = $saved }
  if ($LASTEXITCODE -ne 0) { throw "FAILED: $what (exit $LASTEXITCODE). Nothing was pushed; fix it and run again." }
}
function Git([string]$dir) { $args2 = $args; Run "git $($args2 -join ' ')" { git -C $dir @args2 } }
function GitOut([string]$dir) {
  $args2 = $args
  $saved = $ErrorActionPreference; $ErrorActionPreference = "Continue"
  try { $out = git -C $dir @args2 } finally { $ErrorActionPreference = $saved }
  if ($LASTEXITCODE -ne 0) { throw "FAILED: git $($args2 -join ' ')" }
  return ($out | Out-String).Trim()
}

if (-not (Test-Path (Join-Path $Pdfcraft ".git"))) { throw "$Pdfcraft is not a git clone" }
if ((GitOut $repo status --porcelain) -ne "") { throw "this repository has uncommitted changes: commit or stash them first" }

# 1 -------------------------------------------------------------------------------------------
Step "1. fetch upstream and origin"
Git $Pdfcraft fetch upstream --tags
Git $Pdfcraft fetch origin
$upstream = GitOut $Pdfcraft rev-parse upstream/main
Write-Host "upstream/main is $upstream"

# 2 -------------------------------------------------------------------------------------------
Step "2. bring the fork's main up to upstream/main (local only)"
$mainWorktree = $null
foreach ($line in (GitOut $Pdfcraft worktree list --porcelain) -split "\r?\n") {
  if ($line -match '^worktree (.+)$') { $wt = $Matches[1].Trim() }
  if ($line -match '^branch refs/heads/main$') { $mainWorktree = $wt }
}
$mainOld = GitOut $Pdfcraft rev-parse main
$saved = $ErrorActionPreference; $ErrorActionPreference = "Continue"
git -C $Pdfcraft merge-base --is-ancestor main upstream/main
$canFastForward = ($LASTEXITCODE -eq 0)
$ErrorActionPreference = $saved
if ($canFastForward) {
  if ($mainWorktree) { Git $mainWorktree merge --ff-only upstream/main }
  else { Git $Pdfcraft branch -f main upstream/main }
  Write-Host "main fast-forwarded to $upstream"
} else {
  if (-not $mainWorktree) { throw "main has its own commits and is not checked out anywhere: check it out in a worktree, then run again" }
  if ((GitOut $mainWorktree status --porcelain) -ne "") { throw "$mainWorktree (main) has uncommitted changes" }
  Git $mainWorktree merge --no-edit upstream/main
  Write-Host "merged upstream/main into main (the fork has its own commits)"
}
$mainNew = GitOut $Pdfcraft rev-parse main

# 3 -------------------------------------------------------------------------------------------
Step "3. rebase $PatchBranch onto upstream/main"
if (-not (Test-Path $Worktree)) { Git $Pdfcraft worktree add $Worktree $PatchBranch }
if ((GitOut $Worktree status --porcelain) -ne "") { throw "$Worktree has uncommitted changes" }
$head = GitOut $Worktree rev-parse --abbrev-ref HEAD
if ($head -ne $PatchBranch) { throw "$Worktree is on '$head', not $PatchBranch (finish or abort any rebase in progress first)" }
$patchOld = GitOut $Worktree rev-parse HEAD
# The lease for the force-push: what the fork on GitHub holds now (nothing on a first push).
$saved = $ErrorActionPreference; $ErrorActionPreference = "Continue"
$remoteOld = (git -C $Pdfcraft rev-parse --verify --quiet "refs/remotes/origin/$PatchBranch" | Out-String).Trim()
$ErrorActionPreference = $saved
$count = GitOut $Worktree rev-list --count "upstream/main..HEAD"
Write-Host "rebasing $count patch commit(s)"
$saved = $ErrorActionPreference; $ErrorActionPreference = "Continue"
git -C $Worktree rebase upstream/main
$rebased = ($LASTEXITCODE -eq 0)
$ErrorActionPreference = $saved
if (-not $rebased) {
  throw "FAILED: the rebase stopped on a conflict in $Worktree. Resolve it there (git status; edit; git add; git rebase --continue), or git rebase --abort, then run this script again."
}
$patch = GitOut $Worktree rev-parse HEAD
Write-Host "patch branch: $patchOld -> $patch"

# 4 -------------------------------------------------------------------------------------------
if (-not $SkipPdfcraftTests) {
  Step "4. scoped pdfcraft tests ($($PatchedCrates -join ', '))"
  Push-Location $Worktree
  try {
    foreach ($crate in $PatchedCrates) {
      Run "cargo fmt -p $crate --check" { cargo fmt -p $crate --check }
      Run "cargo clippy -p $crate" { cargo clippy -p $crate -j $Jobs --all-targets -- -D warnings }
      Run "cargo test -p $crate" { cargo test -p $crate -j $Jobs }
    }
  } finally { Pop-Location }
} else {
  Write-Warning "4. skipped the pdfcraft tests"
}

# 5 -------------------------------------------------------------------------------------------
Step "5. tag the patch commit"
$date = Get-Date -Format "yyyyMMdd"
$tag = "fps-upstream-$date"
$n = 1
while ((GitOut $Pdfcraft tag --list $tag) -ne "") { $n++; $tag = "fps-upstream-$date-$n" }
Git $Pdfcraft tag -a $tag $patch -m "Free PDF SDK patches on upstream $upstream"
Write-Host "tagged $patch as $tag"

# 6 -------------------------------------------------------------------------------------------
Step "6. bump the pdfcraft revs in Cargo.toml"
$cargoToml = Join-Path $repo "Cargo.toml"
$text = Get-Content $cargoToml -Raw
$text = [regex]::Replace($text, '(git = "https://github\.com/storytold/pdfcraft", rev = ")[0-9a-f]{40}(")', "`${1}$upstream`${2}")
$text = [regex]::Replace($text, '(git = "https://github\.com/GeorgeMilroy/pdfcraft", rev = ")[0-9a-f]{40}(")', "`${1}$patch`${2}")
[System.IO.File]::WriteAllText($cargoToml, $text, (New-Object System.Text.UTF8Encoding $false))
$notice = Join-Path $repo "NOTICE"
$noticeText = Get-Content $notice -Raw
$noticeText = [regex]::Replace($noticeText, '(?m)^(\s+)[0-9a-f]{40}( \(pdfcraft [^)]*\))?\s*$', "`${1}$upstream (pdfcraft upstream)")
[System.IO.File]::WriteAllText($notice, $noticeText, (New-Object System.Text.UTF8Encoding $false))

# 7 -------------------------------------------------------------------------------------------
Step "7. cargo update and the whole test suite"
. (Join-Path $PSScriptRoot "dev-env.ps1") -LocalFork $Pdfcraft
Push-Location $repo
try {
  Run "cargo update (pdfcraft)" { cargo update -p pdfcraft-engine -p pdfcraft-ui-egui }
  Run "collect notices" { node scripts/collect-notices.mjs --pdfcraft $Pdfcraft }
  Run "cargo fmt --check" { cargo fmt --all --check }
  Run "cargo clippy (native)" { cargo clippy -p fps-core -p fps-brand -p fps-tests --all-targets -j $Jobs -- -D warnings }
  Run "cargo clippy (wasm)" { cargo clippy -p fps-viewer --target wasm32-unknown-unknown -j $Jobs -- -D warnings }
  Run "cargo test" { cargo test -p fps-core -p fps-brand -p fps-tests -j $Jobs }
  # The PowerShell running this script (powershell.exe on Windows, pwsh elsewhere).
  $ps = (Get-Process -Id $PID).Path
  Run "build" { & $ps -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "build.ps1") -LocalFork $Pdfcraft -Jobs $Jobs }
  Run "e2e" { npx playwright test -c e2e/playwright.config.ts }
} finally { Pop-Location }

$pushPatch = if ($remoteOld) { "git -C `"$Pdfcraft`" push --force-with-lease=${PatchBranch}:$remoteOld origin ${PatchBranch}" } else { "git -C `"$Pdfcraft`" push -u origin ${PatchBranch}" }
Write-Host "`nAll green. Review the changes (git diff), commit them, then push:" -ForegroundColor Green
Write-Host @"

  # the fork (the rebase rewrote the patch branch: the lease is what GitHub holds now)
  $pushPatch
  git -C "$Pdfcraft" push origin $tag
  git -C "$Pdfcraft" push origin main          # main: $mainOld -> $mainNew

  # this repository, after committing Cargo.toml, Cargo.lock, NOTICE and third-party/
  git -C "$repo" push origin HEAD

"@
