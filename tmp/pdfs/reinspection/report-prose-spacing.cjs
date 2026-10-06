const fs=require('fs');
const p='llm_artifacts/2026_10_06_15_40_INNER_TOMI_REINSPECTION_FINDINGS.md';
let s=fs.readFileSync(p,'utf8');
const replacements=[
 [/\ball(?=\d)/g,'all '],[/\bAll(?=\d)/g,'All '],[/\bat(?=\d)/g,'at '],
 [/\bsource(?=\d)/g,'source '],[/\bSource(?=\d)/g,'Source '],
 [/\bchapter(?=[IVXL]+\b)/g,'chapter '],[/\bpage(?=\d)/g,'page '],
 [/\bphysical(?=\d)/g,'physical '],[/\bwhite(?=\d)/g,'white '],
 [/\bneighbor(?=\d)/g,'neighbor '],[/\bNeighbor(?=\d)/g,'Neighbor '],
 [/\bfull(?=[IVX]+\b)/g,'full '],[/\bCurrent(?=\d)/g,'Current '],
 [/\bduplicate(?=D\d)/g,'duplicate '],[/\brepeat(?=D\d)/g,'repeat '],
 [/\boccurrence(?=D\d)/g,'occurrence '],[/\bslice(?=S\d)/g,'slice '],
 [/\bline(?=C\d)/g,'line '],[/\bclipping(?=C\d)/g,'clipping '],
 [/\bomission(?=C\d)/g,'omission '],[/\bgap(?=M\d)/g,'gap '],
 [/\brectangle(?=\d)/g,'rectangle '],[/\brecord(?=\d)/g,'record '],
 [/\bhas(?=\d)/g,'has '],[/\brepeat(?=\d)/g,'repeat '],
 [/\bthe(?=\d)/g,'the '],[/\bof(?=\d)/g,'of '],
 [/\bcontains(?=\d)/g,'contains '],[/\bcovering(?=\d)/g,'covering '],
 [/\bfor(?=\d)/g,'for '],[/\bfrom(?=\d)/g,'from '],
 [/\bwith(?=\d)/g,'with '],[/\bincluding(?=C\d)/g,'including '],
 [/\bproduced(?=C\d)/g,'produced '],
 [/\b(\d+)(?=DPI\b)/g,'$1 DPI'],
 [/\b(\d+)-page reset/g,'$1-page reset'],
 [/\bon(?=\d)/g,'on '],[/\bon(?=[IVX]+\/)/g,'on '],
 [/\bending(?=\d)/g,'ending '],[/\bEnding(?=\d)/g,'Ending '],
 [/\bneighborhood(?=\d)/g,'neighborhood '],
 [/\b(\d+)body/g,'$1 body'],
 ];
// Apply prose spacing without changing any quoted Latin anchor.
s=s.split(/(`[^`]*`)/g).map((x,i)=>{if(i%2)return x;for(const [a,b] of replacements)x=x.replace(a,b);return x;}).join('');
fs.writeFileSync(p,s);
