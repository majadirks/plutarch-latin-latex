param([int]$Tomus=8)
Add-Type -AssemblyName System.Drawing
$files=@(Get-ChildItem tmp/pdfs/inner_review -Filter "t$Tomus-???.png" | Sort-Object Name)
for($offset=0;$offset -lt $files.Count;$offset+=48){
 $canvas=New-Object System.Drawing.Bitmap(1320,1920)
 $g=[System.Drawing.Graphics]::FromImage($canvas);$g.Clear([System.Drawing.Color]::LightGray)
 $font=New-Object System.Drawing.Font('Arial',11)
 for($i=0;$i -lt 48 -and ($offset+$i) -lt $files.Count;$i++){
  $img=[System.Drawing.Image]::FromFile($files[$offset+$i].FullName)
  $x=($i%8)*165;$y=[math]::Floor($i/8)*320
  $g.DrawImage($img,$x,$y+20,165,252)
  $g.DrawString($files[$offset+$i].BaseName,$font,[System.Drawing.Brushes]::Black,$x,$y)
  $img.Dispose()
 }
 $canvas.Save((Join-Path $PWD "tmp/pdfs/inner_review/sheet_t${Tomus}_$offset.png"))
 $font.Dispose();$g.Dispose();$canvas.Dispose()
}
