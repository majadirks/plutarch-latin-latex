$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System;using System.Drawing;using System.Drawing.Imaging;using System.Runtime.InteropServices;
public class EdgeReview {
 public static int[] Rows(Bitmap b) {
  var r=new int[b.Height];var rect=new Rectangle(0,0,b.Width,b.Height);
  var d=b.LockBits(rect,ImageLockMode.ReadOnly,PixelFormat.Format24bppRgb);
  var bytes=new byte[d.Stride*b.Height];Marshal.Copy(d.Scan0,bytes,0,bytes.Length);b.UnlockBits(d);
  for(int y=(int)(b.Height*.155);y<(int)(b.Height*.855);y++)for(int x=(int)(b.Width*.12);x<(int)(b.Width*.88);x++)if(bytes[y*d.Stride+x*3]<185)r[y]++;
  return r;
 }
}
'@
$map=@(Import-Csv tmp/pdfs/reinspection/body_map.csv)
$metrics=@()
for($t=1;$t -le 8;$t++) {
 $last=(Get-ChildItem tmp/pdfs/reinspection -Filter "t$t-*.png" | Sort-Object Name | Select-Object -Last 1).BaseName -replace "t$t-",''
 while(-not(Test-Path "tmp/pdfs/reinspection/high$t-$last.png")){Start-Sleep -Seconds 2}
 $pages=@($map | Where-Object {[int]$_.tomus -eq $t})
 for($i=0;$i -lt $pages.Count;$i+=60) {
  $b=New-Object System.Drawing.Bitmap 2235,2240;$g=[System.Drawing.Graphics]::FromImage($b);$g.Clear([System.Drawing.Color]::White);$f=New-Object System.Drawing.Font 'Arial',10
  for($j=0;$j -lt 60 -and $i+$j -lt $pages.Count;$j++) {
   $r=$pages[$i+$j];$p=([int]$r.page).ToString('000');$im=New-Object System.Drawing.Bitmap "$pwd/tmp/pdfs/reinspection/high$t-$p.png"
   $rows=[EdgeReview]::Rows($im);$ink=@();for($y=0;$y -lt $rows.Length;$y++){if($rows[$y] -gt 20){$ink+=$y}}
   if($ink.Count -eq 0){$im.Dispose();continue}
   $first=$ink[0];$end=$ink[-1];$gap=0;$gapstart=0
   for($k=1;$k -lt $ink.Count;$k++){if($ink[$k]-$ink[$k-1] -gt $gap){$gap=$ink[$k]-$ink[$k-1];$gapstart=$ink[$k-1]}}
   $metrics+=[pscustomobject]@{tomus=$t;page=$r.page;folio=$r.folio;unit=$r.unit;first=$first;last=$end;rows=$ink.Count;gap=$gap;gapstart=$gapstart}
   $x=($j%3)*745;$y=[math]::Floor($j/3)*112
   $g.DrawString("$t/$($r.page) r$($r.folio) top/bottom",$f,[System.Drawing.Brushes]::Red,$x,$y)
   $src=New-Object System.Drawing.Rectangle 0,([math]::Max(0,$first-3)),$im.Width,44
   $dest=New-Object System.Drawing.Rectangle $x,($y+18),745,44;$g.DrawImage($im,$dest,$src,[System.Drawing.GraphicsUnit]::Pixel)
   $src=New-Object System.Drawing.Rectangle 0,([math]::Max(0,$end-41)),$im.Width,44
   $dest=New-Object System.Drawing.Rectangle $x,($y+65),745,44;$g.DrawImage($im,$dest,$src,[System.Drawing.GraphicsUnit]::Pixel);$im.Dispose()
  }
  $b.Save("$pwd/tmp/pdfs/reinspection/edges-$t-$i.png");$g.Dispose();$b.Dispose();$f.Dispose()
 }
 $metrics | Export-Csv tmp/pdfs/reinspection/metrics.csv -NoTypeInformation
 Write-Output "Edges tomus $t complete"
}
