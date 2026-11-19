# 0294. 운영 웹판은 https://nail.beardworks.io/play/ — Cloudflare Workers 정적 파일, 옛 주소는 기록을 들고 넘긴다

- 날짜: 2026-11-18
- 상태: 채택(사장 2026-11-18: "nail.beardworks.io/play 이런식으론 못하나?" → 이 방식, Worker 이름 `web-game-nail`은 사장이 만듦). 0292(언젠가 Cloudflare로)의 nail 운영판 몫을 실행한다. 개발판은 Netlify 별칭 `dev` 그대로.
- 근거(KB): `reference/web/pwa-install-fullscreen-orientation.md`(웹 기록·PWA·서비스 워커는 출처마다 따로 — "웹 게임의 주소를 옮기지 않는다"), `reference/netlify/free-plan-limits.md`, `reference/cloudflare/static-site-extras.md`.

## 왜
- 게임 주소 `nail-under-the-roots.netlify.app`은 **Netlify에 묶인 출처**다. 0292대로 언젠가 Cloudflare로 가면 출처가 바뀌어 플레이어 기록(localStorage)·설치한 앱이 끊긴다. 어차피 한 번 옮길 거면 플레이어가 적고 상점 링크(Epic 검토 중)가 퍼지기 전인 지금이 가장 싸다.
- 옮길 곳을 **우리 도메인 아래 경로**로 두면 그 뒤로는 무엇이 내주든 출처가 바뀌지 않는다. 하위 도메인(`play-nail.…`)보다 `/play/`가 낫다: 공식 페이지와 같은 주소 아래(공유·검색), 인증서 고민 없음(Cloudflare 무료 인증서는 1단계 하위 도메인까지).
- Cloudflare Workers 정적 파일은 요청·저장이 무료·무제한이라 Netlify Legacy의 월 100 GB에서 멈추는 위험이 사라진다. `assets.base_path`(2026-11-11 나옴, Wrangler 4.148+)로 빌드 결과를 그대로 `/play/`에서 낸다.
- 공식 페이지(같은 호스트의 나머지 경로)는 GitHub Pages 그대로 — Worker는 route `nail.beardworks.io/play`·`/play/*`만 받는다. Worker 코드가 없으니 모든 요청이 정적 파일(무료)이다.

## 무엇을
- `tools/web/wrangler.jsonc`: Worker `web-game-nail`, route 둘(끝 `/` 없는 `/play`는 `/play/`로 307), `assets.base_path: "/play"`, `workers_dev`·`preview_urls` 끔.
  Wrangler가 "Will match assets: dist-play/play"라고 경고하지만 `base_path`를 셈하지 않은 옛 경고다 — `wrangler dev`(4.156.2)에서 `/play/`·`/play/nail.wasm`·`_headers`가 맞게 나오는 것을 봤다.
- `tools/web/assemble.py dist-play <id> release /play`: `_headers`의 경로 앞에 `/play`(Workers 정적 파일은 `_headers`에 공개 경로 전체를 쓴다). 게임 파일은 서로 상대 경로라 그대로.
- 서비스 워커는 페이지를 `./`로만 담는다: Cloudflare는 `/play/index.html`을 `/play/`로 돌려보내는데(307), 그 응답을 페이지 이동에 쓰면 브라우저가 거절해 오프라인 실행이 깨진다.
- **기록 넘김**: 옛 운영 주소(Netlify 프로덕션)에는 게임 대신 넘김 페이지 `web/moved/`. 그 출처의 `localStorage "nail.*"`를 JSON → base64url로 `https://nail.beardworks.io/play/#nail-import=…`에 실어 보낸다(`#` 뒤는 서버로 가지 않는다). 새 주소의 `web/nail-host.js`가 게임이 저장소를 읽기 전에 받아 넣고 `#`을 지운다. **새 주소에 기록(`nail.profile*`)이 이미 있으면 아무것도 넣지 않는다**(덮지 않는다 — 섞이면 슬롯·이어하기가 짝이 안 맞는다). 넘김 페이지는 옛 서비스 워커·캐시도 걷고(`moved/sw.js`는 스스로 내려가는 워커), 옛 주소의 다른 길은 `404.html`이 넘김 페이지로.
  - iframe으로 새 주소에서 옛 저장소를 읽는 길은 쓰지 않는다: 요즘 브라우저는 iframe 속 저장소를 출처 쌍으로 갈라 둔다.
  - 한계: 아이폰 홈 화면에 설치한 옛 앱은 저장소가 Safari와 따로라(KB 같은 노트) 그 앱으로 옛 주소를 열어야 그 기록이 넘어온다. 새 주소에서 다시 설치해야 한다.
- 배포(`deploy-web.yml`): main은 ① Cloudflare `wrangler deploy` → 새 주소의 `build.txt`가 이 커밋인지 확인 → ② 그 뒤에만 Netlify 프로덕션을 넘김 페이지로(앞이 실패하면 옛 주소의 게임이 남는다). release/**·PR은 `wrangler deploy --dry-run`(토큰 없이)으로 설정만 본다. **Cloudflare 토큰은 `web-production`(main 전용)에만** — 개발 갈래가 운영 Worker에 닿지 않게(0277의 파이프라인 지키기와 같은 까닭).
- 시험: `tools/test/moved.js`(두 출처를 흉내 내 셋 — 넘어오고 `#`이 지워짐·덮지 않음·옛 기록 없음, 2026-11-18 로컬 통과), `wrangler dev`로 서비스 워커 범위 `/play/`·오프라인 다시 열기.

## 덧붙임(2026-11-19) — Worker 이름 `web-game` → `web-game-nail`
- knellroot도 같은 꼴로 옮기며(knellroot 0246, Worker `web-game-knellroot`) 이름을 짝지었다(사장 2026-11-19, 대시보드에서 이름 바꿈 — route·주소는 그대로 이어졌다, `/play/` 200 확인). `tools/web/wrangler.jsonc`의 `name`도 같이 — 다르면 다음 배포가 옛 이름의 새 Worker를 만들려 한다(토큰은 그 Worker 하나만이라 실패).

## 사람이 할 일
- [x] Worker `web-game-nail` 만들기(사장 2026-11-18).
- [ ] API 토큰: Worker `web-game-nail`의 **Editor** + **Zone → Workers Routes → Edit**(beardworks.io). GitHub nail 레포 → Settings → Environments → **`web-production`에만** `CLOUDFLARE_API_TOKEN`·`CLOUDFLARE_ACCOUNT_ID`.
- [ ] 다음 `release/v1.0 → main` Merge 때 바뀐다(공식 페이지 링크도 같이). 그 뒤: beardworks `ops/src/config.js`의 nail `prod`를 새 주소로, 상점(Epic·Steam) 등록 정보의 웹 링크.
- [ ] (나중) knellroot도 같은 꼴 — Worker는 따로(한 Worker에 두면 하나를 올릴 때 둘 다 바뀐다).
