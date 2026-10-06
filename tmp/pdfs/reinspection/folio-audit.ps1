$ErrorActionPreference='Stop'
$out=@()
for($s=1;$s -le 2;$s++) {
 $pages=(Get-Content "tmp/pdfs/reinspection/source$s.txt" -Raw -Encoding UTF8).Split([char]12)
 for($p=1;$p -le $pages.Count;$p++) {
  $lines=@($pages[$p-1] -split '\r?\n' | Where-Object {$_.Trim() -ne ''} | Select-Object -First 3)
  $top=$lines -join ' '
  $headerline=' '
  if($lines.Count -gt 0) {$headerline=$lines[0]}
  $m=[regex]::Match($headerline,'(?<!\d)\d{1,4}(?!\d)')
  $out += [pscustomobject]@{source=$s;page=$p;folio=$m.Value;header=$top}
 }
}
$out | Export-Csv tmp/pdfs/reinspection/source-folios.csv -NoTypeInformation -Encoding UTF8
$ranges=(Get-Content src/facsimile/fixtures/inner_tomi_repair_source_trace.json -Raw -Encoding UTF8 | ConvertFrom-Json).global_scale_verification.ranges
if(-not $ranges) {
 $ledger=Get-Content src/facsimile/fixtures/inner_tomi_repair_source_trace.json -Raw -Encoding UTF8 | ConvertFrom-Json
 $ranges=$ledger.global_scale.ranges
}
$out | Where-Object {$_.page -gt 18 -and $_.folio -eq ''} | Format-Table source,page,header -Wrap
