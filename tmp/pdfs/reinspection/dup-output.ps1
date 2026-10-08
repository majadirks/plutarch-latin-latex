$ErrorActionPreference='Stop'
$ranges=@(@(5,208,240),@(6,38,65),@(6,210,241),@(6,326,341),@(7,24,36),@(7,202,228),@(7,323,347),@(8,216,242),@(8,277,299))
foreach($r in $ranges) {
 & pdftoppm -f $r[1] -l $r[2] -r 180 -png "build_facsimile/tomus_$($r[0])_inner.pdf" "tmp/pdfs/reinspection/dup$($r[0])" 2>> tmp/pdfs/reinspection/dup-render.log
 Write-Output "$($r[0])/$($r[1])-$($r[2])"
}
