// Render the boot screen images from the logo SVGs, with Playwright's
// Chromium: bgrt-fallback.png (the mark, 192 px square) and watermark.png
// (the mark and "Celenn OS", 70 px high). Usage: node render.js (from this
// folder)
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const logo = (name) => fs.readFileSync(path.join(__dirname, '../logo', name), 'utf8');
const theme = path.join(__dirname, '../../system_files/usr/share/plymouth/themes/celenn');
const HEIGHT = 70; // of the watermark

(async () => {
  const browser = await chromium.launch();
  const render = async (svg, width, height, out) => {
    const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
    await page.setContent('<html><body style="margin:0;overflow:hidden;background:transparent">' + svg + '</body></html>');
    await page.evaluate(() => { document.querySelector('svg').style.cssText = 'width:100vw;height:100vh;display:block'; });
    await page.screenshot({ path: path.join(theme, out), omitBackground: true });
    await page.close();
  };
  await render(logo('celenn-logo.svg'), 192, 192, 'bgrt-fallback.png');
  const name = logo('celenn-os-logo-horizontal-dark.svg');
  const [, , w, h] = name.match(/viewBox="([^"]+)"/)[1].split(' ').map(Number);
  await render(name, Math.round(w * HEIGHT / h), HEIGHT, 'watermark.png');
  await browser.close();
})();
