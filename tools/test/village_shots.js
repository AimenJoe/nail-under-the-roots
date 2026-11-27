// 마을 화면 기준선(0307 0단계): 리팩터링 앞뒤로 마을 전경·마을 창 탭·텃밭 창을 같은 순서로 찍어 견준다.
// 가짜 시계(Playwright `page.clock`)로 멈춰 두고 흘리므로 같은 빌드는 같은 그림이 나와야 한다 — 먼저 같은 빌드를 두 번 찍어
// 차이(잡음 바닥)를 잰다(KB solo-game-codebase-health-signals: 잡음을 안 재면 화면 비교는 아무것도 말해 주지 않는다).
//   cargo build --release --target wasm32-unknown-unknown
//   python3 tools/web/assemble.py dist local dev
//   node tools/test/village_shots.js dist out-a          # 한국어·독일어로 찍는다
//   node tools/test/village_shots.js dist out-b
//   python3 tools/test/shots_diff.py out-a out-b         # 파일마다 다른 픽셀 수
// 기록은 개발 판(sandbox)이라 슬롯에 쓰지 않는다. 등불·텃밭·화단이 고루 보이게 채운 기록으로 연다.
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'village-shots');
const LOCALES = (process.env.LOCALES || 'ko-KR,de-DE').split(',');
const PORT = +(process.env.PORT || 8131);
const START_MS = 30000;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm', '.png': 'image/png',
  '.ttf': 'font/ttf', '.mp3': 'audio/mpeg', '.ogg': 'audio/ogg', '.txt': 'text/plain; charset=utf-8', '.md': 'text/markdown',
  '.webmanifest': 'application/manifest+json' };

// 이웃이 다 돌아왔고(반쪽 엔딩), 부탁 둘을 들어줬고, 등불은 이웃마다 다른 단계, 텃밭은 다 자란 칸·자라는 칸·빈 칸, 화단 꽃 넷.
const PROFILE = ['nail-profile 1',
  'flag tutorial', 'flag bori', 'flag line.village_intro', 'flag momo', 'flag doto_memory', 'flag kongi', 'flag clover',
  'flag half_ending', 'flag line.halmae_arrive',
  'flag quest.popo.asked', 'flag quest.popo.found', 'flag quest.popo.done',
  'flag quest.bori.asked', 'flag quest.bori.found', 'flag quest.bori.done',
  'flag decor.0', 'flag decor.2', 'flag decor.9',   // 꽃밭·통나무 의자·돌 우물(9 = village::EXTRA_FIRST — 위에 맡긴 반딧불이 뜬다)
  'count glowbugs_banked 450', 'count village_seeds 50', 'count pack.berry 1',
  'count lamp.popo 3', 'count lamp.bori 2', 'count lamp.momo 1',
  'count plot.0.crop 6', 'count plot.0.age 2', 'count plot.1.crop 2', 'count plot.1.age 1',
  'count garden.flowers 4', 'count garden.spot.0 1', 'count garden.spot.1 3', 'count garden.spot.2 4', 'count garden.spot.3 7',
  'count garden.berries 1', ''].join('\n');

// 개발 목록에서 "마을"의 자리: [묶음 번호, 묶음 안 자리](src/scenario.rs — smoke.js와 같은 읽기).
const VILLAGE = (() => {
  const src = fs.readFileSync(path.join(__dirname, '..', '..', 'src', 'scenario.rs'), 'utf8');
  const all = src.slice(src.indexOf('pub const ALL'), src.indexOf('];', src.indexOf('pub const ALL')));
  const cats = [...all.matchAll(/Cat::(\w+)/g)].map((m) => m[1]);
  const seen = {};
  for (const m of src.slice(src.indexOf('pub const LIST')).matchAll(/cat: Cat::(\w+),\s*name: "(.*?)"/g)) {
    const c = cats.indexOf(m[1]); seen[c] = seen[c] || 0;
    if (m[2] === '마을') return [c, seen[c]];
    seen[c]++;
  }
  throw new Error('개발 목록에 "마을"이 없다');
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

async function run() {
  fs.mkdirSync(OUT, { recursive: true });
  const srv = await serve();
  const browser = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
  for (const locale of LOCALES) {
    const lang = locale.slice(0, 2);
    const ctx = await browser.newContext({ viewport: { width: 960, height: 540 }, locale });
    await ctx.addInitScript((p) => { try { localStorage.clear(); localStorage.setItem('nail.profile', p); } catch (e) {} }, PROFILE);
    const pg = await ctx.newPage();
    const errors = [];
    // 알려진 잡음(smoke.js KNOWN_NOISE와 같다): 번들의 quad_net 플러그인.
    pg.on('pageerror', (e) => { if (!/register_plugin is not defined/.test(String(e))) errors.push(String(e)); });
    // 시계는 페이지를 열기 전에 건다. 게임이 뜬 뒤(타이틀)에 걸었더니 마을 전경(게임 장면)만 검게 찍혔다 — 창은 멀쩡했다.
    // 열기 전에 걸면 사라진다(도중에 걸 때 게임이 보는 시각이 한 번 뛰는 탓으로 짐작). 그 뒤로는 가짜 시계만 흘린다.
    await pg.clock.install({ time: new Date('2026-01-01T12:00:00Z') });
    let now = 0;                                              // 가짜 시계가 흐른 ms
    const tick = async (ms) => { await pg.clock.runFor(ms); now += ms; };
    await pg.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'load' });
    // 게임의 시각은 "wasm이 시작한 때부터"다(macroquad get_time = now − start_time). wasm을 받고 컴파일하는 데는 실제 시간이 들어
    // 실행마다 다르니, **wasm이 시작할 때까지 가짜 시계를 흘리지 않는다**(시작 시각 = 가짜 0). 그 뒤 에셋 읽기는 프레임이 돌아야 하고
    // 몇 프레임이 걸릴지 실행마다 다르다: 한 프레임씩 흘리며 첫 그림을 기다린 뒤 가짜 시각을 정한 값(START_MS)까지 맞춘다 —
    // 그때는 인트로(8.3초)가 저절로 끝나 타이틀이고, 그 뒤의 모든 시각이 실행마다 같다.
    await pg.waitForFunction(() => typeof wasm_exports !== 'undefined' && !!wasm_exports, null, { timeout: 60000 });
    for (let i = 0; i < 1200; i++) {
      await tick(16); await new Promise((r) => setTimeout(r, 50));
      if (await pg.evaluate(() => document.getElementById('loading').classList.contains('done'))) break;
    }
    if (now > START_MS - 10000) throw new Error(`로딩이 너무 길었다(가짜 ${now}ms) — START_MS를 늘린다`);
    await tick(START_MS - now); await pg.focus('#glcanvas');
    const press = async (k, ms = 300) => { await pg.keyboard.down(k); await tick(50); await pg.keyboard.up(k); await tick(ms); };
    const hold = async (k, ms) => { await pg.keyboard.down(k); await tick(ms); await pg.keyboard.up(k); await tick(250); };
    const shot = async (tag) => { await pg.screenshot({ path: path.join(OUT, `${lang}_${tag}.png`) }); };

    await press('ArrowUp', 300); await press('Enter', 800);    // 타이틀: 위로 = 마지막 줄(개발)
    for (let i = 0; i < VILLAGE[0]; i++) await press('ArrowRight', 120);
    for (let i = 0; i < VILLAGE[1]; i++) await press('ArrowDown', 120);
    await press('Enter', 3000);
    for (let i = 0; i < 14; i++) await press('Enter', 400);  // 들어서며 나오는 한마디를 넘긴다(어느 줄이 나올지는 난수 — 넉넉히)
    await tick(1000);
    await shot('01_village');
    // 텃밭(뿌리 길 오른쪽 아래): 창 → 둘째 칸 → 빈 셋째 칸에서 씨앗 바꾸기.
    await hold('ArrowRight', 330); await hold('ArrowDown', 220);
    await shot('02_garden_walk');
    await press('KeyJ', 600); await shot('03_garden_ripe');
    await press('ArrowDown', 300); await shot('04_garden_growing');
    await press('ArrowDown', 300); await press('ArrowRight', 300); await shot('05_garden_empty');
    await press('Escape', 600);
    // 포포(왼쪽 위 좌판): 마을 창 탭 넷.
    await hold('ArrowUp', 1120); await hold('ArrowLeft', 1150); await hold('ArrowUp', 150); await hold('ArrowDown', 130); await hold('ArrowLeft', 200);
    await shot('06_popo_walk');
    await press('KeyJ', 700); await tick(4000);               // 인사말이 설명으로 바뀔 때까지
    await shot('07_decor'); await press('ArrowDown', 300); await shot('08_decor_2');
    await press('KeyE', 400); await shot('09_pack');
    await press('KeyE', 400); await shot('10_lamp_done');
    await press('ArrowDown', 300); await shot('11_lamp_next');
    await press('ArrowDown', 300); await shot('12_lamp_first');
    await press('KeyE', 400); await shot('13_notes');
    for (let i = 0; i < 6; i++) await press('ArrowDown', 200);
    await shot('14_notes_neighbor');
    await press('Escape', 600); await shot('15_closed');
    if (errors.length) { console.error(`${lang}: 페이지 에러 ${JSON.stringify(errors)}`); process.exitCode = 1; }
    console.log(`${lang}: 15장 → ${OUT}`);
    await ctx.close();
  }
  await browser.close(); srv.close();
}
run();
