$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
$refs=@(@(2,603),@(2,375),@(2,480),@(2,213),@(1,52),@(1,289),@(1,290),@(1,295),@(1,296),@(1,307),@(1,308),@(1,603),@(1,604),@(2,624))
for($i=0;$i -lt $refs.Count;$i+=4){
 $b=New-Object System.Drawing.Bitmap 1600,3600;$g=[System.Drawing.Graphics]::FromImage($b);$g.Clear([System.Drawing.Color]::White);$f=New-Object System.Drawing.Font 'Arial',16
 for($j=0;$j -lt 4 -and $i+$j -lt $refs.Count;$j++){
  $s=$refs[$i+$j][0];$p=$refs[$i+$j][1];$im=New-Object System.Drawing.Bitmap "$pwd/build_facsimile/render_cache/Ploutarchou_Bioi_v$s-p$($p.ToString('0000'))-300dpi.png"
  $x=($j%2)*800;$y=[math]::Floor($j/2)*1800
  $g.DrawString("SOURCE v$s/$p",$f,[System.Drawing.Brushes]::Red,$x,$y)
  $rect=New-Object System.Drawing.Rectangle ([int]($im.Width*.46)),0,([int]($im.Width*.54)),$im.Height
  $dest=New-Object System.Drawing.Rectangle $x,($y+30),800,1770;$g.DrawImage($im,$dest,$rect,[System.Drawing.GraphicsUnit]::Pixel);$im.Dispose()
 }
 $b.Save("$pwd/tmp/pdfs/reinspection/final-source-$([int]($i/4)).png");$g.Dispose();$b.Dispose();$f.Dispose()
}
$refs=@(@(2,235),@(3,298),@(1,67),@(1,68),@(1,69),@(2,207),@(2,208),@(6,228),@(6,229),@(6,230))
for($i=0;$i -lt $refs.Count;$i+=4){
 $b=New-Object System.Drawing.Bitmap 1490,2330;$g=[System.Drawing.Graphics]::FromImage($b);$g.Clear([System.Drawing.Color]::White);$f=New-Object System.Drawing.Font 'Arial',14
 for($j=0;$j -lt 4 -and $i+$j -lt $refs.Count;$j++){
  $t=$refs[$i+$j][0];$p=$refs[$i+$j][1];$im=[System.Drawing.Image]::FromFile("$pwd/tmp/pdfs/reinspection/high$t-$($p.ToString('000')).png")
  $x=($j%2)*745;$y=[math]::Floor($j/2)*1165;$g.DrawString("OUTPUT $t/$p",$f,[System.Drawing.Brushes]::Red,$x,$y);$g.DrawImage($im,$x,($y+30),745,1134);$im.Dispose()
 }
 $b.Save("$pwd/tmp/pdfs/reinspection/final-output-$([int]($i/4)).png");$g.Dispose();$b.Dispose();$f.Dispose()
}
