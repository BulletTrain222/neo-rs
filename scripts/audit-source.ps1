$ErrorActionPreference = 'Stop'
$rewriteRoot = Split-Path $PSScriptRoot -Parent
$allowed = @('.rs','.toml','.lock','.md','.txt','.py','.java','.sh','.ps1','.cmd','.bat','.json','.jsonl','.yml','.yaml','.ts','.js','.html','.css','.example','.gitignore','.gitattributes')
$violations = @()
Get-ChildItem -LiteralPath $rewriteRoot -File -Force | ForEach-Object {
    if ($_.Name -like 'LICENSE*' -or $_.Name -in @('.gitignore','.gitattributes')) { return }
    if ($_.Extension -notin $allowed) { $violations += $_.Name }
}
$roots = @('crates','scripts','docs','specs','third_party','.cargo')
foreach ($dir in $roots) {
    Get-ChildItem -LiteralPath (Join-Path $rewriteRoot $dir) -File -Recurse -Force | ForEach-Object {
        $rel = $_.FullName.Substring($rewriteRoot.Length + 1)
        if ($rel -match '(^|[\\/])(target|__pycache__|context|\.git)([\\/]|$)' -or $_.Name -eq 'kit.toml') { return }
        if ($_.Name -like 'LICENSE*' -or $_.Name -in @('kit','.gitignore','.gitattributes','.keep')) { return }
        if ($_.Extension -notin $allowed) { $violations += $rel }
    }
}
if ($violations.Count) { $violations | ForEach-Object { Write-Host "Review unexpected source file: $_" }; exit 1 }
Write-Host 'Source extension audit passed. Private data, mods, caches, and builds are excluded.'
Write-Host 'This check does not establish legal provenance or detect game bytes pasted into source.'
