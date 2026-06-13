# Cairn installer (Windows).
#
#   irm https://cairn.sh/install.ps1 | iex
#
# Honors: $env:CAIRN_REPO, $env:CAIRN_INSTALL_DIR
$ErrorActionPreference = 'Stop'

$Repo       = if ($env:CAIRN_REPO) { $env:CAIRN_REPO } else { 'cairn-dev/cairn' }
$InstallDir = if ($env:CAIRN_INSTALL_DIR) { $env:CAIRN_INSTALL_DIR } else { "$env:LOCALAPPDATA\Cairn\bin" }
$target     = 'x86_64-pc-windows-msvc'

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Write-Host "Installing cairn ($target) -> $InstallDir"

$url = "https://github.com/$Repo/releases/latest/download/cairn-$target.zip"
try {
    $zip = Join-Path $env:TEMP 'cairn.zip'
    Invoke-WebRequest -Uri $url -OutFile $zip
    Expand-Archive -Path $zip -DestinationPath $InstallDir -Force
    Remove-Item $zip -Force
}
catch {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "No prebuilt release found; building from source with cargo…"
        cargo install --git "https://github.com/$Repo" cairn-cli
    }
    else {
        throw "No prebuilt binary available and cargo is not installed."
    }
}

# Add to the user PATH if missing.
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable('Path', "$userPath;$InstallDir", 'User')
    Write-Host "Added $InstallDir to your PATH (restart your shell)."
}

Write-Host "Done. Start the server with:  cairn serve"
