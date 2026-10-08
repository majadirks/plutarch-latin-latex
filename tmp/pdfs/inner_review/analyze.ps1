Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System; using System.Drawing; using System.Collections.Generic;
public class ReviewScan {
 public static string Scan(string path) {
 using(var b=new Bitmap(path)) {
 var rows=new List<int>(); int total=0;
 for(int y=0;y<b.Height;y++) {int ink=0;for(int x=0;x<b.Width;x++){var c=b.GetPixel(x,y);if(c.R<190 && c.G<190 && c.B<190)ink++;}total+=ink;if(y>b.Height*.14 && y<b.Height*.87 && ink>3)rows.Add(y);}
 int gap=0,start=0,end=0; for(int i=1;i<rows.Count;i++)if(rows[i]-rows[i-1]>gap){gap=rows[i]-rows[i-1];start=rows[i-1];end=rows[i];}
 return total+","+rows.Count+","+gap+","+start+","+end;
 }
 }
}
'@
$out=@('file,ink,body_rows,max_gap,gap_start,gap_end')
Get-ChildItem tmp/pdfs/inner_review -Filter 't?-*.png' | ForEach-Object { $out += $_.Name+','+[ReviewScan]::Scan($_.FullName) }
$out | Set-Content tmp/pdfs/inner_review/scan.csv
