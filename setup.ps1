$destPath = "$env:APPDATA\adsb\adsb-viewer.db"

$url = "https://github.com/Lunna5/air-database/releases/download/0.1.0/output.db"

$folder = Split-Path -Parent $destPath
if (-not (Test-Path $folder)) {
    New-Item -ItemType Directory -Path $folder -Force
}

Invoke-WebRequest -Uri $url -OutFile $destPath

Write-Host "Archivo descargado correctamente a: $destPath"