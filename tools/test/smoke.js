// 웹 빌드 연기 시험(0169 R7). 헤드리스 Chromium으로 웹 빌드를 띄워 타이틀 → 개발 메뉴 → 시나리오 몇 개에 들어가
// 입력을 넣고 몇 초 돌린 뒤 스크린숏을 남긴다. 콘솔 에러·패닉·화면이 한 색으로 멈춤을 잡는다(단위 테스트가 못 보는 wasm·JS 번들·로딩 화면).
//   cargo build --release --target wasm32-unknown-unknown
//   (dist/ 조립은 .github/workflows/deploy-web.yml의 "Assemble publish dir"과 같게)
//   node tools/test/smoke.js dist [출력 폴더] [시나리오 번호,...]
// CI: .github/workflows/smoke-web.yml. 시나리오 번호는 src/scenario.rs LIST 순서(타이틀 "개발" 목록).
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'smoke-out');
// 기본: 튜토리얼 보스·쉼터·필드·던전·적 다섯 종·마지막 쉼터·가장 깊은 곳·마을·토끼풀.
const SCENARIOS = (process.argv[4] || '0,1,3,4,13,25,26,27,36').split(',').map(Number);
const PORT = 8129;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm', '.png': 'image/png',
  '.ttf': 'font/ttf', '.mp3': 'audio/mpeg', '.ogg': 'audio/ogg', '.txt': 'text/plain; charset=utf-8', '.md': 'text/markdown' };
// 한 색으로 멈춘 화면은 PNG가 아주 작다(검정 320×180×3 ≈ 1KB). 게임 화면은 수십 KB.
const MIN_SHOT_BYTES = 6000;
// 알려진 잡음(실패 아님). mq_js_bundle.js(macroquad 0.4.16, 상류와 바이트 같음)의 quad_net 플러그인이 "use strict" 아래에서
// 선언 없는 `register_plugin=`에 대입해 번들 끝에서 ReferenceError가 난다. 이 게임은 quad-net을 안 써 영향 없다(web/README.md).
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
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
    page.on('console', (m) => { if (m.type() === 'error') note(m.text()); });
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

  // 타이틀: 인트로 넘기기 → 첫 항목에서 위로 가면 마지막 항목(개발)으로 감긴다 → 목록에서 idx번째.
  async function dev(idx) {
    const s = await open(`s${idx}`);
    await s.key('Enter', 600);
    await s.key('ArrowUp', 200);
    await s.key('Enter', 700);
    for (let i = 0; i < idx; i++) await s.key('ArrowDown', 60);
    await s.key('Enter', 1500);
    return s;
  }

  {
    const s = await open('title');
    const size = await s.shot('intro');
    check(size > MIN_SHOT_BYTES, `title: 로딩이 끝나고 화면이 그려진다 (${size}B)`);
    check(s.errors.length === 0, `title: 콘솔 에러 없음 ${JSON.stringify(s.errors)}`);
    await s.ctx.close();
  }
  for (const idx of SCENARIOS) {
    const s = await dev(idx);
    // 입력 한 바퀴: 이동·평타(J) 연타·모으기·회피(K)·특기(L)·확인(Enter, 컷씬·말 걸기 넘김)·일시정지 열고 닫기.
    for (let n = 0; n < 3; n++) {
      for (const k of ['ArrowRight', 'ArrowDown', 'ArrowLeft', 'ArrowUp']) { await s.page.keyboard.down(k); await s.page.waitForTimeout(220); await s.page.keyboard.up(k); }
      for (let i = 0; i < 4; i++) await s.key('j', 90);
      await s.page.keyboard.down('j'); await s.page.waitForTimeout(700); await s.page.keyboard.up('j'); await s.page.waitForTimeout(300);
      await s.key('k', 300); await s.key('l', 400); await s.key('Enter', 400);
    }
    await s.key('Escape', 300); await s.shot('pause'); await s.key('Escape', 300);
    await s.page.waitForTimeout(1500);
    const size = await s.shot('end');
    check(size > MIN_SHOT_BYTES, `scenario ${idx}: 화면이 그려진다 (${size}B)`);
    check(s.errors.length === 0, `scenario ${idx}: 콘솔 에러·패닉 없음 ${JSON.stringify(s.errors)}`);
    await s.ctx.close();
  }
  await browser.close();
  srv.close();
  if (failures.length) { console.error(`\n${failures.length} failed`); process.exit(1); }
  console.log('\nsmoke ok');
}
run().catch((e) => { console.error(e); process.exit(1); });
