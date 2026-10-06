const fs=require('fs'),dir='tmp/pdfs/reinspection';
const pages=fs.readFileSync(dir+'/latin2.txt','utf8').split('\f');
const targets=[[5,115,122],[6,211,214],[6,295,298],[6,359,362],[7,375,378],[7,463,466],[7,521,524],[8,671,674],[8,699,702]];
const tokenize=p=>new Set((p.match(/[A-Za-zÀ-ÿ]{4,}/g)||[]).map(x=>x.toLowerCase()));
const output=fs.readdirSync(dir).filter(x=>/^dup\d-\d+\.txt$/.test(x)).map(x=>({file:x,tomus:+x[3],page:+x.match(/-(\d+)/)[1],text:fs.readFileSync(dir+'/'+x,'utf8')}));
const matches=[];
for(const [t,start,end] of targets) {
 console.log(`TOMUS ${t} SOURCE ${start}-${end}`);
 for(let p=start;p<=end;p++) {
  const a=tokenize(pages[p-1]);const ranked=output.filter(x=>x.tomus===t).map(x=>{const b=tokenize(x.text);let common=0;for(const w of b)if(a.has(w))common++;return {...x,score:common/b.size};}).sort((a,b)=>b.score-a.score).slice(0,5);
  console.log(p,ranked.map(x=>`${x.page}(${x.score.toFixed(2)})`).join(' '));matches.push({source:p,tomus:t,matches:ranked.map(({text,...x})=>x)});
 }
}
fs.writeFileSync(dir+'/output-matches.json',JSON.stringify(matches,null,2));
