const fs=require('fs'),cp=require('child_process'),path=require('path');
const dir='tmp/pdfs/reinspection';
const files=fs.readdirSync(dir).filter(x=>/^dup\d-\d+\.png$/.test(x));
let index=0,done=0;
async function worker(){while(index<files.length){const file=files[index++];const out=path.join(dir,file.slice(0,-4));if(!fs.existsSync(out+'.txt'))await new Promise((resolve,reject)=>cp.execFile('C:/Program Files/Tesseract-OCR/tesseract.exe',[path.join(dir,file),out,'--psm','3'],{windowsHide:true},(err)=>err?reject(err):resolve()));done++;if(done%25===0)console.log(`${done}/${files.length}`);}}
Promise.all(Array.from({length:6},worker)).then(()=>console.log('OCR complete')).catch(e=>{console.error(e.message);process.exitCode=1});
