// Render a GLSL fragment shader (common.glsl + NAME.frag) to a PNG with
// headless Chromium (WebGL 2, software rendering), tile by tile.
// Usage: node render.js NAME OUT.png [WIDTH HEIGHT]  (default 5120 2880)
// SHADER_DEFINES='#define X 1' in the environment sets options of a shader.
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const [name, out, w = '5120', h = '2880'] = process.argv.slice(2);
const W = +w, H = +h;
const src = (process.env.SHADER_DEFINES || '') + '\n' + fs.readFileSync(path.join(__dirname, 'common.glsl'), 'utf8') +
  fs.readFileSync(path.join(__dirname, name + '.frag'), 'utf8');
(async () => {
  const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
  const page = await browser.newPage();
  page.on('console', m => console.log(m.text()));
  const data = await page.evaluate(async ({ src, W, H }) => {
    const c = document.createElement('canvas'); c.width = W; c.height = H;
    const gl = c.getContext('webgl2', { preserveDrawingBuffer: true, antialias: false });
    const sh = (t, s) => { const o = gl.createShader(t); gl.shaderSource(o, s); gl.compileShader(o);
      if (!gl.getShaderParameter(o, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(o)); return o; };
    const p = gl.createProgram();
    gl.attachShader(p, sh(gl.VERTEX_SHADER, '#version 300 es\nin vec2 a;void main(){gl_Position=vec4(a,0.,1.);}'));
    gl.attachShader(p, sh(gl.FRAGMENT_SHADER, '#version 300 es\nprecision highp float;\nuniform vec2 iResolution;\nout vec4 fragColor;\n' + src));
    gl.linkProgram(p); gl.useProgram(p);
    const b = gl.createBuffer(); gl.bindBuffer(gl.ARRAY_BUFFER, b);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
    const loc = gl.getAttribLocation(p, 'a'); gl.enableVertexAttribArray(loc); gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);
    gl.uniform2f(gl.getUniformLocation(p, 'iResolution'), W, H);
    gl.enable(gl.SCISSOR_TEST); gl.viewport(0, 0, W, H);
    const T = 512;
    for (let y = 0; y < H; y += T) for (let x = 0; x < W; x += T) {
      gl.scissor(x, y, T, T); gl.drawArrays(gl.TRIANGLES, 0, 3); gl.finish();
      await new Promise(r => setTimeout(r, 0));
    }
    return c.toDataURL('image/png');
  }, { src, W, H });
  fs.writeFileSync(out, Buffer.from(data.split(',')[1], 'base64'));
  await browser.close();
})().catch(e => { console.error(e.message || e); process.exit(1); });
