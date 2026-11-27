// 마을 화면 기준선(0307 0단계): 리팩터링 앞뒤로 마을 전경·마을 창 탭·텃밭 창을 같은 순서로 찍어 견준다.
// 가짜 시계(Playwright `page.clock`)로 멈춰 두고 흘리므로 같은 빌드는 같은 그림이 나와야 한다 — 먼저 같은 빌드를 두 번 찍어
// 차이(잡음 바닥)를 잰다(KB solo-game-codebase-health-signals: 잡음을 안 재면 화면 비교는 아무것도 말해 주지 않는다).
//   cargo build --release --target wasm32-unknown-unknown
//   python3 tools/web/assemble.py dist local dev
//   node tools/test/village_shots.js dist out-a          # 한국어·독일어로 33장씩 찍는다
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
// 개발 목록의 어느 줄에서 시작하나: 기본은 "마을"(아래 기록으로). 확인용 줄(기록을 스스로 꾸민다)도 이름으로 고를 수 있다.
const ENTRY = process.env.ENTRY || '확인 0307: 사자마자 마을에';
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

// 개발 목록에서 시작할 줄(ENTRY)의 자리: [묶음 번호, 묶음 안 자리](src/scenario.rs — smoke.js와 같은 읽기).
const VILLAGE = (() => {
  const src = fs.readFileSync(path.join(__dirname, '..', '..', 'src', 'scenario.rs'), 'utf8');
  const all = src.slice(src.indexOf('pub const ALL'), src.indexOf('];', src.indexOf('pub const ALL')));
  const cats = [...all.matchAll(/Cat::(\w+)/g)].map((m) => m[1]);
  const seen = {};
  for (const m of src.slice(src.indexOf('pub const LIST')).matchAll(/cat: Cat::(\w+),\s*name: "(.*?)"/g)) {
    const c = cats.indexOf(m[1]); seen[c] = seen[c] || 0;
    if (m[2] === ENTRY) return [c, seen[c]];
    seen[c]++;
  }
  throw new Error(`개발 목록에 "${ENTRY}"이 없다`);
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
  for (const locale of LOCALES) {
    // 언어마다 브라우저를 새로 띄운다: 한 브라우저에서 한국어 다음에 독일어를 찍으면 독일어 화면이 가끔(기계가 바쁠 때)
    // 통째로 다른 갈래로 나왔다 — 독일어만 따로 네 번은 늘 같았다. 원인은 못 찾았다(0307).
    const browser = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
    const lang = locale.slice(0, 2);
    const ctx = await browser.newContext({ viewport: { width: 960, height: 540 }, locale });
    await ctx.addInitScript((p) => { try { localStorage.clear(); localStorage.setItem('nail.profile', p); } catch (e) {} }, PROFILE);
    const pg = await ctx.newPage();
    const errors = [];
    // 알려진 잡음(smoke.js KNOWN_NOISE와 같다): 번들의 quad_net 플러그인.
    pg.on('pageerror', (e) => { if (!/register_plugin is not defined/.test(String(e))) errors.push(String(e)); });
    let inflight = 0;                                         // 받는 중인 요청 수
    pg.on('request', () => { inflight++; });
    pg.on('requestfinished', () => { inflight--; });
    pg.on('requestfailed', () => { inflight--; });
    // 시계는 페이지를 열기 전에 걸고 **바로 멈춘다**. `clock.install`만으로는 시계가 멈추지 않는다 — 실제 시간과 함께 계속 흐르고
    // `runFor`가 그 위에 더할 뿐이다(Playwright 1.63에서 잼: install 뒤 실제 0.5초 = 가짜 0.5초, `pauseAt` 뒤에는 0).
    // 그래서 같은 빌드를 두 번 찍으면 스크립트가 걸린 실제 시간만큼 게임 시각이 달랐다(입자·몸짓·걷는 거리까지).
    // `pauseAt` 뒤로는 `runFor`로 흘린 만큼만 흐른다. 게임이 뜬 뒤에 걸면 시각이 한 번 뛰어 게임 장면이 검게 찍힌다 — 열기 전에.
    const T0 = new Date('2026-01-01T12:00:00Z');
    await pg.clock.install({ time: T0 });
    await pg.clock.pauseAt(new Date(T0.getTime() + 1000));
    let now = 0;                                              // 가짜 시계가 흐른 ms
    const tick = async (ms) => { await pg.clock.runFor(ms); now += ms; };
    await pg.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'load' });
    // 게임의 시각은 "wasm이 시작한 때부터"다(macroquad get_time = now − start_time). wasm을 받고 컴파일하는 데는 실제 시간이 들어
    // 실행마다 다르니, **wasm이 시작할 때까지 가짜 시계를 흘리지 않는다**(시작 시각 = 가짜 0). 그 뒤 에셋 읽기는 프레임이 돌아야 하고
    // 몇 프레임이 걸릴지 실행마다 다르다: 한 프레임씩 흘리며 첫 그림을 기다린 뒤 가짜 시각을 정한 값(START_MS)까지 맞춘다 —
    // 그때는 인트로(8.3초)가 저절로 끝나 타이틀이고, 그 뒤의 모든 시각이 실행마다 같다.
    await pg.waitForFunction(() => typeof wasm_exports !== 'undefined' && !!wasm_exports, null, { timeout: 60000 });
    // **한 프레임을 흘리기 전에 받는 중인 것이 다 올 때까지 기다린다**(실제 시간): 게임은 프레임마다 "다 왔나"를 보고 다음 것을 읽으니,
    // 받는 데 걸린 실제 시간이 프레임 수로 새면 게임이 시작한 프레임이 실행마다 달라진다(기계가 바쁠 때 화면이 통째로 다른 갈래로 나왔다).
    // 다 온 뒤에 한 프레임씩이면 프레임 수는 읽을 것의 수로 정해진다.
    const idle = async () => {
      let quiet = 0;
      for (let i = 0; i < 1000 && quiet < 6; i++) { await new Promise((r) => setTimeout(r, 5)); quiet = inflight === 0 ? quiet + 1 : 0; }
    };
    const settle = async (frames) => { for (let i = 0; i < frames; i++) { await idle(); await tick(16); } };
    let loadFrames = 0;
    for (; loadFrames < 3000; loadFrames++) {
      await idle(); await tick(16);
      if (await pg.evaluate(() => document.getElementById('loading').classList.contains('done'))) break;
    }
    await settle(80);                                         // 첫 그림 뒤에 읽기 시작하는 것(타이틀 곡 등)
    if (process.env.VERBOSE) console.log(`${lang}: 로딩 ${loadFrames}프레임`);
    if (now > START_MS - 10000) throw new Error(`로딩이 너무 길었다(가짜 ${now}ms) — START_MS를 늘린다`);
    await tick(START_MS - now); await pg.focus('#glcanvas');
    // 키는 브라우저의 입력 길(CDP → 브라우저 → 합성 스레드 → 메인 스레드)로 넣지 않고 **페이지 안에서 바로** 쏜다:
    // 그 길은 `runFor`(메인 스레드로 곧장 가는 evaluate)와 순서가 보장되지 않아, 기계가 바쁘면 키가 한두 프레임 늦게 닿았다
    // (걷다 멈춘 자리가 달라지고 그 뒤 화면이 통째로 갈렸다). web/nail-host.js가 창의 keydown·keyup을 받아 게임에 넘긴다.
    const key = async (down, k) => {
      await pg.evaluate(([type, code]) => {
        window.dispatchEvent(new KeyboardEvent(type, { code, key: code, bubbles: true, cancelable: true }));
      }, [down ? 'keydown' : 'keyup', k]);
    };
    const press = async (k, ms = 300) => { await key(true, k); await tick(50); await key(false, k); await tick(ms); };
    const hold = async (k, ms) => { await key(true, k); await tick(ms); await key(false, k); await tick(250); };
    // 찍기 전에 실제 시간으로 잠깐: 마지막으로 그린 프레임이 화면에 올라온 뒤에 찍는다(가짜 시계와 상관없는 Node 쪽 기다림).
    const shot = async (tag) => { await new Promise((r) => setTimeout(r, 80)); await pg.screenshot({ path: path.join(OUT, `${lang}_${tag}.png`) }); };

    await press('ArrowUp', 300); await press('Enter', 800);    // 타이틀: 위로 = 마지막 줄(개발)
    for (let i = 0; i < VILLAGE[0]; i++) await press('ArrowRight', 120);
    for (let i = 0; i < VILLAGE[1]; i++) await press('ArrowDown', 120);
    // 장면이 시작하며 읽는 것(마을의 곡·환경음)도 같은 까닭으로 실제 시간을 주며 흘린다(가짜 시간으로는 전과 같은 3초).
    await press('Enter', 100); await settle(60); await tick(3000 - 100 - 60 * 16);
    // 확인용 줄은 들어설 때의 한마디를 다 들은 기록이라 대화가 없다(src/scenario.rs `hear_village_lines`) — 넘길 것 없이 바로 찍는다.
    await tick(1000);
    await shot('01_village');
    // 텃밭(뿌리 길 오른쪽 아래): 창 → 칸마다 → 빈 칸에서 씨앗 바꾸고 심기 → 자라는 칸에서 확인 → 다 자란 칸 거두기.
    await hold('ArrowRight', 330); await hold('ArrowDown', 220);
    await shot('02_garden_walk');
    await press('KeyJ', 600); await shot('03_garden_ripe');
    await press('ArrowDown', 300); await shot('04_garden_growing');
    await press('Enter', 300); await shot('05_garden_wait');
    await press('ArrowDown', 300); await press('ArrowRight', 300); await shot('06_garden_empty');
    await press('Enter', 300); await shot('07_garden_planted');
    await press('ArrowUp', 300); await press('ArrowUp', 300); await press('Enter', 300); await shot('08_garden_harvested');
    await press('Escape', 600); await tick(400); await shot('09_village_after_garden');
    // 포포(왼쪽 위 좌판): 마을 창 탭 넷을 보고, 탭마다 한 번씩 해 본다.
    await hold('ArrowUp', 1120); await hold('ArrowLeft', 1150); await hold('ArrowUp', 150); await hold('ArrowDown', 130); await hold('ArrowLeft', 320);
    await tick(600);                                          // 멈춘 뒤 말 걸기 표시가 뜰 때까지(바로 누르면 헛손질이 된다)
    await shot('10_popo_walk');
    await press('KeyJ', 700); await shot('11_decor_hello'); await tick(4000);   // 인사말 → 설명
    await shot('12_decor'); await press('ArrowDown', 300); await shot('13_decor_2');
    await press('ArrowDown', 300); await press('Enter', 300); await shot('14_decor_bought');
    await press('Enter', 300); await shot('15_decor_again');
    await press('ArrowUp', 300); await press('ArrowUp', 300); await press('ArrowUp', 300); await press('Enter', 300); await shot('16_decor_journal');
    await press('KeyE', 400); await shot('17_pack');
    await press('ArrowRight', 300); await shot('18_pack_plus');
    await press('ArrowRight', 300); await press('ArrowRight', 300); await shot('19_pack_full');
    await press('ArrowLeft', 300); await shot('20_pack_minus');
    await press('ArrowDown', 300); await press('ArrowDown', 300); await shot('21_pack_pouch');
    await press('KeyE', 400); await shot('22_lamp_done'); await press('Enter', 300); await shot('23_lamp_done_msg');
    await press('ArrowDown', 300); await shot('24_lamp_next');
    await press('ArrowDown', 300); await shot('25_lamp_first');
    await press('KeyE', 400); await shot('26_notes');
    for (let i = 0; i < 6; i++) await press('ArrowDown', 200);
    await shot('27_notes_neighbor');
    for (let i = 0; i < 12; i++) await press('ArrowDown', 120);
    await shot('28_notes_end');
    await press('KeyQ', 400); await shot('29_tab_back');     // 앞 탭으로: 등불
    // 보리에게 셋째 등불(300): 장면이 나오고, 끝나면 곁에 등불이 걸린다. 남은 반딧불로는 모모의 다음 이야기(200)가 모자란다.
    await press('ArrowDown', 300); await press('Enter', 1500); await shot('30_lamp_scene');
    for (let i = 0; i < 30; i++) await press('Enter', 500);
    await tick(1500); await shot('31_village_after_lamp');
    await press('KeyJ', 700); await press('KeyE', 400); await press('KeyE', 400);
    await press('ArrowDown', 300); await press('ArrowDown', 300); await press('Enter', 300); await shot('32_lamp_poor');
    await press('Escape', 600); await shot('33_closed');
    if (errors.length) { console.error(`${lang}: 페이지 에러 ${JSON.stringify(errors)}`); process.exitCode = 1; }
    console.log(`${lang}: 33장 → ${OUT}`);
    await ctx.close();
    await browser.close();
  }
  srv.close();
}
run();
