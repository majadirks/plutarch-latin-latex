$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
for($t=1;$t -le 8;$t++) {
 & pdftoppm -r 45 -png "build_facsimile/tomus_${t}_inner.pdf" "tmp/pdfs/reinspection/t$t" 2> "tmp/pdfs/reinspection/render$t.log"
 & pdftotext -layout "build_facsimile/tomus_${t}_inner.pdf" "tmp/pdfs/reinspection/text$t.txt"
 $files=@(Get-ChildItem tmp/pdfs/reinspection -Filter "t$t-*.png" | Sort-Object Name)
 for($i=0;$i -lt $files.Count;$i+=64) {
  $b=New-Object System.Drawing.Bitmap 1536,2496
  $g=[System.Drawing.Graphics]::FromImage($b);$g.Clear([System.Drawing.Color]::White)
  $font=New-Object System.Drawing.Font 'Arial',11
  for($j=0;$j -lt 64 -and $i+$j -lt $files.Count;$j++) {
   $im=[System.Drawing.Image]::FromFile($files[$i+$j].FullName)
   $x=($j%8)*192;$y=[math]::Floor($j/8)*312
   $g.DrawString("$t / $($i+$j+1)",$font,[System.Drawing.Brushes]::Red,$x,$y)
   $g.DrawImage($im,$x,$y+23,186,284);$im.Dispose()
  }
  $b.Save("$pwd/tmp/pdfs/reinspection/sheet-$t-$([int]($i/64)).png");$g.Dispose();$b.Dispose();$font.Dispose()
 }
 Write-Output "Tomus $t : $($files.Count) pages"
}
