$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
$pairs=Get-Content tmp/pdfs/reinspection/duplicate-candidates.json -Raw | ConvertFrom-Json
foreach($p in $pairs) {
 if(($p.second -ge 339 -and $p.second -le 348) -or $p.second -eq 103 -or $p.second -eq 104 -or $p.second -eq 369 -or $p.second -eq 370) {continue}
 foreach($n in @($p.first,$p.second)) {
  & pdftoppm -f $n -l $n -r 180 -singlefile -png Ploutarchou_Bioi_v2.pdf "tmp/pdfs/reinspection/src2-$n" 2>> tmp/pdfs/reinspection/source-render.log
 }
 $b=New-Object System.Drawing.Bitmap 2280,1808;$g=[System.Drawing.Graphics]::FromImage($b);$g.Clear([System.Drawing.Color]::White)
 $f=New-Object System.Drawing.Font 'Arial',15;$x=0
 foreach($n in @($p.first,$p.second)) {
  $im=[System.Drawing.Image]::FromFile("$pwd/tmp/pdfs/reinspection/src2-$n.png")
  $g.DrawString("Source v2 physical page $n",$f,[System.Drawing.Brushes]::Red,$x,0)
  $g.DrawImage($im,$x,24,1140,1784);$im.Dispose();$x+=1140
 }
 $b.Save("$pwd/tmp/pdfs/reinspection/pair-$($p.first)-$($p.second).png");$g.Dispose();$b.Dispose();$f.Dispose()
}
