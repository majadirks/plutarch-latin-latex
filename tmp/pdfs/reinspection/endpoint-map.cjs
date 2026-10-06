const fs=require('fs'),d='tmp/pdfs/reinspection';
const ends=fs.readFileSync(`${d}/endings.csv`,'utf8').trim().split(/\r?\n/).slice(1).map(l=>l.match(/"([^"]*)"/g).map(x=>x.slice(1,-1)));
const units=JSON.parse(fs.readFileSync('facsimile_units.json','utf8'));
const ranges=JSON.parse(fs.readFileSync('src/facsimile/fixtures/inner_tomi_repair_source_trace.json','utf8')).scale.width_sampling_ranges;
const sources=[1,2].map(s=>fs.readFileSync(`${d}/latin${s}.txt`,'utf8').split('\f'));
const words=s=>(s.match(/[A-Za-z]{3,}/g)||[]).map(x=>x.toLowerCase());
const result=[];
for(const [t,p,asset,id,folio] of ends){
const u=units.find(x=>x.id===id),r=ranges.find(x=>x.unit_id===id);
const txt=fs.readFileSync(`${d}/high${t}-${p.padStart(3,'0')}.txt`,'utf8');
const lines=txt.trim().split('\n').filter(x=>x.trim()&&!/^\d+$/.test(x.trim()));
const tail=lines.slice(-4).join(' '),a=words(tail);
const rank=[];for(let q=r.end_page-3;q<=r.end_page;q++){const b=new Set(words(sources[u.source][q-1]));rank.push({q,n:a.filter(x=>b.has(x)).length});}
rank.sort((a,b)=>b.n-a.n);result.push({t:+t,p:+p,id,folio:+folio,s:u.source+1,source:rank[0].q,tail,rank});
}
fs.writeFileSync(`${d}/endpoint-map.json`,JSON.stringify(result,null,2));
for(const r of result)console.log(`${r.t}/${r.p} ${r.id} v${r.s}/${r.source} [${r.rank.map(x=>x.q+':'+x.n).join(',')}] ${r.tail}`);
