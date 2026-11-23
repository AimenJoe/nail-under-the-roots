// 앱인토스 다리(0300): 토스 앱 안에서만 싣는 작은 스크립트(esbuild → dist/nail-toss.js, index.html에서 nail-host.js 앞).
// 게임 코드(Rust)는 토스를 모른다 — 여기서 토스 SDK를 부르고, 게임에는 키 사건으로만 말한다.
import { Screen, graniteEvent } from "@apps-in-toss/web-framework";

window.NAIL_TOSS = true; // nail-host.js가 서비스 워커·PWA 안내를 건너뛴다

// 토스 앱 밖(로컬 브라우저 시험)에는 호스트가 넣는 상수가 없어 SDK가 바로 던진다 — 그때는 아무것도 안 한다.
const inToss = typeof window.__appsInTossConstants === "object" && window.__appsInTossConstants !== null;

// 가로 고정(체크리스트 "설정한 가로 모드가 의도대로"). 옛 토스 앱(5.215 미만)은 조용히 넘어간다.
if (inToss) {
  try { Screen.setOrientation({ type: "landscape" }).catch(() => {}); } catch (e) {}
}

// Android 뒤로(체크리스트 "뒤로가기 또는 종료"): 구독하면 기본 뒤로가기가 막히고, 게임의 뒤로(Esc)를 한 번 보낸다 —
// 싸움 중이면 일시정지 창, 창 안이면 한 칸 뒤로. 미니앱 닫기는 오른쪽 위 X(토스가 종료 확인 창을 띄운다).
const esc = (type) => window.dispatchEvent(new KeyboardEvent(type, { key: "Escape", code: "Escape", keyCode: 27, bubbles: true }));
if (inToss) try {
  graniteEvent.addEventListener("backEvent", {
    onEvent: () => { esc("keydown"); setTimeout(() => esc("keyup"), 60); },
    onError: () => {},
  });
} catch (e) {}
