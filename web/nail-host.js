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

// ── iPhone·iPad: touchstart의 preventDefault 막기(0281) ─────────────────────────────────────
// 번들은 캔버스 touchstart마다 preventDefault를 부른다. WKWebView(홈 화면 웹앱·앱 안 브라우저)에서는 그러면 실제 키보드의
// keydown이 아예 오지 않고(WebKit 버그 206887, Safari 탭은 된다), iPadOS는 화면을 탭할 때까지 페이지에 키보드 포커스를 주지
// 않는데(버그 276640) 그 탭도 막혀 키가 영영 안 들어온다. 그래서 iOS에서만 touchstart 이벤트의 preventDefault를 아무것도
// 안 하는 것으로 덮는다(창 capture가 번들 처리기보다 먼저 돈다). 스크롤·확대는 CSS touch-action: none이, 흉내 클릭은 번들의
// touchend preventDefault가 그대로 막는다. touchmove·touchend는 건드리지 않는다.
if (ios) {
  window.addEventListener("touchstart", function (e) {
    try { Object.defineProperty(e, "preventDefault", { value: function () {}, configurable: true }); } catch (err) {}
  }, { capture: true, passive: true });
}

// ── 실제 키보드를 캔버스로(0278) ───────────────────────────────────────────────────────────
// 번들은 `canvas.onkeydown/onkeyup`만 듣고 `e.code`로 키를 가른다. 모바일 브라우저에 블루투스 키보드를 붙이면
// (1) 포커스가 캔버스 밖(문서 몸통·브라우저 UI)에 있어 키가 캔버스에 오지 않거나 — 터치 처리가 기본 동작을 막아
// 화면을 짚어도 포커스가 돌아오지 않는다 — (2) 키보드 앱(IME)을 거쳐 `code`가 비어("" · "Unidentified") 온다.
// 그래서: 캔버스를 만지면 포커스를 되돌리고, 창 전체에서 키를 받아(capture) 캔버스 밖이거나 `code`가 빈 것만
// `key`·`keyCode`(한글 두벌식 자모 포함)로 `code`를 채워 번들 처리기에 넘긴다. 정상인 것은 건드리지 않는다.
(function () {
  var canvas = document.getElementById("glcanvas");
  if (!canvas) return;
  function refocus() {
    if (document.activeElement !== canvas) { try { canvas.focus({ preventScroll: true }); } catch (e) {} }
  }
  ["touchstart", "pointerdown", "mousedown"].forEach(function (t) { canvas.addEventListener(t, refocus, { passive: true }); });
  window.addEventListener("focus", refocus);
  // 한글 입력 상태의 두벌식 자모 → 그 자리의 영문 키(쌍자음·ㅒ·ㅖ는 Shift 자리).
  var JAMO = { "ㅂ": "Q", "ㅈ": "W", "ㄷ": "E", "ㄱ": "R", "ㅅ": "T", "ㅛ": "Y", "ㅕ": "U", "ㅑ": "I", "ㅐ": "O", "ㅔ": "P",
    "ㅁ": "A", "ㄴ": "S", "ㅇ": "D", "ㄹ": "F", "ㅎ": "G", "ㅗ": "H", "ㅓ": "J", "ㅏ": "K", "ㅣ": "L",
    "ㅋ": "Z", "ㅌ": "X", "ㅊ": "C", "ㅍ": "V", "ㅠ": "B", "ㅜ": "N", "ㅡ": "M",
    "ㅃ": "Q", "ㅉ": "W", "ㄸ": "E", "ㄲ": "R", "ㅆ": "T", "ㅒ": "O", "ㅖ": "P" };
  var NAMED = { "Enter": "Enter", "Escape": "Escape", "Esc": "Escape", "Tab": "Tab", "Backspace": "Backspace", " ": "Space",
    "Spacebar": "Space", "ArrowLeft": "ArrowLeft", "ArrowRight": "ArrowRight", "ArrowUp": "ArrowUp", "ArrowDown": "ArrowDown",
    "Left": "ArrowLeft", "Right": "ArrowRight", "Up": "ArrowUp", "Down": "ArrowDown", "Shift": "ShiftLeft",
    "Control": "ControlLeft", "Alt": "AltLeft", ",": "Comma", ".": "Period", "/": "Slash", ";": "Semicolon", "'": "Quote" };
  var KEYCODE = { 8: "Backspace", 9: "Tab", 13: "Enter", 16: "ShiftLeft", 17: "ControlLeft", 18: "AltLeft", 27: "Escape",
    32: "Space", 37: "ArrowLeft", 38: "ArrowUp", 39: "ArrowRight", 40: "ArrowDown",
    186: "Semicolon", 188: "Comma", 190: "Period", 191: "Slash", 222: "Quote" };
  function good(code) { return !!code && code !== "Unidentified"; }
  function codeOf(e) {
    if (good(e.code)) return e.code;
    var k = e.key || "";
    if (/^[a-zA-Z]$/.test(k)) return "Key" + k.toUpperCase();
    if (/^[0-9]$/.test(k)) return "Digit" + k;
    if (JAMO[k]) return "Key" + JAMO[k];
    if (NAMED[k]) return NAMED[k];
    var c = e.keyCode || e.which || 0;
    if (c >= 65 && c <= 90) return "Key" + String.fromCharCode(c);
    if (c >= 48 && c <= 57) return "Digit" + (c - 48);
    if (c >= 112 && c <= 123) return "F" + (c - 111);
    return KEYCODE[c] || "";
  }
  // 번들 처리기를 한 번 감싸, 우리가 대신 넘긴 이벤트는 번들이 빈 `code`로 또 처리하지 않게 한다.
  function bundled(type) {
    var h = canvas["on" + type];
    if (!h) return null;
    if (h.__nailOrig) return h.__nailOrig;
    var w = function (ev) { if (!ev.__nailDone) return h.call(this, ev); };
    w.__nailOrig = h;
    canvas["on" + type] = w;
    return h;
  }
  function route(type) {
    return function (e) {
      if (e.target === canvas && good(e.code)) return;
      var h = bundled(type), code = codeOf(e);
      if (!h || !code) return;
      refocus();
      e.__nailDone = true;
      h.call(canvas, {
        code: code, key: e.key, repeat: e.repeat, ctrlKey: e.ctrlKey, shiftKey: e.shiftKey, altKey: e.altKey,
        metaKey: e.metaKey, charCode: e.charCode, keyCode: e.keyCode,
        preventDefault: function () { e.preventDefault(); }
      });
    };
  }
  window.addEventListener("keydown", route("keydown"), true);
  window.addEventListener("keyup", route("keyup"), true);
})();

load("nail.wasm");
})();
