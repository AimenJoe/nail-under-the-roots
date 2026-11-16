// 상점 캐러셀용 영어 스크린숏(Epic 검토 2026-11-13: 영어로 읽히지 않는 글이 든 캐러셀 그림은 받지 않는다).
// 공식 페이지 스크린숏(tools/site/shots.js)은 한국어 화면이라 상점에 못 쓴다 — 같은 장면을 영어(en-US)로 다시 찍는다.
//   cargo build --release --target wasm32-unknown-unknown      # 개발 기능(기본) + 팩(assets/vendor)이 있는 빌드
//   python3 tools/web/assemble.py dist local dev
//   node tools/store/shots_en.js dist <저장 폴더>
// 장면마다 몇 장씩 찍는다. 고른 것을 docs/epic/shots/<이름>.png(960×540 = 게임 화면 3배)로 옮기고
// python3 tools/store/epic_media.py 로 캐러셀(×2 = 1920×1080)을 만든다.
// 개발 목록은 이름으로 고른다(src/scenario.rs LIST — 묶음 탭과 그 안의 자리를 읽는다, tools/test/smoke.js와 같은 방식).
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots-en');
const PORT = 8131;
const LOCALE = 'en-US';
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm', '.png': 'image/png',
  '.ttf': 'font/ttf', '.mp3': 'audio/mpeg', '.ogg': 'audio/ogg', '.txt': 'text/plain; charset=utf-8',
  '.webmanifest': 'application/manifest+json' };
const base = 'nail-profile 1\nflag tutorial\nflag bori\nflag line.village_intro\n';
// 다 자란 마을: 이웃 모두, 깬 수 4(잎 단계 4), 꾸미기 전부(tools/site/shots.js와 같은 기록).
const full = base + 'flag momo\nflag doto_memory\nflag kongi\nflag half_ending\ncount clears.Mole 4\ncount momo_wins 3\n' +
  [0, 1, 2, 3, 4, 5, 6, 7, 8].map((i) => `flag decor.${i}\n`).join('') + 'count glowbugs_banked 300\n';

const DEV = (() => {
  const src = fs.readFileSync(path.join(__dirname, '..', '..', 'src', 'scenario.rs'), 'utf8');
  const all = src.slice(src.indexOf('pub const ALL'), src.indexOf('];', src.indexOf('pub const ALL')));
  const cats = [...all.matchAll(/Cat::(\w+)/g)].map((m) => m[1]);
  const list = src.slice(src.indexOf('pub const LIST'));
  const seen = {}; const out = {};
  for (const m of list.matchAll(/cat: Cat::(\w+),\s*name: "(.*?)"/g)) {
    const c = cats.indexOf(m[1]); seen[c] = (seen[c] || 0); out[m[2]] = [c, seen[c]++];
  }
  return out;
})();

function serve() {
  return new Promise((resolve) => {
    const srv = http.createServer((req, res) => {
      let p = decodeURIComponent(req.url.split('?')[0]);
      if (p.endsWith('/')) p += 'index.html';
      const file = path.join(DIST, p);
      if (!file.startsWith(DIST) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404); res.end(); return; }
      res.writeHead(200, { 'Content-Type': MIME[path.extname(file)] || 'application/octet-stream', 'Content-Length': fs.statSync(file).size });
      fs.createReadStream(file).pipe(res);
    });
    srv.listen(PORT, '127.0.0.1', () => resolve(srv));
  });
}

async function open(browser, prof) {
  const ctx = await browser.newContext({ viewport: { width: 960, height: 540 }, locale: LOCALE });
  await ctx.addInitScript((p) => {
    try { if (!sessionStorage.getItem('x')) { sessionStorage.setItem('x', 1); localStorage.setItem('nail.profile', p); } } catch (e) {}
  }, prof);
  const page = await ctx.newPage();
  await page.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'load' });
  await page.waitForFunction(() => document.getElementById('loading').classList.contains('done'), null, { timeout: 60000 });
  await page.waitForTimeout(800);
  await page.focus('#glcanvas');
  const key = async (k, ms = 300) => { await page.keyboard.press(k); await page.waitForTimeout(ms); };
  const hold = async (k, ms) => { await page.keyboard.down(k); await page.waitForTimeout(ms); await page.keyboard.up(k); await page.waitForTimeout(150); };
  const shot = (name) => page.screenshot({ path: path.join(OUT, `${name}.png`) });
  return { ctx, page, key, hold, shot };
}

// 타이틀 → 개발(첫 항목에서 위로 = 마지막 항목) → 묶음 탭(오른쪽) → 그 안의 줄(아래).
async function dev(browser, name, prof = base, wait = 1800) {
  const at = DEV[name];
  if (!at) throw new Error(`개발 목록에 없다: ${name}`);
  const s = await open(browser, prof);
  await s.key('Enter', 600);
  await s.key('ArrowUp', 250);
  await s.key('Enter', 700);
  for (let i = 0; i < at[0]; i++) await s.key('ArrowRight', 80);
  for (let i = 0; i < at[1]; i++) await s.key('ArrowDown', 60);
  await s.key('Enter', wait);
  return s;
}

// 공격을 세 번씩 누르며 여러 장(예고·타격이 보이는 칸을 고른다).
async function swings(s, name, n = 6) {
  for (let i = 0; i < n; i++) {
    await s.key('j', 90); await s.key('j', 90); await s.key('j', 40);
    await s.shot(`${name}_${i}`); await s.page.waitForTimeout(300);
  }
}

// 걸어 들어가 적이 다가오게 한 뒤 몇 장.
async function walk(s, name, dir = 'ArrowRight') {
  await s.hold(dir, 400); await s.page.waitForTimeout(1200);
  for (let i = 0; i < 4; i++) { await s.shot(`${name}_${i}`); await s.page.waitForTimeout(900); }
}

(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  const srv = await serve();
  const browser = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
  const scenes = [
    ['meadow', '환경: 꽃 핀 입구', walk],
    ['roots', '실타래 굴: 뿌리', walk],
    ['silk', '실타래 굴: 흰 실', walk],
    ['water', '환경: 물길 굴', walk],
    ['boss', '콩이', swings, true],
    ['doto', '도토', swings, true],
    ['hall', '넓은 방', swings],
    ['mobs', '새 졸개 다섯', walk],
  ];
  const only = (process.env.ONLY || '').split(',').filter(Boolean);
  for (const [name, entry, how, intro] of scenes) {
    if (only.length && !only.includes(name)) continue;
    const s = await dev(browser, entry);
    if (intro) { await s.hold('Escape', 2500); await s.page.waitForTimeout(2500); } // 등장 컷씬은 뒤로를 길게 눌러 건너뛴다
    await how(s, name);
    await s.ctx.close();
    console.log('done', name);
  }
  if (!only.length || only.includes('village')) { // 다 자란 마을: 들어설 때 한마디를 건너뛰고(뒤로 길게) 위로 걷는다.
    const s = await dev(browser, '마을', full, 2500);
    await s.hold('Escape', 2500); await s.page.waitForTimeout(1200);
    await s.hold('ArrowUp', 700); await s.shot('village_0');
    await s.hold('ArrowUp', 900); await s.shot('village_1');
    await s.hold('ArrowLeft', 600); await s.shot('village_2');
    await s.ctx.close();
    console.log('done village');
  }
  await browser.close();
  srv.close();
})();
