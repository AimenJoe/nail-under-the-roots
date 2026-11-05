// 웹 빌드 연기 시험(0169 R7). 헤드리스 Chromium으로 웹 빌드를 띄워 타이틀 → 개발 메뉴 → 시나리오 몇 개에 들어가
// 입력을 넣고 몇 초 돌린 뒤 스크린숏을 남긴다. 콘솔 에러·패닉·화면이 한 색으로 멈춤을 잡는다(단위 테스트가 못 보는 wasm·JS 번들·로딩 화면).
//   cargo build --release --target wasm32-unknown-unknown
//   python3 tools/web/assemble.py dist local dev
//   node tools/test/smoke.js dist [출력 폴더] [개발 목록 이름,... | none]
// CI: .github/workflows/smoke-web.yml. 이름은 src/scenario.rs LIST의 name(타이틀 "개발"의 묶음 탭 안 줄).
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'smoke-out');
// 기본: 튜토리얼 보스·쉼터·필드·던전·적 다섯 종·마지막 쉼터·가장 깊은 곳·마을·토끼풀.
// 기본: 튜토리얼 보스·쉼터·필드·던전·적 다섯 종·마지막 쉼터·가장 깊은 곳·마을·실타래 굴(흰 실). 이름으로 고른다(0269: 개발 목록이
// 묶음 탭으로 갈려 번호가 바뀐다 — src/scenario.rs의 LIST에서 묶음과 그 안의 자리를 읽는다).
// `none`이면 시나리오 없이 PWA·모바일·타이틀만 본다 — 개발 메뉴가 없는 운영판(--no-default-features) 빌드용(0277).
const SCENARIO_ARG = process.argv[4];
const SCENARIOS = SCENARIO_ARG === 'none' ? []
  : (SCENARIO_ARG || '튜토리얼 보스,쉼터,필드(모험),던전 바로,적 다섯 종,마지막 쉼터,가장 깊은 곳,마을,실타래 굴: 흰 실').split(',');
// 개발 목록의 묶음 순서(scenario::Cat::ALL)와 줄: { 이름 → [묶음 번호, 묶음 안 자리] }.
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
const PORT = 8129;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm', '.png': 'image/png',
  '.ttf': 'font/ttf', '.mp3': 'audio/mpeg', '.ogg': 'audio/ogg', '.txt': 'text/plain; charset=utf-8', '.md': 'text/markdown',
  '.webmanifest': 'application/manifest+json' };
// 한 색으로 멈춘 화면은 PNG가 아주 작다(검정 320×180×3 ≈ 1KB). 게임 화면은 수십 KB.
const MIN_SHOT_BYTES = 6000;
// 알려진 잡음(실패 아님). mq_js_bundle.js(macroquad 0.4.16, 상류와 바이트 같음)의 quad_net 플러그인이 "use strict" 아래에서
// 선언 없는 `register_plugin=`에 대입해 번들 끝에서 ReferenceError가 난다. 이 게임은 quad-net을 안 써 영향 없다(web/README.md).
const KNOWN_NOISE = [
  /ReferenceError: register_plugin is not defined/,
  // 유료 팩(assets/vendor, 0010)이 없는 빌드는 팩 그림을 찾다 404를 받고 직접 그린 그림으로 간다 — 정상.
  /404 \(Not Found\) @ .*\/assets\/vendor\//,
];
// 타이틀 메뉴가 다 보이게(이어하기·저장 슬롯·…·개발): 튜토리얼을 마친 프로필.
const PROFILE = 'nail-profile 1\nflag tutorial\nflag bori\nflag line.village_intro\n';

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
  const failures = [];
  const check = (cond, msg) => { if (!cond) { failures.push(msg); console.error('FAIL ' + msg); } else console.log('ok   ' + msg); };

  async function open(name) {
    const ctx = await browser.newContext({ viewport: { width: 960, height: 540 }, locale: 'ko-KR' });
    await ctx.addInitScript((p) => { try { localStorage.setItem('nail.profile', p); } catch (e) {} }, PROFILE);
    const page = await ctx.newPage();
    const errors = [];
    const note = (t) => { if (!KNOWN_NOISE.some((re) => re.test(t))) errors.push(t); };
    page.on('console', (m) => { if (m.type() === 'error') note(`${m.text()} @ ${(m.location() || {}).url || ''}`); });
    page.on('pageerror', (e) => note(String(e)));
    await page.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'load' });
    // 로딩 화면(web/index.html)은 게임이 첫 프레임을 그리면 .done이 된다(src/loading.rs).
    await page.waitForFunction(() => document.getElementById('loading').classList.contains('done'), null, { timeout: 60000 });
    await page.waitForTimeout(800);
    await page.focus('#glcanvas');
    const key = async (k, ms = 250) => { await page.keyboard.press(k); await page.waitForTimeout(ms); };
    const shot = async (tag) => { const f = path.join(OUT, `${name}_${tag}.png`); await page.screenshot({ path: f }); return fs.statSync(f).size; };
    return { ctx, page, key, shot, errors };
  }

  // 타이틀: 인트로 넘기기 → 첫 항목에서 위로 가면 마지막 항목(개발)으로 감긴다 → 묶음 탭(오른쪽) → 그 안의 줄(아래).
  async function dev(name) {
    const at = DEV[name];
    if (!at) throw new Error(`개발 목록에 없다: ${name}`);
    const s = await open(`s_${at[0]}_${at[1]}`);
    await s.key('Enter', 600);
    await s.key('ArrowUp', 200);
    await s.key('Enter', 700);
    for (let i = 0; i < at[0]; i++) await s.key('ArrowRight', 80);
    for (let i = 0; i < at[1]; i++) await s.key('ArrowDown', 60);
    await s.key('Enter', 1500);
    return s;
  }

  // PWA(0269): 매니페스트·아이콘, 서비스 워커 등록 → 오프라인으로 다시 열기, 모바일 가로 화면, 세로면 회전 안내.
  {
    const s = await open('pwa');
    const man = await s.page.evaluate(async () => {
      const r = await fetch('manifest.webmanifest'); const m = await r.json();
      const icons = await Promise.all(m.icons.map((i) => fetch(i.src).then((x) => x.status)));
      return { ok: r.ok, m, icons };
    });
    check(man.ok && man.m.name && man.m.start_url && ['fullscreen', 'standalone'].includes(man.m.display), 'pwa: 매니페스트 필드');
    check(man.m.icons.some((i) => i.sizes === '192x192') && man.m.icons.some((i) => i.sizes === '512x512') && man.icons.every((c) => c === 200), `pwa: 아이콘 192·512 ${JSON.stringify(man.icons)}`);
    const sw = await s.page.evaluate(async () => {
      if (!('serviceWorker' in navigator)) return 'none';
      const reg = await navigator.serviceWorker.ready; return reg.active ? reg.active.state : 'no-active';
    });
    check(sw === 'activated' || sw === 'activating', `pwa: 서비스 워커 ${sw}`);
    await s.page.waitForTimeout(2500); // 미리 받기가 끝날 틈
    await s.ctx.setOffline(true);
    await s.page.reload({ waitUntil: 'load' });
    await s.page.waitForFunction(() => document.getElementById('loading').classList.contains('done'), null, { timeout: 60000 });
    const size = await s.shot('offline');
    check(size > MIN_SHOT_BYTES, `pwa: 오프라인으로 다시 열어도 뜬다 (${size}B)`);
    await s.ctx.close();
  }
  for (const [tag, w, h, portrait] of [['landscape', 844, 390, false], ['portrait', 390, 844, true]]) {
    const ctx = await browser.newContext({ viewport: { width: w, height: h }, locale: 'ko-KR', hasTouch: true, isMobile: true });
    await ctx.addInitScript((p) => { try { localStorage.setItem('nail.profile', p); } catch (e) {} }, PROFILE);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((re) => re.test(String(e)))) errors.push(String(e)); });
    await page.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'load' });
    await page.waitForFunction(() => document.getElementById('loading').classList.contains('done'), null, { timeout: 60000 });
    await page.waitForTimeout(800);
    const on = await page.evaluate(() => document.getElementById('rotate').classList.contains('on'));
    check(on === portrait, `mobile ${tag}: 회전 안내 ${portrait ? '뜬다' : '없다'}`);
    const f = path.join(OUT, `mobile_${tag}.png`); await page.screenshot({ path: f });
    check(fs.statSync(f).size > MIN_SHOT_BYTES, `mobile ${tag}: 화면이 그려진다`);
    if (portrait) {
      // 한 번 누르면 이번 창에서는 세로로 계속한다.
      await page.click('#rotate');
      await page.waitForTimeout(300);
      const off = await page.evaluate(() => !document.getElementById('rotate').classList.contains('on'));
      check(off, 'mobile portrait: 누르면 안내가 닫힌다');
    }
    check(errors.length === 0, `mobile ${tag}: 에러 없음 ${JSON.stringify(errors)}`);
    await ctx.close();
  }
  {
    const s = await open('title');
    const size = await s.shot('intro');
    check(size > MIN_SHOT_BYTES, `title: 로딩이 끝나고 화면이 그려진다 (${size}B)`);
    check(s.errors.length === 0, `title: 콘솔 에러 없음 ${JSON.stringify(s.errors)}`);
    await s.ctx.close();
  }
  // 실제 키보드(0278, web/nail-host.js): 포커스가 캔버스 밖이어도, 키보드 앱을 거쳐 code가 비어 와도 키가 게임에 한 번씩 닿는다.
  // 번들이 게임에 넘기는 key_down을 엿본다(wasm_exports는 번들의 전역 — 내보낸 함수 표를 복사해 하나만 감싼다).
  {
    const s = await open('keys');
    await s.page.evaluate(() => {
      const ex = wasm_exports, o = {}; for (const k in ex) o[k] = ex[k];
      window.__kd = []; o.key_down = (c, m, r) => { window.__kd.push(c); return ex.key_down(c, m, r); }; wasm_exports = o;
    });
    const kd = () => s.page.evaluate(() => { const v = window.__kd; window.__kd = []; return v; });
    await s.page.evaluate(() => { document.activeElement.blur(); document.body.tabIndex = -1; document.body.focus(); });
    await s.key('ArrowDown', 200);
    const outside = await kd();
    const back = await s.page.evaluate(() => document.activeElement && document.activeElement.id);
    check(JSON.stringify(outside) === '[264]' && back === 'glcanvas', `keys: 캔버스 밖 포커스에서도 닿고 포커스가 돌아온다 ${JSON.stringify(outside)} ${back}`);
    // 빈 code: 한글 자모(ㅓ = J), "Unidentified" + key Enter, keyCode만(75 = K). 정상 code(KeyJ)는 번들이 그대로.
    await s.page.evaluate(() => {
      const c = document.getElementById('glcanvas');
      const send = (key, code, keyCode) => {
        const e = new KeyboardEvent('keydown', { key, code, bubbles: true });
        if (keyCode) Object.defineProperty(e, 'keyCode', { get: () => keyCode });
        c.dispatchEvent(e); c.dispatchEvent(new KeyboardEvent('keyup', { key, code, bubbles: true }));
      };
      send('ㅓ', ''); send('Enter', 'Unidentified'); send('Unidentified', '', 75); send('j', 'KeyJ');
    });
    await s.page.waitForTimeout(200);
    const empty = await kd();
    check(JSON.stringify(empty) === '[74,257,75,74]', `keys: 빈 code를 key·keyCode로 채운다(두 번 들어가지 않는다) ${JSON.stringify(empty)}`);
    await s.ctx.close();
  }
  for (const idx of SCENARIOS) {
    const s = await dev(idx);
    // 입력 한 바퀴: 이동·평타(Z) 연타·모으기·회피(X)·특기(C)·확인(Enter, 컷씬·말 걸기 넘김)·일시정지 열고 닫기.
    for (let n = 0; n < 3; n++) {
      for (const k of ['ArrowRight', 'ArrowDown', 'ArrowLeft', 'ArrowUp']) { await s.page.keyboard.down(k); await s.page.waitForTimeout(220); await s.page.keyboard.up(k); }
      for (let i = 0; i < 4; i++) await s.key('z', 90);
      await s.page.keyboard.down('z'); await s.page.waitForTimeout(700); await s.page.keyboard.up('z'); await s.page.waitForTimeout(300);
      await s.key('x', 300); await s.key('c', 400); await s.key('Enter', 400);
    }
    await s.key('Escape', 300); await s.shot('pause'); await s.key('Escape', 300);
    await s.page.waitForTimeout(1500);
    const size = await s.shot('end');
    check(size > MIN_SHOT_BYTES, `scenario ${idx}: 화면이 그려진다 (${size}B)`);
    check(s.errors.length === 0, `scenario ${idx}: 콘솔 에러·패닉 없음 ${JSON.stringify(s.errors)}`);
    // 개발 메뉴에서 연 판은 슬롯에 쓰지 않는다(0269 B3): 마을의 한마디·꾸미기·일시정지 저장이 진짜 기록을 바꾸지 않는다.
    const kept = await s.page.evaluate(() => localStorage.getItem('nail.profile'));
    check(kept === PROFILE, `scenario ${idx}: 개발 판이 슬롯 기록을 바꾸지 않는다`);
    await s.ctx.close();
  }
  await browser.close();
  srv.close();
  if (failures.length) { console.error(`\n${failures.length} failed`); process.exit(1); }
  console.log('\nsmoke ok');
}
run().catch((e) => { console.error(e); process.exit(1); });
