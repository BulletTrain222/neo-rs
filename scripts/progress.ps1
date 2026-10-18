# Prints project progress. See docs/PROGRESS.md for what each number means.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent

function Bar([double]$pct) {
    $filled = [int][Math]::Round([Math]::Max(0, [Math]::Min(100, $pct)) / 5)
    '[' + ('#' * $filled) + ('.' * (20 - $filled)) + ']'
}
function Pct([double]$part, [double]$whole) { if ($whole -le 0) { 0.0 } else { 100.0 * $part / $whole } }
function TableRows([string]$path) {
    # Markdown table rows as string arrays, skipping header and separator lines.
    if (-not (Test-Path -LiteralPath $path)) { return @() }
    $rows = @()
    foreach ($line in Get-Content -LiteralPath $path) {
        if ($line -notmatch '^\s*\|' -or $line -match '^\s*\|\s*-') { continue }
        $cells = @($line.Trim().Trim('|').Split('|') | ForEach-Object { $_.Trim() })
        $rows += ,$cells
    }
    $rows
}
function Line([string]$label, [double]$pct, [string]$detail) {
    Write-Host ('  {0,-14} {1} {2,5:N1}%  {3}' -f $label, (Bar $pct), $pct, $detail)
}

Write-Host ''
Write-Host 'Neopets clean-room rewrite: progress'
Write-Host ''

# 1. Code coverage, from the research side's aggregate report.
Write-Host 'Code coverage (share of original game code recreated, by bytes)'
$progressPath = Join-Path $root 'specs/PROGRESS.md'
$stages = @{}
foreach ($r in (TableRows $progressPath)) {
    if ($r.Count -ge 3 -and $r[0] -in @('Total', 'Analyzed', 'Specced', 'Implemented', 'Verified')) {
        $stages[$r[0]] = @{ Functions = [double]($r[1] -replace '[^0-9]', ''); Bytes = [double]($r[2] -replace '[^0-9]', '') }
    }
}
if ($stages.ContainsKey('Total') -and $stages['Total'].Bytes -gt 0) {
    $asOf = (Select-String -LiteralPath $progressPath -Pattern '^- As of:\s*(.+)$' | Select-Object -First 1)
    $scope = (Select-String -LiteralPath $progressPath -Pattern '^- Coverage of list:\s*(.+)$' | Select-Object -First 1)
    $total = $stages['Total']
    foreach ($s in @('Analyzed', 'Specced', 'Implemented', 'Verified')) {
        if ($stages.ContainsKey($s)) {
            Line $s (Pct $stages[$s].Bytes $total.Bytes) ('{0:N0} of {1:N0} functions' -f $stages[$s].Functions, $total.Functions)
        }
    }
    if ($asOf) { Write-Host "  Report as of $($asOf.Matches[0].Groups[1].Value)" }
    if ($scope) { Write-Host "  Function list: $($scope.Matches[0].Groups[1].Value)" }
    if ($stages.ContainsKey('Verified')) {
        Write-Host ('  Headline: {0:N1}% of the original code is recreated and verified.' -f (Pct $stages['Verified'].Bytes $total.Bytes))
    }
} else {
    Write-Host '  No report yet. The research side delivers specs/PROGRESS.md (request R-0000).'
}
Write-Host ''

# 2. Asset coverage, from the clean side's own disc inspector.
Write-Host 'Asset coverage (disc resources our code can decode)'
$assetPath = Join-Path $root 'private/asset-coverage.txt'
if (Test-Path -LiteralPath $assetPath) {
    $sumDone = 0.0; $sumAll = 0.0
    foreach ($line in Get-Content -LiteralPath $assetPath) {
        if ($line -match '^\s*(\S+)\s+(\d+)\s+(\d+)\s*$') {
            $d = [double]$Matches[2]; $a = [double]$Matches[3]; $sumDone += $d; $sumAll += $a
            Line $Matches[1] (Pct $d $a) ('{0:N0} of {1:N0}' -f $d, $a)
        }
    }
    Line 'All' (Pct $sumDone $sumAll) ('{0:N0} of {1:N0}' -f $sumDone, $sumAll)
} else {
    Write-Host '  Not measurable yet. Arrives with M2 (inspector --coverage writes private/asset-coverage.txt).'
}
Write-Host ''

# 3. Spec pipeline and milestones.
Write-Host 'Pipeline'
$requests = @(TableRows (Join-Path $root 'specs/REQUESTS.md') | Where-Object { $_[0] -match '^R-\d+' })
$openReq = @($requests | Where-Object { $_[-1] -match '^Open' -and $_[-1] -notmatch 'standing' }).Count
$specs = @(TableRows (Join-Path $root 'specs/README.md') | Where-Object { $_[0] -match '^SPEC-\d+' })
$byStatus = @{}
foreach ($s in $specs) { $byStatus[$s[2]] = 1 + [int]$byStatus[$s[2]] }
Write-Host ('  Requests open: {0} of {1}' -f $openReq, @($requests | Where-Object { $_[-1] -notmatch 'standing' }).Count)
Write-Host ('  Specs: {0} total; Approved {1}, Implemented {2}, Verified {3}' -f $specs.Count, [int]$byStatus['Approved'], [int]$byStatus['Implemented'], [int]$byStatus['Verified'])
$milestones = @(TableRows (Join-Path $root 'docs/ROADMAP.md') | Where-Object { $_[0] -match '^M\d+' })
$done = @($milestones | Where-Object { $_[-1] -match '\*\*Done' })
Line 'Milestones' (Pct $done.Count $milestones.Count) ('{0} of {1} done' -f $done.Count, $milestones.Count)
$next = $milestones | Where-Object { $_[-1] -notmatch '\*\*Done' } | Select-Object -First 1
if ($next) { Write-Host "  Next: $($next[0]) $($next[1]) (needs $($next[2]))" }
Write-Host ''
