// Print the bounding box and centre of each avatar drawing (background, stars
// and ground removed), to check that it sits in the middle: node bbox.js
const {chromium}=require('playwright');const fs=require('fs'),path=require('path');
(async()=>{const b=await chromium.launch();const p=await b.newPage({viewport:{width:256,height:256}});
for(const f of fs.readdirSync(path.join(__dirname,'svg')).sort()){let s=fs.readFileSync(path.join(__dirname,'svg',f),'utf8');
s=s.replace(/<rect width="256" height="256" fill="url\(#g\)"\/>/,'').replace(/<g class="ground">.*?<\/g>/,'').replace(/<circle cx="\d+" cy="\d+" r="[\d.]+" fill="#fff" opacity="0\.\d+"\/>/g,'');
await p.setContent(`<body style="margin:0;background:transparent">${s}</body>`);
const r=await p.evaluate(async()=>{const svg=document.querySelector('svg');const img=new Image();img.src='data:image/svg+xml;base64,'+btoa(new XMLSerializer().serializeToString(svg));await img.decode();const c=document.createElement('canvas');c.width=c.height=256;const x=c.getContext('2d');x.drawImage(img,0,0);const d=x.getImageData(0,0,256,256).data;let a=[256,256,0,0];for(let y=0;y<256;y++)for(let X=0;X<256;X++)if(d[(y*256+X)*4+3]>40){a[0]=Math.min(a[0],X);a[1]=Math.min(a[1],y);a[2]=Math.max(a[2],X);a[3]=Math.max(a[3],y);}return a;});
console.log(f.padEnd(20),'bbox',r.join(','),' centre',((r[0]+r[2])/2).toFixed(0),((r[1]+r[3])/2).toFixed(0),' size',r[2]-r[0],r[3]-r[1]);}
await b.close();})();
