$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
$rows=@(Import-Csv tmp/pdfs/reinspection/endings.csv)
foreach($r in $rows) {
 & pdftoppm -f $r.page -l $r.page -r 180 -singlefile -png "build_facsimile/tomus_$($r.tomus)_inner.pdf" "tmp/pdfs/reinspection/end-$($r.tomus)-$($r.page)" 2>> tmp/pdfs/reinspection/end-render.log
}
for($i=0;$i -lt $rows.Count;$i+=4) {
 $b=New-Object System.Drawing.Bitmap 1488,2308;$g=[System.Drawing.Graphics]::FromImage($b);$g.Clear([System.Drawing.Color]::White)
 $f=New-Object System.Drawing.Font 'Arial',13
 for($j=0;$j -lt 4 -and $i+$j -lt $rows.Count;$j++) {
  $r=$rows[$i+$j];$im=[System.Drawing.Image]::FromFile("$pwd/tmp/pdfs/reinspection/end-$($r.tomus)-$($r.page).png")
  $x=($j%2)*744;$y=[math]::Floor($j/2)*1154
  $g.DrawString("$($r.tomus)/$($r.page) folio $($r.folio) $($r.unit)",$f,[System.Drawing.Brushes]::Red,$x,$y)
  $g.DrawImage($im,$x,$y+22,744,1132);$im.Dispose()
 }
 $b.Save("$pwd/tmp/pdfs/reinspection/end-sheet-$([int]($i/4)).png");$g.Dispose();$b.Dispose();$f.Dispose()
}
Write-Output "Rendered $($rows.Count) endings at 180 DPI"
