$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
$rows=Get-Content -Raw tmp/pdfs/reinspection/endpoint-map.json | ConvertFrom-Json
$units=Get-Content -Raw facsimile_units.json | ConvertFrom-Json
for($i=0;$i -lt $rows.Count;$i+=8){
 $b=New-Object System.Drawing.Bitmap 1600,2400;$g=[System.Drawing.Graphics]::FromImage($b);$g.Clear([System.Drawing.Color]::White);$font=New-Object System.Drawing.Font 'Arial',12
 for($j=0;$j -lt 8 -and $i+$j -lt $rows.Count;$j++){
  $r=$rows[$i+$j];$q=[int]$r.source;if($r.id -eq 'comparatio_sertorii_et_eumenis'){$q=105;$r.source=105}
  $x=($j%2)*800;$y=[math]::Floor($j/2)*600
  $g.DrawString("$($r.id) $($r.t)/$($r.p) r$($r.folio) SOURCE v$($r.s)/$q",$font,[System.Drawing.Brushes]::Red,$x,$y)
  $im=New-Object System.Drawing.Bitmap "$pwd/tmp/pdfs/reinspection/end-$($r.t)-$($r.p).png"
  $last=170
  for($yy=170;$yy -lt 1020;$yy++){$n=0;for($xx=65;$xx -lt 650;$xx+=2){$c=$im.GetPixel($xx,$yy);if($c.R -lt 120){$n++}};if($n -gt 12){$last=$yy}}
  $top=[math]::Max(160,$last-170);$rect=New-Object System.Drawing.Rectangle 55,$top,610,([math]::Min(190,$im.Height-$top));$dest=New-Object System.Drawing.Rectangle $x,($y+25),790,245;$g.DrawImage($im,$dest,$rect,[System.Drawing.GraphicsUnit]::Pixel);$im.Dispose()
  $src=New-Object System.Drawing.Bitmap "$pwd/build_facsimile/render_cache/Ploutarchou_Bioi_v$($r.s)-p$($q.ToString('0000'))-300dpi.png"
  $next=$units | Where-Object {$_.source -eq ($r.s-1) -and $_.start_page -eq $q -and $_.id -ne $r.id} | Select-Object -First 1
  $end=[int]($src.Height*.90);if($next){$end=[int]($src.Height*$next.title_top_frac)-10}
  $top=[math]::Max(0,$end-650);$rect=New-Object System.Drawing.Rectangle ([int]($src.Width*.46)),$top,([int]($src.Width*.54)),($end-$top)
  $dest=New-Object System.Drawing.Rectangle $x,($y+280),790,315;$g.DrawImage($src,$dest,$rect,[System.Drawing.GraphicsUnit]::Pixel);$src.Dispose()
 }
 $b.Save("$pwd/tmp/pdfs/reinspection/endpoint-proof-$([int]($i/8)).png");$g.Dispose();$b.Dispose();$font.Dispose()
}
$rows | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 tmp/pdfs/reinspection/endpoint-map.json
