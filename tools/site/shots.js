// 공식 페이지 스크린숏(docs/decisions/0069). 웹 빌드를 로컬에서 띄운 뒤(dist/를 http://127.0.0.1:8127/ 로)
//   node tools/site/shots.js <저장 폴더>
// 팩(assets/vendor)이 있는 빌드로 찍는다. 고른 것을 site/img/s_*.png로 옮긴다(village_b → s_village 등, 0069).
// Playwright + Chromium(swiftshader). 960×540 = 게임 화면 3배.
const { chromium } = require('playwright');
const OUT = process.argv[2] || 'shots';
const URL = process.env.NAIL_URL || 'http://127.0.0.1:8127/';
const base = 'nail-profile 1\nflag tutorial\nflag bori\nflag line.village_intro\n';
// 다 자란 마을: 이웃 모두, 깬 수 4(잎 단계 4), 꾸미기 전부.
const full = base + 'flag momo\nflag doto_memory\nflag kongi\nflag half_ending\ncount clears.Mole 4\ncount momo_wins 3\n' +
  [0, 1, 2, 3, 4, 5, 6, 7, 8].map((i) => `flag decor.${i}\n`).join('') + 'count glowbugs_banked 300\n';

async function open(browser, prof) {
  const ctx = await browser.newContext({ viewport: { width: 960, height: 540 }, locale: 'ko-KR' });
  await ctx.addInitScript((p) => {
    try { if (!sessionStorage.getItem('x')) { sessionStorage.setItem('x', 1); localStorage.setItem('nail.profile', p); } } catch (e) {}
  }, prof);
  const page = await ctx.newPage();
  await page.goto(URL, { waitUntil: 'load' });
  await page.waitForTimeout(3500);
  await page.focus('#glcanvas');
  const key = async (k, ms = 300) => { await page.keyboard.press(k); await page.waitForTimeout(ms); };
  const hold = async (k, ms) => { await page.keyboard.down(k); await page.waitForTimeout(ms); await page.keyboard.up(k); await page.waitForTimeout(150); };
  const shot = (name) => page.screenshot({ path: `${OUT}/${name}.png` });
  return { ctx, page, key, hold, shot };
}

// 타이틀 → 개발 → 시나리오 idx(src/scenario.rs 목록 순서).
async function dev(browser, idx) {
  const s = await open(browser, base);
  await s.key('Enter', 600);
  for (let i = 0; i < 4; i++) await s.key('ArrowDown', 150);
  await s.key('Enter', 600);
  for (let i = 0; i < idx; i++) await s.key('ArrowDown', 80);
  await s.key('Enter', 1500);
  return s;
}

(async () => {
  const browser = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
  { const s = await open(browser, base); await s.key('Escape', 1500); await s.shot('title2'); await s.ctx.close(); }
  { const s = await open(browser, full); await s.key('Enter', 2500);
    for (let i = 0; i < 4; i++) await s.key('Enter', 400);
    await s.hold('ArrowUp', 700); await s.shot('village_b'); await s.ctx.close(); }
  { const s = await dev(browser, 1); for (let i = 0; i < 4; i++) await s.key('Enter', 400); await s.shot('camp'); await s.ctx.close(); }
  // 셋째 웨이브부터: 세 번 휘두를 때마다 찍어 예고·타격이 보이는 칸을 고른다.
  { const s = await dev(browser, 18);
    for (let n = 0; n < 8; n++) { await s.key('j', 90); await s.key('j', 90); await s.key('j', 40); await s.shot(`wave3_${n}`); await s.page.waitForTimeout(250); }
    await s.ctx.close(); }
  for (const [idx, name] of [[26, 'doto'], [24, 'momo']]) {
    const s = await dev(browser, idx);
    await s.shot(`${name}_talk`);
    for (let i = 0; i < 4; i++) await s.key('Enter', 400);
    await s.page.waitForTimeout(1800);
    await s.shot(`${name}_fight`);
    await s.ctx.close();
  }
  await browser.close();
})();
