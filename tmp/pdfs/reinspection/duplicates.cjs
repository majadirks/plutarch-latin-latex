const fs=require('fs');
const results=[];
for(let s=1;s<=2;s++) {
 const pages=fs.readFileSync(`tmp/pdfs/reinspection/source${s}.txt`,'utf8').split('\f');
 const sets=pages.map(p=>new Set((p.match(/[A-Za-zÀ-ÿ]{3,}/g)||[]).map(x=>x.toLowerCase())));
 for(let i=(s===1?13:17);i<(s===1?637:708);i++) for(let j=(s===1?13:17);j<i;j++) {
  const a=sets[i],b=sets[j];let intersection=0;for(const w of a) if(b.has(w))intersection++;
  const score=intersection/(a.size+b.size-intersection);
  if(score>.52)results.push({source:s,first:j+1,second:i+1,jaccard:score});
 }
}
fs.writeFileSync('tmp/pdfs/reinspection/duplicate-candidates.json',JSON.stringify(results,null,2));
console.log(results);
