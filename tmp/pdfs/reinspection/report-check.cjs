const fs=require('fs');
const p='llm_artifacts/2026_10_06_15_40_INNER_TOMI_REINSPECTION_FINDINGS.md',t=fs.readFileSync(p,'utf8');
const cats=[...t.matchAll(/^## ([1-8])\./gm)].map(x=>x[1]);
const ends=[...t.matchAll(/^\| E(\d\d) \|/gm)].map(x=>+x[1]);
const expected=fs.readFileSync('tmp/pdfs/reinspection/endings.csv','utf8').trim().split(/\r?\n/).slice(1).map(l=>l.match(/"([^"]*)"/g).map(x=>x.slice(1,-1)));
const roman=['','I','II','III','IV','V','VI','VII','VIII'];
for(let i=0;i<expected.length;i++){
 const [v,page,asset,u,f]=expected[i];
 const line=t.split('\n').find(x=>x.startsWith(`| E${String(i+1).padStart(2,'0')} |`));
 if(!line||!line.includes(`| ${roman[+v]}/${page} | ${f} |`))throw Error(`endpoint mismatch ${u}`);
}
if(cats.join('')!=='12345678'||ends.length!==68||new Set(ends).size!==68)throw Error('coverage');
for(let n=4;n<=12;n++)if(!t.includes(`| D${n} |`))throw Error(`missing D${n}`);
const bad=t.split('\n').map((x,i)=>/[ \t]+$/.test(x)?i+1:0).filter(Boolean);
if(bad.length)throw Error(`trailing whitespace ${bad}`);
const rows=JSON.parse(fs.readFileSync('src/facsimile/fixtures/inner_tomi_repair_source_trace.json','utf8')).repairs.filter(x=>x.disposition==='mask');
console.log(JSON.stringify({categories:cats.length,endpoints:ends.length,mask_records:rows.length,mask_rectangles:rows.reduce((s,r)=>s+r.mask_rectangles.length,0),report_bytes:fs.statSync(p).size,whitespace:'passed'}));
