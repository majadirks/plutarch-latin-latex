$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
for($s=1;$s -le 2;$s++) {
 $start=14;$end=637;if($s -eq 2){$start=18;$end=708}
 for($p=$start;$p -le $end;$p+=48) {
  $b=New-Object System.Drawing.Bitmap 1800,2240;$g=[System.Drawing.Graphics]::FromImage($b);$g.Clear([System.Drawing.Color]::White);$f=New-Object System.Drawing.Font 'Arial',12
  for($i=0;$i -lt 48 -and $p+$i -le $end;$i++) {
   $n=$p+$i;$x=($i%3)*600;$y=[math]::Floor($i/3)*140
   $file="build_facsimile/render_cache/Ploutarchou_Bioi_v$s-p$($n.ToString('0000'))-300dpi.png"
   $g.DrawString("v$s source page $n",$f,[System.Drawing.Brushes]::Red,$x,$y)
   if(Test-Path $file) {
    $im=[System.Drawing.Image]::FromFile("$pwd/$file")
    $dest=New-Object System.Drawing.Rectangle $x,($y+20),600,115
    $src=New-Object System.Drawing.Rectangle 0,0,$im.Width,([int]($im.Height*.19))
    $g.DrawImage($im,$dest,$src,[System.Drawing.GraphicsUnit]::Pixel);$im.Dispose()
   }else{$g.DrawString('CACHE MISSING',$f,[System.Drawing.Brushes]::Black,$x,($y+40))}
  }
  $b.Save("$pwd/tmp/pdfs/reinspection/headers-$s-$p.png");$g.Dispose();$b.Dispose();$f.Dispose()
 }
}
