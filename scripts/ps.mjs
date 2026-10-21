// Runs a PowerShell script on any OS, for npm scripts: PowerShell 7 (pwsh) where it is
// installed, otherwise Windows PowerShell.
// Usage: node scripts/ps.mjs <script.ps1> [arguments...]
import { spawnSync } from "node:child_process";

const args = ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File", ...process.argv.slice(2)];
for (const shell of ["pwsh", "powershell"]) {
  const run = spawnSync(shell, args, { stdio: "inherit" });
  if (run.error && run.error.code === "ENOENT") continue;
  process.exit(run.status ?? 1);
}
console.error("PowerShell was not found. Install PowerShell 7 (pwsh): https://aka.ms/powershell");
process.exit(1);
