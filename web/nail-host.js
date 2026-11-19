// 웹 셸의 게임 쪽 다리(0269 R37: index.html 안에 있던 플러그인을 파일로). mq_js_bundle.js·패드 플러그인 다음에 읽고,
// 끝에서 nail.wasm을 띄운다. 플러그인 이름·버전은 Rust 쪽 `<이름>_crate_version`과 맞춘다(mq_js_bundle.js init_plugins).
// 서비스 워커(sw.js)가 미리 받는 목록에 이 파일도 있다.
(function () {
"use strict";
// 브라우저 선호 언어 → 게임 언어(src/i18n.rs가 고른다: 가진 첫 언어, 없으면 영어).
// nail_locale(i, k): i번째 선호 언어 태그(예 "zh-Hant-TW")의 k번째 네 글자를 u32 하나로(빅엔디언). 끝이면 0. ASCII만.
miniquad_add_plugin({
  name: "nail_locale",
  version: 2,
  register_plugin: function (importObject) {
    importObject.env.nail_locale = function (i, k) {
      var langs = navigator.languages && navigator.languages.length ? navigator.languages : [navigator.language];
      var tag = String(langs[i] || "").replace(/[^A-Za-z0-9_.@-]/g, "").slice(k * 4, k * 4 + 4);
      var w = 0;
      for (var j = 0; j < 4; j++) w = (w * 256) + (j < tag.length ? tag.charCodeAt(j) : 0);
      return w >>> 0;
    };
  }
});
// 설정 저장(src/prefs.rs): 정수 칸 i ↔ localStorage "nail.pref.i". 없으면 0. 사생활 보호 모드 등에서 막히면 저장하지 않는다.
miniquad_add_plugin({
  name: "nail_prefs",
  version: 1,
  register_plugin: function (importObject) {
    importObject.env.nail_pref_get = function (i) {
      try { var v = parseInt(window.localStorage.getItem("nail.pref." + i), 10); return isNaN(v) ? 0 : (v >>> 0); } catch (e) { return 0; }
    };
    importObject.env.nail_pref_set = function (i, v) {
      try { window.localStorage.setItem("nail.pref." + i, String(v >>> 0)); } catch (e) {}
    };
  }
});
// 기록(프로필) 저장(src/profile.rs): 저장 슬롯 셋 + 마지막 슬롯 번호(0078). wasm 메모리에서 UTF-8 바이트를 직접 읽고 쓴다(wasm-bindgen 없이).
miniquad_add_plugin({
  name: "nail_save",
  version: 2,
  register_plugin: function (importObject) {
    // 칸 0 = 마지막으로 쓴 슬롯 번호, 1~3 = 저장 슬롯(src/profile.rs, 0078). 슬롯 1은 예전 한 칸 저장 열쇠 그대로.
    // 칸 4~6 = 슬롯마다 진행 중인 런(쉼터부터 이어하기, 0080).
    // 칸 7~9 = Lab 프리셋(개발판만, 0269) — 진행 기록과 따로.
    var key = function (cell) { return cell === 0 ? "nail.slot" : cell === 1 ? "nail.profile" : cell <= 3 ? "nail.profile." + cell : cell <= 6 ? "nail.run." + (cell - 3) : "nail.lab." + (cell - 6); };
    var get = function (cell) { try { return window.localStorage.getItem(key(cell)) || ""; } catch (e) { return ""; } };
    importObject.env.nail_save_len = function (cell) { return new TextEncoder().encode(get(cell)).length; };
    importObject.env.nail_save_read = function (cell, ptr, len) {
      var bytes = new TextEncoder().encode(get(cell));
      new Uint8Array(wasm_memory.buffer, ptr, len).set(bytes.subarray(0, len));
    };
    importObject.env.nail_save_write = function (cell, ptr, len) {
      var text = new TextDecoder().decode(new Uint8Array(wasm_memory.buffer, ptr, len));
      try { window.localStorage.setItem(key(cell), text); } catch (e) {}
    };
  }
});
// 로딩 화면 갱신(src/loading.rs): 준비 단계, 첫 프레임을 그림.
miniquad_add_plugin({
  name: "nail_loading",
  version: 1,
  register_plugin: function (importObject) {
    importObject.env.nail_load_step = function (done, all) { window.__nailLoading.step(done, all); };
    importObject.env.nail_load_ready = function () { window.__nailLoading.ready(); };
  }
});
// 처음 안내 글의 장치(src/input.rs): 주 포인터가 굵으면(손가락) 터치로 시작. 키·패드를 누르면 그쪽으로 바뀐다.
miniquad_add_plugin({
  name: "nail_device",
  version: 1,
  register_plugin: function (importObject) {
    importObject.env.nail_coarse_pointer = function () {
      try { return window.matchMedia("(pointer: coarse)").matches ? 1 : 0; } catch (e) { return 0; }
    };
  }
});

// ── PWA·화면(0269, 요구 2.4) ─────────────────────────────────────────────────────────────
// 전체 화면은 사용자가 누른 입력에서만(게임 설정 "전체 화면" → nail_fs_toggle). 캔버스가 아니라 문서 전체를 올려
// 회전 안내·로딩이 위에 보이게. 들어간 뒤 가로 잠금을 시도하고(Android Chrome), 안 되면 조용히 넘어간다(iPhone·데스크톱).
var ua = navigator.userAgent || "";
var ios = /iPhone|iPad|iPod/.test(ua) || (/Macintosh/.test(ua) && navigator.maxTouchPoints > 1);
function fsEnabled() { return !!(document.fullscreenEnabled || document.webkitFullscreenEnabled); }
function fsActive() { return !!(document.fullscreenElement || document.webkitFullscreenElement); }
function standalone() {
  try {
    return navigator.standalone === true || matchMedia("(display-mode: standalone)").matches || matchMedia("(display-mode: fullscreen)").matches;
  } catch (e) { return false; }
}
function lockLandscape() {
  try { if (screen.orientation && screen.orientation.lock) screen.orientation.lock("landscape").catch(function () {}); } catch (e) {}
}
function toggleFs() {
  var el = document.documentElement;
  try {
    if (fsActive()) {
      (document.exitFullscreen || document.webkitExitFullscreen).call(document);
    } else {
      var req = el.requestFullscreen || el.webkitRequestFullscreen;
      var p = req && req.call(el, { navigationUI: "hide" });
      if (p && p.then) p.then(lockLandscape).catch(function () {}); else lockLandscape();
    }
  } catch (e) {}
}
// 세로로 든 터치 기기: 글 없는 회전 그림(#rotate)을 띄우고 게임은 일시정지한다(nail_portrait). 한 번 누르면 이번 창에서는
// 세로로 계속한다(강제하지 않는다 — 세로에서도 작게 돈다).
var rotateOff = false;
try { rotateOff = sessionStorage.getItem("nail.rotate.off") === "1"; } catch (e) {}
function portraitBlocked() {
  if (rotateOff) return false;
  try { return matchMedia("(pointer: coarse)").matches && matchMedia("(orientation: portrait)").matches; } catch (e) { return false; }
}
function showRotate() {
  var r = document.getElementById("rotate");
  if (r) r.classList.toggle("on", portraitBlocked());
}
window.addEventListener("resize", showRotate);
window.addEventListener("orientationchange", showRotate);
document.addEventListener("DOMContentLoaded", showRotate);
showRotate();
var rot = document.getElementById("rotate");
if (rot) rot.addEventListener("click", function () {
  rotateOff = true;
  try { sessionStorage.setItem("nail.rotate.off", "1"); } catch (e) {}
  showRotate();
  var c = document.getElementById("glcanvas"); if (c) c.focus();
});
// 캔버스 크기 다시 맞추기(knellroot 0187·0243을 옮김 — 사장 2026-11-19 "아이폰 가로모드 버그는 nail에도 동일하게 있네, 아예 시작시부터
// 가로모드면 아래쪽에 여백 생김", iPhone 13 Pro Max). 번들은 window.onresize 때 한 번만 캔버스의 CSS 크기를 읽어 그리기 크기를 정한다
// (miniquad js/gl.js resize). iOS Safari는 처음 그릴 때·주소창이 접힐 때·돌릴 때 그 값을 레이아웃이 끝나기 전에 주기도 해서, 그때의
// (작은) 높이가 박혀 아래가 빈 채로 남았다. 번들은 고치지 않고 크기가 바뀔 만한 일 뒤에 몇 번 더 재서, 어긋나 있으면 번들의 처리기를
// 다시 부른다(ResizeObserver는 레이아웃이 끝난 크기를 준다). 로딩 중에는 처리기가 아직 없어 넘어가므로 다 그린 순간(ready)에 다시 잰다.
(function () {
  var c = document.getElementById("glcanvas");
  if (!c) return;
  function check() {
    if (typeof window.onresize !== "function") return; // 번들이 아직 처리기를 달기 전
    var d = window.high_dpi ? (window.devicePixelRatio || 1) : 1;
    if (c.width !== Math.floor(c.clientWidth * d) || c.height !== Math.floor(c.clientHeight * d)) window.onresize();
  }
  var timers = [];
  function soon() {
    timers.forEach(clearTimeout);
    timers = [50, 150, 350, 700, 1200].map(function (ms) { return setTimeout(check, ms); });
    requestAnimationFrame(check);
  }
  window.addEventListener("resize", soon);
  window.addEventListener("orientationchange", soon);
  document.addEventListener("fullscreenchange", soon);
  document.addEventListener("webkitfullscreenchange", soon);
  if (window.visualViewport) window.visualViewport.addEventListener("resize", soon);
  if (window.ResizeObserver) new ResizeObserver(check).observe(c);
  window.nailCanvasCheck = check; // 헤드리스 시험이 부를 수 있다
  // 다 그린 순간(index.html __nailLoading.ready): 한 번 더 재고, 처음 몇 초는 1초마다(주소창이 늦게 접히는 iPhone).
  window.nailCanvasSoon = function () {
    soon();
    for (var i = 1; i <= 5; i++) setTimeout(check, i * 1000);
  };
})();
miniquad_add_plugin({
  name: "nail_web",
  version: 1,
  register_plugin: function (importObject) {
    // 비트: 1 전체 화면을 쓸 수 있다, 2 설치한 앱(독립 실행)으로 열렸다, 4 iPhone·iPad(홈 화면에 추가 안내).
    importObject.env.nail_web_flags = function () { return (fsEnabled() ? 1 : 0) | (standalone() ? 2 : 0) | (ios ? 4 : 0); };
    importObject.env.nail_fs_active = function () { return fsActive() ? 1 : 0; };
    importObject.env.nail_fs_toggle = function () { toggleFs(); };
    importObject.env.nail_portrait = function () { return portraitBlocked() ? 1 : 0; };
  }
});

// 서비스 워커(sw.js): https(또는 localhost)에서만, PR 미리 보기 주소(pr-N--)에는 걸지 않는다. 새 판은 다음 실행부터(sw.js 머리 주석).
if ("serviceWorker" in navigator && (location.protocol === "https:" || location.hostname === "localhost" || location.hostname === "127.0.0.1") && !/^pr-\d+--/.test(location.hostname)) {
  window.addEventListener("load", function () { navigator.serviceWorker.register("sw.js").catch(function () {}); });
}

load("nail.wasm");
})();
