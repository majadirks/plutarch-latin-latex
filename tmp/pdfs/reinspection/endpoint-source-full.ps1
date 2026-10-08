$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
$refs=@(@(1,390),@(1,438),@(1,530),@(1,584),@(1,637),@(2,70),@(2,105),@(2,191),@(2,285),@(2,428),@(2,481),@(2,565),@(2,679))
for($i=0;$i -lt $refs.Count;$i+=4){
 $b=New-Object System.Drawing.Bitmap 1600,3600;$g=[System.Drawing.Graphics]::FromImage($b);$g.Clear([System.Drawing.Color]::White);$f=New-Object System.Drawing.Font 'Arial',16
 for($j=0;$j -lt 4 -and $i+$j -lt $refs.Count;$j++){
  $s=$refs[$i+$j][0];$p=$refs[$i+$j][1];$im=New-Object System.Drawing.Bitmap "$pwd/build_facsimile/render_cache/Ploutarchou_Bioi_v$s-p$($p.ToString('0000'))-300dpi.png"
  $x=($j%2)*800;$y=[math]::Floor($j/2)*1800
  $g.DrawString("SOURCE v$s/$p",$f,[System.Drawing.Brushes]::Red,$x,$y)
  $rect=New-Object System.Drawing.Rectangle ([int]($im.Width*.46)),0,([int]($im.Width*.54)),$im.Height
  $dest=New-Object System.Drawing.Rectangle $x,($y+30),800,1770;$g.DrawImage($im,$dest,$rect,[System.Drawing.GraphicsUnit]::Pixel);$im.Dispose()
 }
 $b.Save("$pwd/tmp/pdfs/reinspection/endpoint-source-full-$([int]($i/4)).png");$g.Dispose();$b.Dispose();$f.Dispose()
}

