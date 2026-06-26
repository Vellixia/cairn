#Requires -Version 5.1
. "$PSScriptRoot/_lib.ps1"

# Sprint 8-10: CLI subcommands — doctor, bench, setup, mcp, sync, export/import.
# Sprint 0.6.5: expanded to cover setup, onboard, status, reset.

# --- doctor ---
$out = Invoke-CairnCli doctor
Assert-Contains -Haystack $out -Needle 'ok' -Msg 'cairn doctor runs'

# --- status ---
$out2 = Invoke-CairnCli status
Assert-Contains -Haystack $out2 -Needle 'Cairn client v0.6.5' -Msg 'cairn status shows version 0.6.5'
Assert-Contains -Haystack $out2 -Needle 'server' -Msg 'cairn status shows server'

# --- reset --dry-run ---
$out3 = Invoke-CairnCli reset --dry-run
Assert-Contains -Haystack $out3 -Needle 'Would remove' -Msg 'cairn reset --dry-run reports what would be removed'

# --- setup claude-code in a temp project dir ---
$tmpProject = Join-Path $Global:E2E_DataDir 'e2e-setup-test'
New-Item -ItemType Directory -Force -Path $tmpProject | Out-Null
$origDir = Get-Location
try {
    Set-Location $tmpProject
    $out4 = Invoke-CairnCli setup claude-code 2>&1
    Assert-Contains -Haystack $out4 -Needle 'Configured Claude Code' -Msg 'cairn setup claude-code says Configured'
    Assert-Contains -Haystack $out4 -Needle 'Run /mcp in Claude Code' -Msg 'cairn setup prints /mcp approval guidance'

    # Verify .mcp.json has absolute path.
    $mcpPath = Join-Path $tmpProject '.mcp.json'
    Assert-True -Condition (Test-Path $mcpPath) -Msg '.mcp.json was created'
    $mcp = Get-Content $mcpPath -Raw | ConvertFrom-Json
    $cmd = $mcp.mcpServers.cairn.command
    Assert-True -Condition ($cmd -match 'cairn\.exe') -Msg ".mcp.json command contains cairn.exe (got $cmd)"
    Assert-True -Condition ($cmd -match '\\\\|/') -Msg ".mcp.json command is an absolute path (got $cmd)"

    # Re-run — must be idempotent (no extra entries).
    $out5 = Invoke-CairnCli setup claude-code 2>&1
    Assert-Contains -Haystack $out5 -Needle 'Configured Claude Code' -Msg 're-run setup claude-code is ok'
} finally {
    Set-Location $origDir
    Remove-Item -Recurse -Force $tmpProject -ErrorAction SilentlyContinue
}

# --- onboard --skip-agents (first run in fresh env) ---
$tmpOnboard = Join-Path $Global:E2E_DataDir 'e2e-onboard-test'
New-Item -ItemType Directory -Force -Path $tmpOnboard | Out-Null
try {
    # Use a temp XDG_CONFIG_HOME so we appear fresh.
    $env:CAIRN_ONBOARD_DATA = $tmpOnboard
    $out6 = Invoke-CairnCli onboard --skip-agents
    Assert-Contains -Haystack $out6 -Needle 'onboard' -Msg 'cairn onboard --skip-agents prints onboard header'
    Assert-Contains -Haystack $out6 -Needle 'doctor: green' -Msg 'cairn onboard reports doctor green'
} finally {
    Remove-Item -Recurse -Force $tmpOnboard -ErrorAction SilentlyContinue
}

Show-Scenario -Sprint '4.0' -Name 'prompts-cli' -Status pass
