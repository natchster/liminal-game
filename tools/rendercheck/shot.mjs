import { chromium } from '/Users/nohwan/Documents/Personal_Projects/OpenCarole/node_modules/playwright/index.mjs';
const [url, out] = process.argv.slice(2);
const b = await chromium.launch({ args: ['--use-angle=metal', '--enable-gpu', '--ignore-gpu-blocklist'] });
const p = await b.newPage({ viewport: { width: 960, height: 540 } });
await p.goto(url); await p.waitForFunction(() => window.done === true, null, { timeout: 30000 });
await p.waitForTimeout(300);
await (await p.$('#c')).screenshot({ path: out });
await b.close();
