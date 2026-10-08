$ErrorActionPreference='Stop'
$all=@();$ends=@()
for($t=1;$t -le 8;$t++) {
 $tex=Get-Content "build_facsimile/tomus_${t}_inner.tex" -Raw -Encoding UTF8
 $refs=@([regex]::Matches($tex,'\\includegraphics[^\r\n]*?\{([^{}]+\.png)\}') | ForEach-Object {$_.Groups[1].Value})
 $imgs=@(& pdfimages -list "build_facsimile/tomus_${t}_inner.pdf" | Where-Object {$_ -match '^\s*\d+\s+\d+\s+image\s'} | ForEach-Object {($_.Trim() -split '\s+')[0]})
 if($refs.Count -ne $imgs.Count) {throw "Image count mismatch $t $($refs.Count) $($imgs.Count)"}
 $rows=@()
 for($i=0;$i -lt $refs.Count;$i++) {
  if($refs[$i] -match 'facsimile_(.+)_\d+\.png$') {$rows += [pscustomobject]@{tomus=$t;page=[int]$imgs[$i];asset=$refs[$i];unit=$Matches[1]}}
 }
 $first=($rows | Select-Object -First 1).page
 foreach($r in $rows) {$r | Add-Member -NotePropertyName folio -NotePropertyValue ($r.page-$first+1)}
 $all+=$rows
 $ends+=@($rows | Group-Object unit | ForEach-Object {$_.Group | Select-Object -Last 1})
}
$all | Export-Csv tmp/pdfs/reinspection/body_map.csv -NoTypeInformation
$ends | Export-Csv tmp/pdfs/reinspection/endings.csv -NoTypeInformation
$ends | Format-Table tomus,page,folio,unit
