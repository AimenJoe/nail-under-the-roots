# 0296 — 시작부터 가로인 iPhone에서 아래가 비는 것: 캔버스 크기 다시 재기

- 날짜: 2026-11-19
- 상태: 만듦 — iPhone 실기 확인 전(사장: iPhone 13 Pro Max)
- 사장: "아이폰 가로모드 버그는 nail에도 동일하게 있네, 아예 시작시부터 가로모드면 아래쪽에 여백 생김"
- 둘째 게임(knellroot)의 0187·0243을 옮긴다 — 같은 macroquad 번들, 같은 증상.

## 왜
번들(`mq_js_bundle.js`, miniquad `js/gl.js`)은 `window.onresize`가 올 때만 캔버스의 CSS 크기를 읽어 그리기 버퍼 크기를 정한다.
iOS Safari는 처음 그릴 때·주소창이 접힐 때·돌릴 때 그 값을 레이아웃이 끝나기 전에 주기도 해서(높이가 아직 작을 때), 그 크기가 박힌 채
더는 바로잡을 일이 없었다 — 그리기 버퍼가 화면보다 낮아 아래가 빈다. 로딩 중에는 처리기가 아직 없어 그 사이의 크기 변화도 놓쳤다.

## 정한 것(`web/nail-host.js`)
- 크기가 바뀔 만한 일(resize·orientationchange·fullscreenchange·visualViewport resize) 뒤 50·150·350·700·1200ms와 다음 프레임에
  다시 재고, 캔버스의 CSS 크기 × DPR과 그리기 크기가 다르면 번들의 `window.onresize`를 다시 부른다. `ResizeObserver`도 단다
  (레이아웃이 끝난 크기를 준다 — 창 크기 이벤트 없이 주소창만 접혀도 잡힌다).
- 로딩이 끝나는 순간(`index.html` `__nailLoading.ready`)에 한 번 더, 처음 5초는 1초마다(`window.nailCanvasSoon`).
- 번들 복사본은 고치지 않는다(web/README.md).

## 확인
- 헤드리스(844×390, DPR 3): 로딩 동안 페이지 높이를 300px로 묶어 두었다가(번들이 작은 높이를 잰다) 창 크기 이벤트 없이 풀었다 —
  옛 빌드는 그리기 버퍼가 2532×900에 남았고(아래 빈 자리), 새 빌드는 2532×1170으로 따라왔다.
- `tools/test/smoke.js` 통과.
- **iPhone 실기는 아직** — 데스크톱 Chromium은 iOS의 늦은 레이아웃을 그대로 흉내 내지 못한다.
