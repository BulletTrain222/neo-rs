param([string]$Iso)
$ErrorActionPreference = 'Stop'
$rewriteRoot = Split-Path $PSScriptRoot -Parent
if (-not $Iso) { $Iso = Read-Host 'Path to your Neopets USA ISO (or drag the ISO here)' }
$Iso = $Iso.Trim().Trim('"')
$resolvedIso = (Resolve-Path -LiteralPath $Iso).Path
if ((Get-Item -LiteralPath $resolvedIso).PSIsContainer -or [IO.Path]::GetExtension($resolvedIso) -ine '.iso') {
    throw 'Select an uncompressed .iso file.'
}
New-Item -ItemType Directory -Force -Path (Join-Path $rewriteRoot 'private') | Out-Null
[IO.File]::WriteAllText((Join-Path $rewriteRoot 'private/iso-path.txt'), $resolvedIso, [Text.UTF8Encoding]::new($false))
Write-Host 'Saved the local path. The ISO was not copied or modified. Run Inspect-ISO.cmd to validate it.'
