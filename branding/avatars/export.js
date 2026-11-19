// Render svg/*.svg to the 512x512 PNG avatars (Playwright).
// Usage: node export.js ../../system_files/usr/share/pixmaps/faces
const {chromium} = require('playwright');
const fs = require('fs'), path = require('path');
(async () => {
    const out = process.argv[2];
    const browser = await chromium.launch();
    const page = await browser.newPage({viewport: {width: 512, height: 512}});
    for (const f of fs.readdirSync(path.join(__dirname, 'svg')).sort()) {
        const svg = fs.readFileSync(path.join(__dirname, 'svg', f), 'utf8')
            .replace('width="256" height="256"', 'width="512" height="512"');
        await page.setContent(`<body style="margin:0">${svg}</body>`);
        await page.screenshot({path: path.join(out, f.replace('.svg', '.png')),
                               clip: {x: 0, y: 0, width: 512, height: 512}});
    }
    await browser.close();
})();
