$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
$pairs=@(@(118,123),@(212,215),@(296,299),@(360,363),@(376,379),@(464,467),@(522,525),@(672,675),@(700,703))
foreach($pair in $pairs){
 $b=New-Object System.Drawing.Bitmap 1200,1000
 $g=[System.Drawing.Graphics]::FromImage($b);$g.Clear([System.Drawing.Color]::White)
 $font=New-Object System.Drawing.Font 'Arial',16
 for($i=0;$i -lt 2;$i++){
  $n=$pair[$i];$im=New-Object System.Drawing.Bitmap "$pwd/build_facsimile/render_cache/Ploutarchou_Bioi_v2-p$($n.ToString('0000'))-300dpi.png"
  $y=if($i -eq 0){[int]($im.Height*.66)}else{[int]($im.Height*.035)}
  $rect=New-Object System.Drawing.Rectangle ([int]($im.Width*.46)),$y,([int]($im.Width*.54)),([int]($im.Height*.32))
  $dst=New-Object System.Drawing.Rectangle ($i*600),40,600,960
  $g.DrawImage($im,$dst,$rect,[System.Drawing.GraphicsUnit]::Pixel)
  $g.DrawString("v2/$n $(if($i -eq 0){'tail'}else{'head'})",$font,[System.Drawing.Brushes]::Red,($i*600),0)
  $im.Dispose()
 }
 $b.Save("$pwd/tmp/pdfs/reinspection/join-$($pair[0])-$($pair[1]).png");$g.Dispose();$b.Dispose();$font.Dispose()
}

