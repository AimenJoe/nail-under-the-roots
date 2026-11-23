# 0300. 앱인토스 묶음 — 운영 웹 빌드를 .ait로, 업로드는 store-deploy `toss`

- 날짜: 2026-11-20
- 상태: **채택**(사장 2026-11-20: Environment `store`에 토스 API 키를 넣음 → 묶기·업로드 자동화)
- 바탕: 0299(앱인토스 — 먼저 등록, 등급은 나중에 문의), 0298(내려가면 일시정지·웹 소리 멈춤), 0193(운영판 = `--no-default-features`)

## 확인한 것(1차)
- `@apps-in-toss/web-framework` 3.10.0(npm, 2026-11-06)과 그 CLI 소스: `ait build`는 `webBundleDir`(기본 `dist`)의 파일을 **그대로** `<appName>.ait`에 담고 `bundle.json`(appName·deploymentId·SDK 판·설정)을 더한다 — 번들러가 필요 없다. 설정은 `apps-in-toss.config.ts`(c12로 읽어 `process.env`를 쓸 수 있다). 묶음 형식(`@apps-in-toss/ait-format`)은 **Node 24 이상**.
- `ait deploy --api-key <키> --location <파일> --scheme-only -m <메모>`: 프롬프트 없이 올리고 테스트 스킴(`intoss-private://<appName>?_deploymentId=…`)만 찍는다. 키는 콘솔 → 워크스페이스 → 키(문서 "미니앱 테스트하기").
- SDK 함수(`Screen.setOrientation`·`graniteEvent` …)는 ES 모듈이다 — 토스 밖(호스트 상수 `window.__appsInTossConstants`가 없을 때)에서는 부르자마자 던진다.

## 한 것
- `toss/`(npm 작은 꾸러미, 판 고정 + package-lock): `apps-in-toss.config.ts`(appName은 CI 변수 `TOSS_APP_NAME`, 튕김·당겨 새로 고침 끔), `bridge.js` → esbuild로 `dist/nail-toss.js`(약 67KB, eval 없음).
  - 다리가 하는 일: `window.NAIL_TOSS`를 세우고(nail-host.js가 서비스 워커를 걸지 않는다), 토스 안이면 **가로 고정**, Android **뒤로 = 게임의 Esc**(싸움 중이면 일시정지 창, 창 안이면 한 칸 뒤로 — 기본 뒤로가기는 막힌다). 미니앱 닫기는 오른쪽 위 X(토스가 종료 확인을 띄운다).
- `tools/toss/build.sh <운영 웹 빌드>`: 복사 → `sw.js`·`_headers` 빼기 → 다리 넣기(`index.html`의 nail-host.js 앞) → 100MB 한도 검사 → `ait build`.
- `store-deploy` **toss**: 운영판 wasm + `assemble.py release` → `.ait`(아티팩트 `nail-toss-<sha>`) → `ait deploy`, 테스트 스킴을 실행 요약에. Environment `store`의 Secret `TOSS_API_KEY`·Variable `TOSS_APP_NAME`.
- 로컬 확인(2026-11-20): `.ait` 45MB(웹 47MB, 팩 그림 없는 로컬 빌드), 토스 밖 Chromium에서 다리를 실어도 새 에러 없음(번들의 옛 `register_plugin` 잡음만), 서비스 워커 등록 0, 창에 보낸 Esc가 게임에 닿는다(key_down 256).

## 아직(실기·검수 전)
- 토스 앱 안에서: 가로 전환, X 버튼이 HUD와 겹치는지(안전 영역), 뒤로 키, 소리 멈춤·복귀, 10초 안 첫 화면.
- 기록 저장은 지금 localStorage — 토스는 웹 출처를 바꾼 적이 있다(`Migration.getOriginStorage`). 출시 전에 `Storage` SDK로 함께 적기를 볼 것.
- 게임 등급(0299)·검수 요청·출시는 콘솔에서.
