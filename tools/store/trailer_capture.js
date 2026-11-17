// 상점 영상 한 장면 찍기(그림 + 소리 기록). 영어 화면(en-US), 가짜 시계로 1/30초씩 흘리며 한 칸씩 찍는다(tools/site/clip.js와 같은 방식, 0228).
// 소리는 녹음하지 않고 **기록한다**: 가짜 시계에서는 실제 소리가 칸과 맞지 않으므로, macroquad 오디오 플러그인(web/mq_js_bundle.js)의
// 함수를 감싸 "언제(게임 시계) 어느 소리를 어떤 크기로 냈다·바꿨다·멈췄다"와 소리 원본 바이트를 남긴다.
// tools/store/trailer.py가 이것으로 효과음을 칸에 맞춰 다시 섞는다(게임이 낸 그대로의 타격감).
//
//   cargo build --release --target wasm32-unknown-unknown --features clip    # 피해 없음 + 자동 조종(src/autopilot.rs)
//   python3 tools/web/assemble.py <dist> local dev && cp target/wasm32-unknown-unknown/release/nail.wasm <dist>/
//   node tools/store/trailer_capture.js <dist> <out> "<개발 목록 이름>" <초> [auto|walk|keys] [base|full]
//     ESC=1 — 보스 만남 컷씬을 뒤로 길게 눌러 건너뛴다(그동안의 칸은 찍지 않는다). ESC_MS·ESC_AFTER로 누르는·기다리는 시간
//     (너무 길면 싸움이 이미 흘러 보스가 찍기 전에 쓰러진다)
//     WAIT=<ms> — 장면이 뜨기를 기다리는 가짜 시계 시간(기본 3000)
// 나오는 것: <out>/f_0000.png…(960×540), <out>/audio.json(사건 목록·칸 수·fps), <out>/bufs/<번호>.bin(소리 원본)
const { chromium } = require('playwright');
const http = require('http'), fs = require('fs'), path = require('path');

const [root, out, name, secs, plan = 'auto', profKind = 'base'] = process.argv.slice(2);
const F = 30, N = Math.round(+secs * F);
const PORT = +(process.env.PORT || 8193);
const types = { '.html': 'text/html', '.png': 'image/png', '.js': 'application/javascript', '.wasm': 'application/wasm', '.ttf': 'font/ttf', '.mp3': 'audio/mpeg', '.ogg': 'audio/ogg', '.txt': 'text/plain', '.webmanifest': 'application/manifest+json' };

// 도감을 다 본 기록(새 적 알림이 화면을 가리지 않게) + 마을을 다 키운 기록(full).
const kinds = ['FieldMouse', 'HouseMouse', 'SewerRat', 'Chipmunk', 'RedSquirrel', 'Shrew', 'Hedgehog', 'Toad', 'Bat', 'SporeCap', 'Chief', 'Momo', 'Doto', 'Kongi', 'Clover', 'Crayfish', 'Strider', 'Mumu'];
const base = 'nail-profile 1\nflag tutorial\nflag bori\nflag line.village_intro\n' + kinds.map((k) => `flag mob.${k}\n`).join('');
const full = base + 'flag momo\nflag doto_memory\nflag kongi\nflag half_ending\ncount clears.Mole 4\ncount momo_wins 3\n' +
  [0, 1, 2, 3, 4, 5, 6, 7, 8].map((i) => `flag decor.${i}\n`).join('') + 'count glowbugs_banked 300\n';

// 개발 목록: 이름 → [묶음 탭, 탭 안 자리] (src/scenario.rs, tools/test/smoke.js와 같은 읽기).
const DEV = (() => {
  const src = fs.readFileSync(path.join(__dirname, '..', '..', 'src', 'scenario.rs'), 'utf8');
  const all = src.slice(src.indexOf('pub const ALL'), src.indexOf('];', src.indexOf('pub const ALL')));
  const cats = [...all.matchAll(/Cat::(\w+)/g)].map((m) => m[1]);
  const list = src.slice(src.indexOf('pub const LIST'));
  const seen = {}; const o = {};
  for (const m of list.matchAll(/cat: Cat::(\w+),\s*name: "(.*?)"/g)) {
    const c = cats.indexOf(m[1]); seen[c] = (seen[c] || 0); o[m[2]] = [c, seen[c]++];
  }
  return o;
})();

// 오디오 플러그인을 감싸는 코드(번들 끝에 붙인다 — `plugins`는 번들의 전역, register_plugin은 wasm을 만들기 전에 불린다).
const HOOK = `
;(function(){
  window.__alog = []; window.__abuf = {};
  var p = plugins.find(function(x){ return x.name === 'macroquad_audio'; });
  var orig = p.register_plugin;
  p.register_plugin = function(io){
    orig(io);
    var e = io.env, add = e.audio_add_buffer, play = e.audio_play_buffer, sv = e.audio_source_set_volume,
        st = e.audio_source_stop, pst = e.audio_playback_stop, psv = e.audio_playback_set_volume;
    e.audio_add_buffer = function(ptr, len){ var id = add(ptr, len); window.__abuf[id] = new Uint8Array(wasm_memory.buffer.slice(ptr, ptr + len)); return id; };
    e.audio_play_buffer = function(k, v, loop){ var pb = play(k, v, loop); window.__alog.push(['play', Date.now(), k, v, loop ? 1 : 0, pb]); return pb; };
    e.audio_source_set_volume = function(k, v){ window.__alog.push(['svol', Date.now(), k, v]); return sv(k, v); };
    e.audio_source_stop = function(k){ window.__alog.push(['stop', Date.now(), k]); return st(k); };
    e.audio_playback_stop = function(pb){ window.__alog.push(['pstop', Date.now(), pb]); return pst(pb); };
    e.audio_playback_set_volume = function(pb, v){ window.__alog.push(['pvol', Date.now(), pb, v]); return psv(pb, v); };
  };
})();
`;

const at = DEV[name];
if (!at) { console.error('개발 목록에 없다:', name); process.exit(1); }

const srv = http.createServer((q, r) => {
  let p = path.join(root, decodeURIComponent(q.url.split('?')[0]));
  if (p.endsWith('/')) p += 'index.html';
  fs.readFile(p, (e, d) => {
    if (e) { r.writeHead(404); r.end(); return; }
    // 번들 끝의 quad_net 플러그인이 ReferenceError로 멈춰(web/README.md) 끝에 붙이면 돌지 않는다 — 오디오 플러그인 바로 뒤에 끼운다.
    if (p.endsWith('mq_js_bundle.js')) {
      const mark = 'name:"macroquad_audio"})})(),', s = d.toString('utf8'), i = s.indexOf(mark);
      if (i < 0) throw new Error('번들에서 오디오 플러그인 자리를 못 찾았다');
      d = Buffer.from(s.slice(0, i + mark.length) + HOOK.trim().replace(/^;/, '').replace(/;$/, '') + ',' + s.slice(i + mark.length));
    }
    r.writeHead(200, { 'Content-Type': types[path.extname(p)] || 'application/octet-stream' }); r.end(d);
  });
}).listen(PORT, async () => {
  fs.mkdirSync(path.join(out, 'bufs'), { recursive: true });
  const b = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist', '--autoplay-policy=no-user-gesture-required'] });
  const ctx = await b.newContext({ viewport: { width: 960, height: 540 }, locale: 'en-US' });
  await ctx.addInitScript((p) => { try { localStorage.clear(); localStorage.setItem('nail.profile', p); } catch (e) {} }, profKind === 'full' ? full : base);
  const pg = await ctx.newPage();
  pg.on('pageerror', (e) => console.error('pageerror', String(e)));
  await pg.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'load' });
  await pg.waitForFunction(() => document.getElementById('loading').classList.contains('done'), null, { timeout: 60000 });
  await pg.waitForTimeout(800); await pg.focus('#glcanvas');
  await pg.clock.install();
  const tick = (ms) => pg.clock.runFor(ms);
  const press = async (k, ms = 200) => { await pg.keyboard.down(k); await tick(50); await pg.keyboard.up(k); await tick(ms); };
  await press('Enter', 900); await press('ArrowUp', 300); await press('Enter', 800);
  for (let i = 0; i < at[0]; i++) await press('ArrowRight', 120);
  for (let i = 0; i < at[1]; i++) await press('ArrowDown', 120);
  await press('Enter', +(process.env.WAIT || 3000));
  if (process.env.ESC) { await pg.keyboard.down('Escape'); await tick(+(process.env.ESC_MS || 1100)); await pg.keyboard.up('Escape'); await tick(+(process.env.ESC_AFTER || 300)); }
  if (profKind === 'full') { await pg.keyboard.down('Escape'); await tick(2500); await pg.keyboard.up('Escape'); await tick(1200); }
  const t0 = await pg.evaluate(() => Date.now());
  const dirs = ['ArrowUp', 'ArrowRight', 'ArrowDown', 'ArrowLeft'];
  let held = new Set(), acc = 0;
  for (let n = 0; n < N; n++) {
    const want = new Set();
    if (plan === 'walk') { const t = n / F, c = Math.floor(t / 2.0); if (t % 2.0 < 1.6) want.add(dirs[c % 4]); }
    // KEYS="ArrowLeft:0.9,-:1.5,ArrowUp:0.6" — 키:초 차례(- = 손 놓기). 정해 둔 길로 걷는다(아무 데로나 걸으면 벽에 박았다).
    if (plan === 'keys') {
      let t = n / F;
      for (const part of (process.env.KEYS || '').split(',').filter(Boolean)) {
        const [k, d] = part.split(':'); if (t < +d) { if (k !== '-') want.add(k); break; } t -= +d;
      }
    }
    for (const k of held) if (!want.has(k)) await pg.keyboard.up(k);
    for (const k of want) if (!held.has(k)) await pg.keyboard.down(k);
    held = want;
    const next = Math.round((n + 1) * 1000 / F); await pg.clock.runFor(next - acc); acc = next;
    await pg.screenshot({ path: path.join(out, `f_${String(n).padStart(4, '0')}.png`) });
  }
  const { log, bufs } = await pg.evaluate(() => {
    const bufs = {};
    for (const k of Object.keys(window.__abuf)) { let s = ''; const a = window.__abuf[k]; for (let i = 0; i < a.length; i += 0x8000) s += String.fromCharCode.apply(null, a.subarray(i, i + 0x8000)); bufs[k] = btoa(s); }
    return { log: window.__alog, bufs };
  });
  for (const [k, v] of Object.entries(bufs)) fs.writeFileSync(path.join(out, 'bufs', `${k}.bin`), Buffer.from(v, 'base64'));
  fs.writeFileSync(path.join(out, 'audio.json'), JSON.stringify({ fps: F, frames: N, t0, events: log }));
  console.log(`${name}: ${N} frames, ${log.length} audio events, ${Object.keys(bufs).length} buffers`);
  await ctx.close(); await b.close(); srv.close();
});
