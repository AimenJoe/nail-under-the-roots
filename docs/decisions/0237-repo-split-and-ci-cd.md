# 0237. 레포를 나누면 CI/CD는 — 무엇이 깨지고 어떤 순서로 나누나

- 날짜: 2026-10-29
- 상태: **제안**(사장 "레포지토리 분리했을 때 현재의 CI/CD가 유지될지도 한번 고려해줘"). 0232 범위 1(레포 나누기)의 실행안.
- 표시: [1차] = 공식 문서·API로 확인(2026-10-29), [추정] = 그로부터 추론.

## 결론
기술적으로는 거의 그대로 돈다(같은 레포 안의 일: 태그·Release·`release.yml` → `build-android.yml` 호출·시크릿·Netlify 배포). **깨지는 곳은 셋**:
1. **Pages가 내려간다** — Free 계정에서 레포를 비공개로 바꾸면 "Any published GitHub Pages site will be automatically unpublished" [1차]. → 새 공개 레포에 Pages를 **먼저** 연다.
2. **홈페이지의 APK·소식이 막힌다** — `site.js`가 게임 레포 Releases API를 읽고 APK 링크가 게임 레포 Release를 가리킨다. 비공개 레포는 인증 없이 404 [1차: REST 문서], 익명 사용자는 APK를 못 받는다 [추정]. → 운영 태그 때 APK를 **홈페이지 레포 Release에도** 올린다(토큰 하나).
3. **Actions가 유료가 된다** — 공개 레포의 표준 러너는 무료, 비공개는 Free 계정 월 **$12**어치만 포함 [1차, docs.github.com billing/github-actions, Claude가 원문 다시 확인]. 분당 Linux $0.006, macOS $0.062 [1차]. 결제 수단이 없으면 다 쓴 뒤 막힌다 [1차].

## 지금 쓰는 양(실행 1,216건, 2026-10-24~29 API, 하위 에이전트가 재고 Claude가 원문 단가만 다시 봄)
| 워크플로 | 1회(분, 올림) | 30일 환산 |
|---|---|---|
| deploy-web | 2.3 | 4,400 Linux |
| smoke-web | 4.6 | 3,350 Linux |
| build-android | 1.8 | 2,700 Linux |
| release(job 2) | 3.1 | 1,160 Linux |
| build-ios | 2.0 | 1,700 macOS |
- 그대로 비공개면 Linux ≈11,600분($70) + macOS ≈1,700분($105) = **월 ≈ $175**, Free 몫의 15배. 최근 하루 ≈ $18 — **비공개로 바꾼 첫날 무료 몫이 끝난다**.
- 산출물(artifact) ≈1,149개·≈25GB가 90일씩 남는다(Free 저장 500MB, 넘는 몫 $0.25/GB·월) [1차 단가].
- 지금은 여러 세션이 몰아 push하는 시기라 크게 잡힌 숫자다.

## 줄이는 안 [추정]
- smoke-web·build-ios: 수동 실행·태그 때만. build-android: `release/**`·태그만(이미 그렇다). deploy-web: 작업 브랜치 push에서는 빼고 main·release만.
- 산출물 `retention-days: 3`, 옛 산출물은 API로 지운다.
- iOS는 사장의 Mac에 self-hosted runner를 두면 무료 [1차] — 쓸 때만 켠다.
- 이러면 하루 push 10번 기준 Linux ≈1,000분/월(≈$6) + iOS 수동 10번(≈$1.2) → Free $12 안 [추정].

## 나누는 순서(제안)
1. **이름**: 게임 레포 이름을 바꾸고(예: `nail-game`, 비공개), 원래 이름 `nail-under-the-roots`로 홈페이지 레포를 **새로** 만든다. → Pages 주소·`site.js`의 `REPO`·`/releases`·`/issues` 링크·og·hreflang이 그대로. 대가: 옛 이름으로 오던 리다이렉트가 끊긴다(원래 이름을 다시 쓰면 리다이렉트가 멈춘다 [1차]) → git remote·세션 설정·README·KB 링크를 고친다.
2. 공개인 지금 CI를 먼저 줄인다(위). 
3. 홈페이지 레포: `site/` + `pages.yml`. Settings → Pages → GitHub Actions(사람이 켠다).
4. 토큰: 홈페이지 레포 하나만 Contents 읽기·쓰기인 fine-grained PAT(또는 GitHub App 토큰 — 만료 없음, 문서 권장 [1차]) → 게임 레포 시크릿 `SITE_REPO_TOKEN`. `build-android`의 운영 태그 단계에 `gh release create <태그> --repo AimenJoe/<홈페이지> nail.apk`.
5. v0.5.x와 APK를 홈페이지 레포에 손으로 한 번 올려 둔다.
6. `site.js`·`index.html` 링크, `docs/releases/*.md`의 레포 링크, "소스 코드" 링크(뺀다)·의견은 홈페이지 레포 issues로. `tools/site`(make_images·shots·clip)는 **게임 레포에 남긴다**(게임 그림·빌드가 있어야 돈다) — `--site <홈페이지 체크아웃>` 인자를 받게(0232의 "tools/site 이동"을 이렇게 고친다). `docs/release.md`의 페이지 단계도.
7. 게임 레포를 비공개로.
8. 확인: deploy-web·release·홈페이지 Release에 APK, 로그아웃 상태로 `releases/latest`·Pages.

## 그대로인 것
- Netlify 배포(Actions에서 빌드 → CLI 배포): 레포 공개 여부와 무관, Netlify 빌드 분을 쓰지 않는다(KB `reference/netlify/deploy-from-ci.md`). 팀은 옛 Free 요금제 — **크레딧 요금제로 바꾸지 않는다**(운영 배포마다 15크레딧, 무료 300/월 = 20번 [1차]).
- 시크릿 `ASSETS_REPO`·`ASSETS_TOKEN`·`ANDROID_DEBUG_KEYSTORE_B64`·`NETLIFY_*`: 레포에 남는다 [추정: 공개 범위 변경 문서에 시크릿 삭제 언급 없음].
- 개발판 `dev` Release·`nail-dev.apk`: 로그인한 사장만 받는다 — 개발판은 공개하지 않는 게 맞다.
- 별·포크 0개 [1차 API] — 잃을 것 없음. 이미 공개된 역사는 회수할 수 없다.
- Claude 세션: 홈페이지 레포도 세션에 붙여야 한다(세션은 붙인 레포에만 push할 수 있다).

## 위험
- 결제 수단 없이 무료 몫을 다 쓰면 웹 배포·APK가 **조용히 멈춘다** → 예산 알림(GitHub의 budget)을 건다.
- PAT가 만료되면 홈페이지 Release가 멈춘다(App 토큰은 만료 없음).
- 웹 빌드의 wasm·그림은 레포를 나눠도 누구나 받는다(Netlify) — 데모 빌드(0232 범위 2)가 이걸 줄인다.

## 사장이 정할 것
- 레포 이름(위 1안이 링크를 가장 덜 깬다), 비공개 전환 시점.
- CI를 줄일지 / 결제 수단을 걸지(둘 다 해도 된다), iOS를 사장 Mac self-hosted로 돌릴지.
