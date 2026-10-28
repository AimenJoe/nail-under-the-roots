# 배포와 릴리스

한 장으로 끝나는 절차. 배경은 README "배포", `.github/workflows/*.yml`의 머리 주석, KB `maps/ci-cd-hosting.md`.

## 무엇이 어디로 나가나
| 대상 | 트리거 | 워크플로 | 결과 |
|---|---|---|---|
| 웹(게임) | `main` push | `deploy-web.yml` | fmt·clippy(두 갈래)·test·운영판 wasm 빌드(`--no-default-features`) → Netlify 프로덕션(https://nail-under-the-roots.netlify.app). PR이면 미리보기 `pr-<번호>--…` |
| 웹(개발판, 0193) | `release/**` push | `deploy-web.yml` | 개발 기능을 넣은 빌드 → 별칭 `dev`(https://dev--nail-under-the-roots.netlify.app, noindex). 올린 뒤 `build.txt`가 그 커밋인지 확인 |
| APK(개발판, 0193) | `release/**` push(`src/`·`assets/`·…) | `build-android.yml` | 패키지 `io.github.aimenjoe.nail.dev`("Naïl dev") → 미리 보기 Release `dev`의 `nail-dev.apk`를 갈아 끼움(태그 `dev`도 그 커밋으로) |
| Release + APK | `main` push에 `Cargo.toml` 버전이 새 값 **그리고** `docs/releases/v<버전>.md`가 있음 | `release.yml` → `build-android.yml` | 태그 `v<버전>`, GitHub Release(본문 = 그 md), `nail.apk` 첨부 |
| APK(시험용) | `main` push(`src/`·`assets/`·`android/`·`third_party/`·`Cargo.*`), 또는 Actions → build-android → Run workflow에서 브랜치를 골라 수동 실행(0169 R8) | `build-android.yml` | Actions 아티팩트 `nail-apk-<release 또는 dev>-<sha>`(main은 운영판, 수동 실행한 작업 브랜치는 개발판) |
| 웹 연기 시험 | `main` push(`src/`·`assets/`·`web/`·`Cargo.*`·`tools/test/`), 또는 수동 | `smoke-web.yml` | 헤드리스 Chromium으로 타이틀·개발 시나리오 아홉 개(콘솔 에러·패닉·멈춘 화면), 스크린숏 아티팩트 `smoke-shots` |
| iOS(서명 없음) | `main`·`release/**` push | `build-ios.yml` | 아티팩트 `nail-ios-unsigned-release`/`-dev`(.ipa, 개발판은 번들 아이디 `….dev`). 기기 시험은 Sideloadly·AltStore로 직접 서명 |
| 공식 페이지 | `main` push(`site/`) | `pages.yml` | GitHub Pages(https://aimenjoe.github.io/nail-under-the-roots/). 최신 버전·소식은 `site.js`가 Releases API에서 읽고, 못 읽으면 HTML에 적은 값 |

## 갈래(0193)
- `main` = 운영판, 패치만. `release/v0.6` = 마일스톤 개발판(작업 브랜치를 합친다, 묻지 않는다).
- 패치를 main에 넣었으면 `git checkout release/v0.6 && git merge origin/main` 후 push.
- 마일스톤을 닫을 때: release 갈래에 main을 합쳐 두고 → 버전 0.x.0·`docs/releases/v0.x.0.md`를 release 갈래에 커밋 → 그 커밋을 `git push origin HEAD:main`(아래 절차 5). 다음 마일스톤 갈래는 새 main에서 판다.

## 버전 규칙(0079)
- **패치(0.x.y)**: 고친 것·더한 것. 사장이 기기에서 볼 것이 생기면 언제든. 사장이 "배포해"라고 할 때만.
- **마이너(0.x.0)**: 마일스톤 조건표(지금은 0223, v0.5.0은 0191)를 마쳤을 때만.
- Android `version_code`는 **릴리스마다 하나씩** 올린다(0.3.18 = 48, 0.4.0 = 49 — 처음엔 "0.3.8 = 38"이었지만 0.3.10부터 맞지 않는다). 버전 숫자에서 만들면 0.4.0 = 40처럼 **작아져** 덮어 설치가 막힌다. 덮어 설치는 같은 서명 키 + 큰 version_code일 때만 된다.

## 릴리스 절차(v0.3.8에서 그대로 한 것)
1. 작업 브랜치에 고친 것이 모두 커밋·push돼 있고, 사장이 "v<버전>으로 배포"를 말했다.
2. `git fetch origin main && git merge-base --is-ancestor origin/main HEAD` — main이 내 브랜치의 조상인지. 아니면 먼저 main을 합친다.
3. 올린다:
   - `Cargo.toml`: `version = "0.x.y"`, `[package.metadata.android] version_code = <xy>`.
   - `docs/releases/v0.x.y.md`: 제목 `# Naïl: Under the Roots v0.x.y — <한 줄>`, "## 바뀐 것"(플레이어가 느끼는 말로, 결정 번호 없이), "## 해 보기"(웹·APK), "## 출처"(앞 판 복사), 끝에 `**English (short)**:` 한 문단. 앞 판 파일을 복사해 고치는 게 빠르다.
   - `site/index.html`·`site/en/index.html`: `data-latest-version`을 새 버전으로, `<ul class="news">`에 새 `<li>`를 맨 위에 넣고 가장 오래된 줄을 뺀다(세 줄 유지).
   - `python3 tools/site/make_images.py` — 페이지 글꼴 서브셋(새 글자) 다시 만들기. `site/style.css`의 글꼴 해시가 같이 바뀐다.
4. 전체 검사(`CONTRIBUTING.md`의 목록 전부). `Cargo.lock`이 버전 때문에 바뀐다 — 함께 커밋.
5. 커밋 메시지 `v0.x.y: <한 줄>`. 작업 브랜치에 push → 같은 커밋을 `git push origin HEAD:main`.
6. 확인(5~10분 뒤): Actions에서 main의 `deploy-web`·`release`·`build-android`·`pages`·`build-ios`가 모두 success, Release `v0.x.y`에 `nail.apk`(약 18 MB)가 붙었는지, 웹 타이틀 오른쪽 아래 버전 글자. Claude 세션이면 `send_later`로 확인 예약을 걸고, 확인 뒤 지운다.
7. 사장에게 보고: 무엇이 들어갔는지, 볼 것(결정 기록의 "사람이 볼 것").

## 안 되는 경우
| 증상 | 원인 | 처방 |
|---|---|---|
| `release` 워크플로가 돌았는데 Release가 없다 | `docs/releases/v<버전>.md`가 없거나 태그가 이미 있다(로그에 "릴리스하지 않는다") | 파일을 넣어 다시 push(패치 하나 더 올려도 된다). 태그가 이미 있으면 버전을 다시 올린다 — **태그를 지우지 않는다** |
| APK가 기기에서 "패키지가 유효하지 않음" | 서명 키가 바뀌었다(Secrets `ANDROID_DEBUG_KEYSTORE_B64` 없음 → 캐시 키 → 새 키) | Secrets 확인. 키가 바뀐 APK는 앱을 지우고 설치 |
| 웹은 올라갔는데 APK가 없다 | `build-android`만 실패(cargo-quad-apk·NDK·아이콘) | 로그를 본다. `CARGO_QUAD_APK_REV`·`android/res/mipmap-*`·`third_party/miniquad` 변경 여부 |
| 페이지의 버전이 옛것 | `site.js`가 Releases API를 못 읽었다(비율 제한) | HTML의 `data-latest-version`을 올려 두었으면 그 값이 보인다. 올렸는지 확인 |
| 세션에서 태그 push가 안 된다 | 세션 git 프록시는 태그를 받지 않는다 | 그래서 CI가 태그를 만든다. 손으로 만들지 않는다 |
| 이어하기·기록이 사라졌다 | `prefs` `MAGIC`을 올렸거나 `profile` 키를 바꿨다 | 키는 바꾸지 않는다. `MAGIC`은 칸 형식이 깨질 때만 |

## 되돌리기
- 웹: 문제 커밋을 `git revert`해 main에 push하면 다시 배포된다(Netlify UI에서 이전 배포로 "Publish deploy"도 된다).
- APK: Release는 두고 패치 버전을 하나 더 낸다. 이미 받은 사람의 덮어 설치가 되려면 version_code가 커야 한다.
- 되돌린 것은 결정 기록에 "덧붙임: 철회(새 번호)"로 남긴다.

## 필요한 Secrets
`NETLIFY_AUTH_TOKEN`·`NETLIFY_SITE_ID`(웹), `ANDROID_DEBUG_KEYSTORE_B64`(APK 서명 고정), `ASSETS_REPO`·`ASSETS_TOKEN`(비공개 에셋 레포, 만료 최대 1년 — 만료되면 직접 그린 타일로 빌드되고 실패하진 않는다). 없는 시크릿은 그 단계만 건너뛴다.
