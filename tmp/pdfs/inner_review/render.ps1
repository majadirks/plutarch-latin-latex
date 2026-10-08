$ErrorActionPreference = 'Stop'
for ($t=1; $t -le 8; $t++) {
  & pdftoppm -r 45 -png "build_facsimile/tomus_${t}_inner.pdf" "tmp/pdfs/inner_review/t$t" 2> "tmp/pdfs/inner_review/render_t$t.log"
  & pdfimages -list "build_facsimile/tomus_${t}_inner.pdf" | Set-Content "tmp/pdfs/inner_review/images_t$t.txt"
  & pdftotext -layout "build_facsimile/tomus_${t}_inner.pdf" "tmp/pdfs/inner_review/text_t$t.txt"
  Write-Output "Rendered tomus $t"
}
