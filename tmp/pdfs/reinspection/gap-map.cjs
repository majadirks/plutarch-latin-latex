const fs=require('fs'),dir='tmp/pdfs/reinspection';
const words=t=>new Set((t.match(/[A-Za-z]{4,}/g)||[]).map(x=>x.toLowerCase()));
const src=[1,2].map(s=>fs.readFileSync(`${dir}/latin${s}.txt`,'utf8').split('\f').map(words));
const targets=[[1,24],[1,28],[1,36],[1,46],[1,61],[1,62],[1,66],[1,68],[1,71],[1,72],[1,73],[1,75],[1,77],[1,83],[1,87],[1,92],[1,94],[1,97],[1,100],[1,109],[1,111],[2,81],[2,139],[2,203],[2,207],[2,219],[2,223],[2,243],[2,261],[2,290],[3,173],[4,48],[8,87],[8,98],[8,128],[7,244],[6,53]];
let out=[];
for(const [t,p] of targets){const s=t<5?1:2; const a=words(fs.readFileSync(`${dir}/high${t}-${String(p).padStart(3,'0')}.txt`,'utf8'));const rank=src[s-1].map((b,i)=>({p:i+1,n:[...a].filter(w=>b.has(w)).length})).sort((a,b)=>b.n-a.n).slice(0,3);out.push({t,p,s,rank});}
fs.writeFileSync(`${dir}/gap-source-map.json`,JSON.stringify(out,null,2));
for(const r of out)console.log(`${r.t}/${r.p} v${r.s}: ${r.rank.map(x=>x.p+'('+x.n+')').join(' ')}`);
