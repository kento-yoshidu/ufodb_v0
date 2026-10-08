# Render a 1600x900 slide SVG to a 3200x1800 PNG with a headless browser.
# Usage: powershell -File render.ps1 -SvgPath docs\design\foo.svg [-OutPath out.png]
param(
    [Parameter(Mandatory = $true)][string]$SvgPath,
    [string]$OutPath
)

$ErrorActionPreference = 'Stop'

$svg = (Resolve-Path -LiteralPath $SvgPath).Path
if (-not $OutPath) {
    $OutPath = [System.IO.Path]::ChangeExtension($svg, '.png')
}
$out = [System.IO.Path]::GetFullPath($OutPath)

$candidates = @(
    'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe',
    'C:\Program Files\Microsoft\Edge\Application\msedge.exe',
    'C:\Program Files\Google\Chrome\Application\chrome.exe'
)
$browser = $candidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (-not $browser) {
    throw 'No Edge or Chrome found. Install one, or edit $candidates in render.ps1.'
}

# A separate profile dir keeps the headless run from touching the user's own browser profile.
$profileDir = Join-Path $env:TEMP 'ufodb-design-slide-browser'
$url = ([System.Uri]$svg).AbsoluteUri
$started = Get-Date

$browserArgs = @(
    '--headless=new',
    '--disable-gpu',
    '--hide-scrollbars',
    '--force-device-scale-factor=2',
    '--window-size=1600,900',
    "--user-data-dir=`"$profileDir`"",
    "--screenshot=`"$out`"",
    "`"$url`""
)
$p = Start-Process -FilePath $browser -ArgumentList $browserArgs -Wait -PassThru

if ($p.ExitCode -ne 0) {
    throw "Browser exited with code $($p.ExitCode)."
}
if (-not (Test-Path -LiteralPath $out) -or (Get-Item -LiteralPath $out).LastWriteTime -lt $started) {
    throw "PNG was not written: $out"
}

Write-Output "Rendered: $out"
