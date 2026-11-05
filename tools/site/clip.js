// 공식 페이지 게임 클립(0228): 게임을 가짜 시계(Playwright `page.clock`)로 멈춰 두고 1/30초씩 흘리며 한 칸씩 찍는다.
// 헤드리스 녹화(recordVideo)는 도트를 뭉갰고(0155) 실시간 찍기는 칸이 고르지 않다 — 이렇게 하면 30fps 고른 무손실 칸이 나온다.
// macroquad는 requestAnimationFrame·Date.now로 돌아서 가짜 시계가 그대로 먹힌다.
//
// 1) 찍기 전용 빌드(`clip` 기능): 피해를 끄고, 자동 조종(`src/autopilot.rs`)이 월드를 보고 싸운다 — 계획 `auto`(0228 덧붙임).
//    (예전 계획 nail·run은 정해 둔 키를 시간대로 눌러 허공을 쳤다.)
//      cargo build --release --target wasm32-unknown-unknown --features clip
//    dist/를 복사해 wasm만 바꿔 넣고 ROOT=<그 폴더>로 준다(dist/의 운영·개발판 wasm은 건드리지 않는다).
// 2) 장면마다 하나씩(여럿을 한꺼번에 돌리면 swiftshader가 서로 느려져 메뉴 키를 놓친다):
//      ROOT=… node tools/site/clip.js <out> <개발 시나리오 번호> <초> <nail|run|walk> [포트]
//    번호는 src/scenario.rs `LIST` 순서(22 웨이브 3부터, 41 모모: 아레나, 39 도토: 아레나, 45~47 넓은 방). WAIT=<ms>는 장면이 뜨기를 기다리는 시간 —
//    WAIT도 가짜 시계로 흐른다(시계는 메뉴부터). 말풍선은 SKIP=<초>, 보스 만남 컷씬은 ESC=1로 넘긴다.
// 3) 고른 칸을 이어 붙여(r_0000.png…) H.264로 — crf 16·tune animation이면 3배 확대 도트가 원본과 거의 같다:
//      ffmpeg -framerate 30 -i r_%04d.png -c:v libx264 -preset slow -tune animation -crf 16 -pix_fmt yuv420p -movflags +faststart site/img/play.mp4
//    첫 칸은 site/img/play_poster.png로.
const { chromium } = require('playwright');
const http = require('http'), fs = require('fs'), path = require('path');
const root = process.env.ROOT || path.join(__dirname, '..', '..', 'dist');
const [out, idx, secs, plan] = [process.argv[2], +process.argv[3], +process.argv[4], process.argv[5]];
const port = +(process.argv[6] || 8192);
const types = { '.html': 'text/html', '.png': 'image/png', '.js': 'application/javascript', '.wasm': 'application/wasm', '.ttf': 'font/ttf', '.mp3': 'audio/mpeg', '.ogg': 'audio/ogg', '.txt': 'text/plain' };
const kinds = ['FieldMouse','HouseMouse','SewerRat','Chipmunk','RedSquirrel','Shrew','Hedgehog','Toad','Bat','SporeCap','Chief','Momo','Doto','Kongi','Clover'];
const prof = 'nail-profile 1\nflag tutorial\nflag bori\nflag line.village_intro\n' + kinds.map(k => `flag mob.${k}\n`).join('');
// 계획: 프레임마다 눌린 키 집합. 30fps.
const F = 30, N = Math.round(secs * F);
function keysAt(n) {
  const t = n / F, s = new Set();
  const dirs = ['ArrowRight', 'ArrowUp', 'ArrowLeft', 'ArrowDown'];
  if (plan === 'nail') {
    // 4.2초 고리: 걷기 0.5 → 콤보 셋(박자 0.42) → 걷기 → 강타 쥐기 0.9 → 걷기 → 굴파기 0.7
    const c = Math.floor(t / 4.2), u = t % 4.2, d = dirs[c % 4];
    if (u < 0.5) s.add(d);
    else if (u < 1.8) { const v = (u - 0.5) % 0.42; if (v < 0.07) s.add('j'); }
    else if (u < 2.2) s.add(dirs[(c + 1) % 4]);
    else if (u < 2.3) s.add('j');
    else if (u < 2.4) {}
    else if (u < 3.3) s.add('j');
    else if (u < 3.5) s.add(dirs[(c + 2) % 4]);
    else if (u < 4.1) { s.add('k'); s.add(dirs[(c + 3) % 4]); }
  } else if (plan === 'run') {
    // 쥐고 달리기 → 떼서 멀리뛰기 → 공격 몇 번
    const c = Math.floor(t / 3.0), u = t % 3.0, d = dirs[c % 4];
    if (u < 1.1) { s.add('k'); s.add(d); }
    else if (u < 1.3) s.add(d);
    else if (u < 2.8) { const v = (u - 1.3) % 0.42; if (v < 0.07) s.add('j'); }
  } else if (plan === 'auto') {
    // 손은 놓고 자동 조종(`--features clip`의 autopilot, 0228)이 친다.
    // SKIP=<초>면 그동안 0.4초마다 Enter(보스 만남 말풍선 넘기기) — 그 칸은 잘라 낸다.
    // ESC=1이면 첫 칸들에서 Escape 한 번(보스 만남 컷씬 건너뛰기).
    if (process.env.ESC && n >= 3 && n < 5) s.add('Escape');
    else if (t < +(process.env.SKIP || 0) && (n % 12) < 2) s.add('Enter');
  } else if (plan === 'walk') {
    const c = Math.floor(t / 2.0), u = t % 2.0;
    if (u < 1.6) s.add(['ArrowRight', 'ArrowDown', 'ArrowLeft', 'ArrowUp'][c % 4]);
  }
  return s;
}
const srv = http.createServer((q, r) => { let p = path.join(root, decodeURIComponent(q.url.split('?')[0])); if (p.endsWith('/')) p += 'index.html'; fs.readFile(p, (e, d) => { if (e) { r.writeHead(404); r.end(); } else { r.writeHead(200, { 'Content-Type': types[path.extname(p)] || 'application/octet-stream' }); r.end(d); } }); }).listen(port, async () => {
  fs.mkdirSync(out, { recursive: true });
  const b = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
  const ctx = await b.newContext({ viewport: { width: 960, height: 540 }, locale: 'ko-KR' });
  await ctx.addInitScript((p) => { try { localStorage.clear(); localStorage.setItem('nail.profile', p); } catch (e) {} }, prof);
  const pg = await ctx.newPage();
  await pg.goto(`http://127.0.0.1:${port}/`, { waitUntil: 'load' });
  await pg.waitForFunction(() => document.getElementById('loading').classList.contains('done'), null, { timeout: 60000 });
  await pg.waitForTimeout(800); await pg.focus('#glcanvas');
  // 시계는 메뉴부터 건다(0244): 장면이 뜬 뒤에 걸면 지도를 굽던 중이거나 몇 초 돈 뒤일 때 화면이 검게 남는 일이 있었다.
  // 그 뒤로는 실시간 기다림 대신 가짜 시계를 흘린다(키 하나 = 몇 프레임).
  await pg.clock.install();
  const tick = async (ms) => { await pg.clock.runFor(ms); };
  const press = async (k, ms = 200) => { await pg.keyboard.down(k); await tick(50); await pg.keyboard.up(k); await tick(ms); };
  await press('Enter', 900); await press('ArrowUp', 300); await press('Enter', 800);
  for (let i = 0; i < idx; i++) await press('ArrowDown', 120);
  await press('Enter', +(process.env.WAIT || 3000));
  for (const k of (process.env.PRE || '').split(',').filter(Boolean)) await press(k, 700);
  let held = new Set(), acc = 0;
  for (let n = 0; n < N; n++) {
    const want = keysAt(n);
    for (const k of held) if (!want.has(k)) await pg.keyboard.up(k);
    for (const k of want) if (!held.has(k)) await pg.keyboard.down(k);
    held = want;
    const next = Math.round((n + 1) * 1000 / F); await pg.clock.runFor(next - acc); acc = next;
    await pg.screenshot({ path: `${out}/f_${String(n).padStart(4, '0')}.png` });
  }
  await ctx.close(); await b.close(); srv.close();
});
