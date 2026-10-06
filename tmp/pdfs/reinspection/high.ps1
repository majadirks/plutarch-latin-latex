$ErrorActionPreference='Stop'
for($t=1;$t -le 8;$t++) {
 & pdftoppm -r 180 -png "build_facsimile/tomus_${t}_inner.pdf" "tmp/pdfs/reinspection/high$t" 2> "tmp/pdfs/reinspection/high$t.log"
 Write-Output "High resolution tomus $t complete"
}
