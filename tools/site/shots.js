// 공식 페이지 스크린숏(docs/decisions/0069). 웹 빌드를 로컬에서 띄운 뒤(dist/를 http://127.0.0.1:8127/ 로)
//   node tools/site/shots.js <저장 폴더>
// 팩(assets/vendor)이 있는 개발판 빌드(기본 기능, 개발 메뉴)로 찍는다. 고른 것을 site/img/s_*.png로 옮긴다(아래 이름 = 페이지 이름).
// Playwright + Chromium(swiftshader). 960×540 = 게임 화면 3배. 시나리오 번호는 src/scenario.rs `LIST` 순서 — 목록이 바뀌면 고친다.
const { chromium } = require('playwright');
const OUT = process.argv[2] || 'shots';
const URL = process.env.NAIL_URL || 'http://127.0.0.1:8127/';
const base = 'nail-profile 1\nflag tutorial\nflag bori\nflag line.village_intro\n';
// 다 자란 마을: 이웃 모두, 깬 수 4(잎 단계 4), 꾸미기 전부.
const full = base + 'flag momo\nflag doto_memory\nflag kongi\nflag half_ending\ncount clears.Mole 4\ncount momo_wins 3\n' +
  [0, 1, 2, 3, 4, 5, 6, 7, 8].map((i) => `flag decor.${i}\n`).join('') + 'count glowbugs_banked 300\n';

async function open(browser, prof, locale = 'ko-KR') {
  const ctx = await browser.newContext({ viewport: { width: 960, height: 540 }, locale });
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

// 타이틀 → 개발 → 시나리오 idx.
async function dev(browser, idx, wait = 1500, prof = base, locale = 'ko-KR') {
  const s = await open(browser, prof, locale);
  await s.key('Enter', 600);
  await s.key('ArrowUp', 300); // 개발은 타이틀 메뉴 맨 끝 — 위로 한 번이면 돌아서 닿는다(메뉴 줄 수와 상관없이)
  await s.key('Enter', 600);
  for (let i = 0; i < idx; i++) await s.key('ArrowDown', 80);
  await s.key('Enter', wait);
  return s;
}

// 공격을 세 번씩 누르며 여러 장 찍는다(예고·타격이 보이는 칸을 고른다).
async function swings(s, name, n = 8, k = 'j') {
  for (let i = 0; i < n; i++) {
    await s.key(k, 90); await s.key(k, 90); await s.key(k, 40);
    await s.shot(`${name}_${i}`); await s.page.waitForTimeout(250);
  }
}

(async () => {
  const browser = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
  { const s = await dev(browser, 27, 2500, full); await s.key('Escape', 1200);   // 마을(다 자란 기록) — 들어설 때 한마디를 건너뛴다
    await s.hold('ArrowUp', 700); await s.shot('village_a'); await s.hold('ArrowUp', 900); await s.shot('village_b'); await s.ctx.close(); }
  // 쉼터 셋: 첫 쉼터·둘째(뿌리 동굴)·마지막(버섯 숲). 말 걸기 창을 넘긴다.
  for (const [idx, name] of [[1, 'camp'], [2, 'camp2'], [25, 'camp3']]) {
    const s = await dev(browser, idx);
    for (let i = 0; i < 4; i++) await s.key('Enter', 400);
    await s.shot(name); await s.ctx.close();
  }
  // 길 지도(갈림길, 0228 덧붙임): 첫 쉼터에서 뿌리 길(G) 곁으로 걸어가 J. 한국어 = route, 영어 = route_en(en/ 페이지).
  for (const [loc, name] of [['ko-KR', 'route'], ['en-US', 'route_en']]) {
    const s = await dev(browser, 1, 1500, base, loc);
    for (let i = 0; i < 4; i++) await s.key('Enter', 400);
    await s.hold('ArrowRight', 150); await s.hold('ArrowDown', 900); await s.key('j', 1500);
    await s.shot(name); await s.ctx.close();
  }
  // 지대 넷: 잠깐 걸어 적이 다가오게 한 뒤. 꽃 핀 입구는 환경 시나리오, 뿌리·흰 실은 실타래 굴(덤불·그물), 심층은 새 졸개 다섯.
  for (const [idx, name] of [[6, 'meadow'], [15, 'roots'], [14, 'mobs'], [36, 'silk']]) {
    const s = await dev(browser, idx);
    await s.hold('ArrowRight', 400); await s.page.waitForTimeout(1500);
    await s.shot(`${name}_a`); await s.page.waitForTimeout(1200); await s.shot(`${name}_b`);
    await s.ctx.close();
  }
  { const s = await dev(browser, 22); await swings(s, 'fight'); await s.ctx.close(); }   // 웨이브 3부터(나일)
  { const s = await dev(browser, 39); await swings(s, 'doto'); await s.ctx.close(); }    // 도토: 아레나
  { const s = await dev(browser, 41); await swings(s, 'momo'); await s.ctx.close(); }    // 모모: 아레나
  await browser.close();
})();
