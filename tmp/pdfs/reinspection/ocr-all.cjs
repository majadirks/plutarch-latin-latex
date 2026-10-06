const fs=require('fs'),cp=require('child_process'),path=require('path');
const dir='tmp/pdfs/reinspection';
const rows=fs.readFileSync(dir+'/body_map.csv','utf8').trim().split(/\r?\n/).slice(1).map(l=>l.split(',').map(x=>x.replace(/^"|"$/g,'')));
const files=rows.map(r=>`high${r[0]}-${r[1].padStart(3,'0')}`);
let index=0,done=0;
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
async function worker(){while(index<files.length){const file=files[index++];const input=path.join(dir,file+'.png'),out=path.join(dir,file);
 if(!fs.existsSync(out+'.txt')) {while(!fs.existsSync(input)||Date.now()-fs.statSync(input).mtimeMs<2000)await sleep(1000);
 await new Promise((resolve,reject)=>cp.execFile('C:/Program Files/Tesseract-OCR/tesseract.exe',[input,out,'--psm','3'],{windowsHide:true},err=>err?reject(err):resolve()));}
 done++;if(done%100===0)console.log(`${done}/${files.length}`);}}
Promise.all(Array.from({length:8},worker)).then(()=>console.log('Full body OCR complete')).catch(e=>{console.error(e.message);process.exitCode=1});
