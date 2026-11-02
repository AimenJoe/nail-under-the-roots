// 서비스 워커(0269, 요구 2.4): 설치한 앱·두 번째 실행부터 오프라인에서도 뜨게.
//
// 캐시 이름에 빌드 SHA가 들어간다(CI가 __BUILD__ 를 build.txt와 같은 값으로 바꾼다). nail.wasm·에셋 파일 이름에 해시가 없어서
// 오래 두는 캐시는 금지(KB concepts/static-wasm-hosting.md) — 배포마다 sw.js 바이트가 바뀌어 새 워커가 깔리고, 새 캐시를 채운 뒤
// 옛 캐시를 지운다.
// - skipWaiting을 쓰지 않는다: 게임 도중 새 판 파일(음악은 들을 때 받는다)이 섞이지 않게, 새 판은 다음 실행부터.
// - index.html·build.txt·manifest는 네트워크 먼저(못 받으면 캐시), 나머지는 이 판 캐시 먼저(없으면 받아서 담는다).
// - 기록(localStorage)은 건드리지 않는다.
const BUILD = "__BUILD__";
const CACHE = "nail-" + BUILD;
// CI가 조립한 dist의 파일 목록(음악 빼고)을 __PRECACHE__ 자리에 넣는다. 로컬에서 그대로 띄우면 빈 목록 — 받는 대로 담는다.
const PRECACHE = ["./", "index.html", "nail-host.js", "mq_js_bundle.js", "macroquad-gamepads-0.1.js", "nail.wasm", "manifest.webmanifest"].concat(/*__PRECACHE__*/[]);
const NETWORK_FIRST = /(\/|index\.html|build\.txt|manifest\.webmanifest)$/;

self.addEventListener("install", (e) => {
  e.waitUntil(
    caches.open(CACHE).then((c) =>
      // 하나가 실패해도 설치는 끝낸다(못 받은 것은 처음 쓸 때 받는다).
      Promise.all(PRECACHE.map((u) => c.add(new Request(u, { cache: "reload" })).catch(() => {})))
    )
  );
});

self.addEventListener("activate", (e) => {
  e.waitUntil(
    caches.keys().then((keys) => Promise.all(keys.filter((k) => k.startsWith("nail-") && k !== CACHE).map((k) => caches.delete(k))))
  );
});

self.addEventListener("fetch", (e) => {
  const req = e.request;
  if (req.method !== "GET") return;
  const url = new URL(req.url);
  if (url.origin !== location.origin) return;
  if (req.mode === "navigate" || NETWORK_FIRST.test(url.pathname)) {
    e.respondWith(
      fetch(req)
        .then((res) => {
          if (res.ok) {
            const copy = res.clone();
            caches.open(CACHE).then((c) => c.put(req, copy));
          }
          return res;
        })
        .catch(() => caches.match(req, { ignoreSearch: true }).then((hit) => hit || caches.match("index.html")))
    );
    return;
  }
  e.respondWith(
    caches.open(CACHE).then((c) =>
      c.match(req, { ignoreSearch: true }).then(
        (hit) =>
          hit ||
          fetch(req).then((res) => {
            if (res.ok && res.status === 200) c.put(req, res.clone());
            return res;
          })
      )
    )
  );
});
