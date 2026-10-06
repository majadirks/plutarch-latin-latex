$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
$refs=@(@(1,19),@(1,37),@(1,40),@(1,41),@(1,42),@(1,43),@(1,46),@(1,48),@(1,50),@(1,51),@(1,53),@(1,54),@(1,58),@(1,603),@(2,609))
foreach($r in $refs){
 $s=$r[0];$p=$r[1]
 $im=New-Object System.Drawing.Bitmap "$pwd/build_facsimile/render_cache/Ploutarchou_Bioi_v$s-p$($p.ToString('0000'))-300dpi.png"
 $w=[int]($im.Width*.54*.6);$h=[int]($im.Height*.6)
 $b=New-Object System.Drawing.Bitmap $w,$h;$g=[System.Drawing.Graphics]::FromImage($b)
 $rect=New-Object System.Drawing.Rectangle ([int]($im.Width*.46)),0,([int]($im.Width*.54)),$im.Height
 $dest=New-Object System.Drawing.Rectangle 0,0,$w,$h;$g.DrawImage($im,$dest,$rect,[System.Drawing.GraphicsUnit]::Pixel)
 $b.Save("$pwd/tmp/pdfs/reinspection/column$s-$p.png");$g.Dispose();$b.Dispose();$im.Dispose()
}
