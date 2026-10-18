# Tripwire for the clean-room boundary. Not a substitute for the user's review of each spec.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$problems = @()

# Specs must not carry decompiler artifacts, code, or dumps.
$specPatterns = [ordered]@{
    'decompiler name'      = '\b(FUN|DAT|LAB|PTR|SUB|thunk_FUN)_[0-9A-Fa-f]{6,}\b|\bundefined[1248]?\b'
    'MIPS instruction'     = '(?im)^\s*(?:[0-9A-Fa-f]{8}:?\s+)?(addiu|daddu|lw|sw|lq|sq|jal|jalr|jr|beq|bne|lui|ori|sll|srl|mult|lwc1|swc1|mfc1|mtc1)\s+\$?[a-z0-9]+\s*,'
    'code block'           = '(?m)^\s*```\s*(c|cpp|c\+\+|asm|mips|python|py|rust|rs)?\s*$'
    'hex dump'             = '(?m)^\s*(?:[0-9A-Fa-f]{2}\s+){16}'
    'research path'        = 'neopets-research|model-export|ghidra|decompil'
}
$skip = @('README.md', 'TEMPLATE.md', 'REQUESTS.md')
Get-ChildItem -LiteralPath (Join-Path $root 'specs') -File -Filter *.md | Where-Object { $_.Name -notin $skip } | ForEach-Object {
    $text = Get-Content -LiteralPath $_.FullName -Raw
    foreach ($k in $specPatterns.Keys) {
        if ($text -match $specPatterns[$k]) { $problems += "specs/$($_.Name): $k ('$($Matches[0].Trim())')" }
    }
}

# The implementation workspace must not depend on research crates or paths.
$codeFiles = @(Get-ChildItem -LiteralPath $root -File) + @('crates', 'scripts', '.cargo' | ForEach-Object {
    Get-ChildItem -LiteralPath (Join-Path $root $_) -File -Recurse | Where-Object { $_.FullName -notmatch '[\/]target[\/]' }
})
$codeFiles | Where-Object { $_.Extension -in @('.toml', '.rs', '.cmd', '.ps1') -and $_.Name -ne 'audit-specs.ps1' } | ForEach-Object {
    $text = Get-Content -LiteralPath $_.FullName -Raw
    if ($text -match 'ps2-oracle|ps2_oracle|neopets-research|neopets-oracle|decomp-kit') {
        $problems += "$($_.FullName.Substring($root.Length + 1)): references research-side code ('$($Matches[0])')"
    }
}

if ($problems.Count) { $problems | ForEach-Object { Write-Host "Clean-room audit: $_" }; exit 1 }
Write-Host 'Clean-room audit passed (specs and dependency references).'
