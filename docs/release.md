# 배포와 릴리스

한 장으로 끝나는 절차. 배경은 README "배포", `.github/workflows/*.yml`의 머리 주석, KB `maps/ci-cd-hosting.md`.

## 무엇이 어디로 나가나
| 대상 | 트리거 | 워크플로 | 결과 |
|---|---|---|---|
| 웹(게임) | `main` push | `deploy-web.yml` | fmt·clippy(두 갈래)·test·운영판 wasm 빌드(`--no-default-features`) → Netlify 프로덕션(https://nail-under-the-roots.netlify.app). PR이면 미리보기 `pr-<번호>--…` |
| 웹(개발판, 0193) | `release/**` push | `deploy-web.yml` | 개발 기능을 넣은 빌드 → 별칭 `dev`(https://dev--nail-under-the-roots.netlify.app, noindex). 올린 뒤 `build.txt`가 그 커밋인지 확인 |
| APK(개발판, 0193) | `release/**` push(`src/`·`assets/`·…) | `build-android.yml` | 패키지 `io.github.aimenjoe.dev.nail`("Naïl dev" — 마지막 조각은 `nail`이어야 한다: 네이티브 라이브러리 이름, 0286) → 미리 보기 Release `dev`의 `nail-dev.apk`를 갈아 끼움(태그 `dev`도 그 커밋으로) |
| Release + APK + PC | `main` push에 `Cargo.toml` 버전이 새 값 **그리고** `docs/releases/v<버전>.md`가 있음 | `release.yml` → `build-android.yml` → `build-desktop.yml` | 태그 `v<버전>`, GitHub Release(본문 = 그 md), `nail.apk` + PC 셋 첨부 |
| APK(시험용) | `main` push(`src/`·`assets/`·`android/`·`third_party/`·`Cargo.*`), 또는 Actions → build-android → Run workflow에서 브랜치를 골라 수동 실행(0169 R8) | `build-android.yml` | Actions 아티팩트 `nail-apk-<release 또는 dev>-<sha>`(main은 운영판, 수동 실행한 작업 브랜치는 개발판) |
| 웹 연기 시험 | `main` push(`src/`·`assets/`·`web/`·`Cargo.*`·`tools/test/`), 또는 수동 | `smoke-web.yml` | 헤드리스 Chromium으로 타이틀·개발 시나리오 아홉 개(콘솔 에러·패닉·멈춘 화면), 스크린숏 아티팩트 `smoke-shots` |
| Android 에뮬레이터 연기 시험(0286) | `release/**` push(`.github/workflows/android-smoke.yml`·`tools/test/android-smoke.sh`) | `android-smoke.yml` | 개발판 APK(Release `dev`의 `nail-dev.apk`, 그 push가 APK를 다시 만들면 그 커밋 것을 기다림)를 x86_64 에뮬레이터(API 31, ARM 번역)에 깔아 켜고 20초 뒤 살아 있나·화면 둘·logcat을 아티팩트로. 켜자마자 죽으면 실패 — 빌드·서명·badging 검사로는 못 잡는 것 |
| iOS(서명 없음) | `main`·`release/**` push | `build-ios.yml` | 아티팩트 `nail-ios-unsigned-release`/`-dev`(.ipa, 개발판은 번들 아이디 `….dev`). 기기 시험은 Sideloadly·AltStore로 직접 서명 |
| PC 묶음(0240) | `release/**` push(`src/`·`assets/`·…), 손으로, 그리고 릴리스 때 `release.yml`이 부른다 | `build-desktop.yml` | Windows zip(`Nail.exe`)·macOS zip(`Nail.app`, 겸용·임시 서명)·Linux tar.gz. 개발판은 Actions 아티팩트, 릴리스는 Release에 `nail-<os>-release.*` |
| 공식 페이지 | `main` push(`site/`) | `pages.yml` | GitHub Pages(https://nail.beardworks.io/). 최신 버전·소식은 `site.js`가 Releases API에서 읽고, 못 읽으면 HTML에 적은 값 |

## 갈래(0193)
- `main` = 운영판, 패치만. `release/v1.0` = 개발판이자 작업 갈래(v0.9.0 배포 뒤, 작업을 여기에 바로 커밋·push — 사장 2026-11-02·11-04). `release/v0.6`은 v0.7.0 배포 뒤 지웠다 — 개발판 별칭 `dev`를 두 갈래가 같이 쓰면 마지막 push가 덮는다.
- 패치를 main에 넣었으면 `git checkout release/v1.0 && git merge origin/main` 후 push.
- 마일스톤을 닫을 때: release 갈래에 main을 합쳐 두고 → 버전 0.x.0·`docs/releases/v0.x.0.md`를 release 갈래에 커밋 → 그 커밋을 `git push origin HEAD:main`(아래 절차 5). 다음 마일스톤 갈래는 새 main에서 판다.

## 버전 규칙(0079)
- **패치(0.x.y)**: 고친 것·더한 것. 사장이 기기에서 볼 것이 생기면 언제든. 사장이 "배포해"라고 할 때만.
- **마이너(0.x.0)**: 마일스톤 조건표를 마쳤을 때만. v0.6.0 "팔 수 있는 판"은 PC 빌드와 함께 2026-10-29 배포했고, v0.7.0 "놓아준 뒤"는 2026-11-02(0269), v0.8.0 "실의 기억"은 2026-11-03(0273), v0.8.1·v0.8.2는 2026-11-03, v0.9.0 "옆 굴"은 2026-11-04(0275), 현재 v0.9.0이다. 0232에 정한 나머지 판매 준비는 v0.6.x에서 진행한다. 패치는 사장이 배포를 정했을 때 낸다(0079·0227·0232).
- Android `version_code`는 **릴리스마다 하나씩** 올린다(0.3.18 = 48, 0.4.0 = 49 — 처음엔 "0.3.8 = 38"이었지만 0.3.10부터 맞지 않는다). 버전 숫자에서 만들면 0.4.0 = 40처럼 **작아져** 덮어 설치가 막힌다. 덮어 설치는 같은 서명 키 + 큰 version_code일 때만 된다.

## 릴리스 절차(v0.3.8에서 그대로 한 것)
1. 작업 브랜치에 고친 것이 모두 커밋·push돼 있고, 사장이 "v<버전>으로 배포"를 말했다.
2. `git fetch origin main && git merge-base --is-ancestor origin/main HEAD` — main이 내 브랜치의 조상인지. 아니면 먼저 main을 합친다.
3. 올린다:
   - `Cargo.toml`: `version = "0.x.y"`, `[package.metadata.android] version_code = <xy>`.
   - `docs/releases/v0.x.y.md`: 제목 `# Naïl: Under the Roots v0.x.y — <한 줄>`, "## 바뀐 것"(플레이어가 느끼는 말로, 결정 번호 없이), "## 해 보기"(웹·APK), "## 출처"(앞 판 복사), 끝에 `**English (short)**:` 한 문단. 앞 판 파일을 복사해 고치는 게 빠르다.
   - `site/index.html`·`site/en/index.html`: `data-latest-version`을 새 버전으로, `<ul class="news">`에 새 `<li>`를 맨 위에 넣고 가장 오래된 줄을 빼 현재 목록 길이를 유지한다.
   - **페이지 기능 글 대조**: 이번 판(과 지난 판들)에서 바뀐 규칙·수·설정이 페이지 글(조작·지대·이웃·마을·기능·접근성·FAQ·"다음" 칸)과 맞는지 두 언어로 본다 — 소식 한 줄만 고치면 글이 낡는다(v0.7.1에서 찾은 낡은 글 여덟: 발톱 수·재화·포포의 응원·흔들림 설정·PC 기록·"다음" 칸 등). 엔딩 뒤 이야기는 쓰지 않는다(스포일러).
   - `python3 tools/site/make_images.py` — 페이지 글꼴 서브셋(새 글자) 다시 만들기. `site/style.css`의 글꼴 해시가 같이 바뀐다.
4. 전체 검사(`CONTRIBUTING.md`의 목록 전부). `Cargo.lock`이 버전 때문에 바뀐다 — 함께 커밋.
5. 커밋 메시지 `v0.x.y: <한 줄>`. 작업 브랜치에 push → **`release/vX`(또는 패치 갈래 `fix/…`) → main PR을 만들고 사장이 Merge**(2026-11-04부터 — main은 규칙 `main-only`로 PR로만·CI 초록일 때만, 승인 0명, 아래 "파이프라인 지키기"). Claude는 Merge하지 않는다. Merge 뒤 운영 배포 잡이 `web-production` 승인을 기다리면 사장이 Approve.
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
- 웹: 문제 커밋을 `git revert`한 PR을 main에 Merge하면 다시 배포된다(Netlify UI에서 이전 배포로 "Publish deploy"도 된다).
- APK: Release는 두고 패치 버전을 하나 더 낸다. 이미 받은 사람의 덮어 설치가 되려면 version_code가 커야 한다.
- 되돌린 것은 결정 기록에 "덧붙임: 철회(새 번호)"로 남긴다.

## 필요한 Secrets
`NETLIFY_AUTH_TOKEN`·`NETLIFY_SITE_ID`(웹 — **Environment `web-production`·`web-dev`에**, 아래 "파이프라인 지키기"), `ANDROID_DEBUG_KEYSTORE_B64`(APK 서명 고정), `ASSETS_REPO`·`ASSETS_TOKEN`(비공개 에셋 레포, 만료 최대 1년 — 만료되면 받기 단계가 실패한다. 시크릿 자체가 없으면 개발판은 직접 그린 타일로 빌드되지만 운영·상점 판은 실패한다, 0277). 없는 시크릿은 그 단계만 건너뛴다.

## 파이프라인 지키기(사장 2026-11-04: "공개 레포지만 배포 파이프라인은 다른 개발자가 손댈 수 없게")
레포는 공개라 누구나 보고 포크·PR은 할 수 있다. 막아야 하는 것은 셋: **배포 갈래(main·release/**)에 직접 쓰기**, **배포 시크릿을 읽기**, **워크플로를 고쳐 시크릿을 빼돌리기**. 원리(GitHub 문서, 2026-11-04 확인): 포크 PR에는 시크릿이 가지 않는다. 같은 레포의 PR·다른 갈래는 레포 시크릿을 읽을 수 있다 — 그래서 배포 시크릿은 Environment에 두고, 그 환경은 정한 갈래에서만 열리게 한다(공개 레포면 무료 플랜에도 다 된다). 파일 경로로 push를 막는 규칙(push ruleset)은 Team 플랜 비공개 레포 전용이라 쓰지 않는다.

레포가 한 것:
- `deploy-web.yml`: 빌드·시험(`build`)과 배포(`publish`)를 나눴다. `publish`만 Environment(`web-production` = main, `web-dev` = release/**)에 들어가 Netlify 시크릿을 읽는다. PR은 빌드·시험만(PR 미리 보기 배포는 뺐다).
- `.github/CODEOWNERS`: 워크플로·릴리스 문서·배포 도구의 주인 = 사장(PR이 그 파일을 고치면 사장 검토가 걸린다).
- 앞으로 붙는 Steam·Epic 업로드(0276 S4·E4)는 처음부터 Environment `store`(main만 + 사장 승인)에서만 시크릿을 읽는다.
- 공급망·재현성(0277): 워크플로의 액션은 모두 **커밋 SHA로 고정**(`uses: 주인/이름@<40자> # v태그`). 고정 값은 Dependabot(`.github/dependabot.yml`, github-actions만, 달마다 묶음 PR 하나, `release/v1.0`으로)이 올린다 — **새 마일스톤 갈래를 만들면 `target-branch`도 바꾼다**. Rust 버전은 `rust-toolchain.toml`의 숫자 한 곳(CI는 `.github/actions/rust`가 그 줄을 읽는다).
- 운영판 확인(0277): smoke-web이 운영판 빌드도 띄워 본다(시나리오 없이 PWA·모바일·타이틀). 운영 배포도 개발판처럼 운영 주소의 `build.txt`가 이 커밋인지 본다(`tools/web/check_live.sh`).

사람이 할 것(Settings, 한 번):
1. **Rules → Rulesets** 둘(https://github.com/AimenJoe/nail-under-the-roots/settings/rules):
   - `main-only`(대상 `main`, 2026-11-05 바꿈): **Require a pull request before merging**(Required approvals = **0**) + **Require status checks to pass**(`build`, 아래 6) + Restrict deletions + Block force pushes. **Restrict updates는 끈다.** 우회 = Repository admin만.
     - 왜(2026-11-05, v0.9.1 PR #12): Restrict updates는 우회 목록 밖의 **모든** main 변경을 막아 PR Merge도 "병합할 수 없음"이 되고, 사장이 매번 "요구사항을 충족하지 않고 병합하기"(우회)를 골라야 했다. 승인 1명 이상은 쓸 수 없다 — **GitHub은 PR 작성자가 자기 PR을 승인하지 못하게** 하고, 이 세션의 GitHub 도구는 사장 계정으로 PR을 연다(작성자 = 사장). 그래서 "PR로만, CI가 초록일 때만, 승인은 0명".
     - 이렇게 해도 막히는 것: main에 직접 push(PR 없이), 빨간 CI의 Merge, 갈래 지우기·force push. Merge할 수 있는 사람 = Write 이상 = 지금 사장 한 명(4). 포크 PR은 열 수만 있고 Merge는 못 한다.
   - `release-branches`(대상 `release/**`): Restrict updates·deletions·creations·Block force pushes, 우회 = Repository admin + Claude GitHub App — 개발판은 Claude가 바로 push.
     - 왜 이 넷(2026-11-05 공개 API `GET /repos/…/rulesets/26532222`로 걸린 그대로인 것 확인): **creations가 가장 중요하다** — 개발판 환경 `web-dev`는 `release/**` 갈래면 열리므로, 아무나 `release/아무거나`를 만들 수 있으면 그 갈래에서 워크플로를 고쳐 개발판 Netlify 시크릿을 읽는다(운영과 같은 토큰을 넣었다면 운영 배포 권한까지). updates는 바로 push하는 갈래라 PR 필수 대신 우회 목록으로 쓰는 사람을 정한다. deletions·force push는 역사를 지킨다.
     - 일부러 안 거는 것: PR 필수(바로 push하는 갈래), 필수 상태 검사(검사는 push 뒤에 돈다 — 대신 `publish`가 `build`를 기다리니 빨간 커밋은 개발판에 안 나간다).
     - 알아 둘 것: Dependabot은 달마다 `release/v1.0`으로 PR을 연다(`.github/dependabot.yml`). 그 Merge도 "update"라 main에서 겪은 것처럼 "병합할 수 없음" + 우회 단추만 보일 것이다(아직 그런 PR이 온 적은 없어 보지 못함) — 개발판이니 사장이 우회해 Merge해도 된다. 우회 목록의 앱(Integration id 1236702)이 Claude 앱인지는 세션에서 앱 조회 API가 막혀 확인 못 함(Claude의 push가 들어가니 우회는 된다) — Settings → Rules에서 이름으로 볼 것.
   - 알아 둘 것(2026-11-04 확인한 만큼): GitHub API는 이 Claude 세션을 `main-only`도 우회할 수 있는 사용자로 답했다(사장 권한으로 붙는 듯). 한편 `release/v1.0` push는 한 번 "Cannot update this protected ref"가 나온 뒤 들어갔다(앱 자격으로 다시 간 듯). 그래서 사장의 Claude가 main에 직접 쓸 수 있는지는 **확인 못 함**(운영이 배포될 수 있어 일부러 시험하지 않았다). 사장 계정으로 PR을 여는 만큼 그 자격으로 Merge도 될 수 있다고 본다(시험하지 않음). `main-only`는 남(다른 사람·그 사람의 Claude)을 확실히 막고, 사장의 Claude는 문서 규칙("main은 PR로, Merge는 사장" — CLAUDE.md)과 **운영 배포 승인(2, Required reviewers = 사장)**으로 막는다.
2. **Environments → New environment**(https://github.com/AimenJoe/nail-under-the-roots/settings/environments) `web-production`: Deployment branches = Selected → `main`. **Required reviewers = 사장**(운영 배포마다 승인 버튼 — main에 무엇이 들어가도 사장이 누르기 전엔 운영에 안 나간다, 2026-11-05 걸었다. Prevent self-review는 끈다 — 켜면 Merge한 사장이 승인하지 못한다). 승인은 **브라우저에서** `deploy-web` 실행(run) 첫 화면의 노란 알림 → Review deployments → Approve and deploy(2026-11-05 v0.9.1: 링크를 GitHub 앱으로 열었을 때는 단추가 안 보였고 웹에서는 됐다). `web-dev`: Selected → `release/**`. 둘 다에 `NETLIFY_AUTH_TOKEN`·`NETLIFY_SITE_ID`를 넣고, **레포 Secrets의 같은 두 개는 지운다**(지우기 전까지는 레포 시크릿도 읽힌다 — 순서: 환경에 넣기 → 한 번 배포 확인 → 레포 것 지우기).
3. **Actions → General**: Fork pull request workflows = "Require approval for all external contributors", Workflow permissions = **Read repository contents**(쓰기가 필요한 잡은 워크플로에 따로 적혀 있다), "Allow GitHub Actions to create and approve pull requests" 끔.
4. **Collaborators**: 다른 사람에게 Write·Admin을 주지 않는다. 기여는 포크 PR로(시크릿에 닿지 않는다).
5. 계정 2단계 인증.
6. **main에 빨간 CI가 못 들어가게**(0277, 2026-11-04 설정함 — API로 `required_status_checks: build`(GitHub Actions 앱) 확인): Rules → Rulesets → `main-only` → "Require status checks to pass" 켜기 → 검사 이름 `build`(deploy-web의 빌드·시험 잡, PR에서 돈다)를 더한다. PR 화면의 검사 원이 반만 차 보이는 것은 `publish`(배포 잡)가 PR에서는 건너뛰어서다 — 정상. 사장은 우회 목록이라 꼭 필요하면 Merge 화면에서 우회를 골라 넣을 수 있다.
7. **Settings → Advanced Security(또는 Code security)**: Dependabot alerts 켜기(2026-11-04 사장이 켬) — cargo 의존성의 알려진 취약점 알림(올리기 PR은 받지 않는다, 위).

남은 틈(알고 둔다): `ASSETS_TOKEN`(비공개 에셋 레포 읽기 전용)·`ANDROID_DEBUG_KEYSTORE_B64`는 아직 레포 시크릿이다 — Write 권한자가 없으면 읽을 사람이 없다(1·4). 협업자를 들이게 되면 이 둘도 Environment로 옮긴다(빌드 워크플로마다 `environment:` 한 줄).

## 상점 업로드(Steam·Epic, 0276 S4·E4 — 실기 확인 전)
`store-deploy.yml`을 손으로 돌린다(Actions → store-deploy → Run workflow → store = steam | epic). 순서: `build-desktop.yml`이 상점 판(운영판 + `--features steam|epic`)을 묶고 → Environment **`store`**에 들어간 업로드 잡만 상점 자격을 읽어 올린다. Steam은 **베타 갈래에만**(`SetLive`, 기본 갈래로는 사람이 파트너 사이트에서 옮긴다 — SteamPipe가 자동으로 못 건다), Epic은 아티팩트에 바이너리만(라벨·출시는 사람이 포털에서).

Environment `store` 만들기(사람): Settings → Environments → New `store` → Deployment branches = `main`(+ 출시 전 베타 시험을 하려면 `release/**`), **Required reviewers = 사장**. 넣을 값:

| 종류 | 이름 | 무엇 |
|---|---|---|
| Secret | `STEAM_USERNAME` | 업로드 전용 빌더 계정 이름(앱 메타데이터 편집·변경 게시만 준 계정) |
| Secret | `STEAM_CONFIG_VDF` | 그 계정으로 한 번 `steamcmd +login <계정>` 해 Steam Guard를 지난 뒤의 `config/config.vdf`를 base64로(`base64 -w0 config.vdf`). 가끔 만료 → 다시 만들어 바꾼다 |
| Variable | `STEAM_APP_ID`·`STEAM_DEPOT_WINDOWS`·`STEAM_DEPOT_MACOS`·`STEAM_DEPOT_LINUX` | 파트너 사이트의 앱 번호·데포 번호 |
| Secret | `NAIL_EOS_PRODUCT_ID`·`NAIL_EOS_SANDBOX_ID`·`NAIL_EOS_DEPLOYMENT_ID`·`NAIL_EOS_CLIENT_ID`·`NAIL_EOS_CLIENT_SECRET` | 게임 안 EOS 연결(업적)에 박는 아이디 — Epic 포털의 제품 설정(Live 샌드박스·배포, GameClient 정책의 클라이언트) |
| Secret | `EPIC_BPT_CLIENT_ID`·`EPIC_BPT_CLIENT_SECRET` | BuildPatchTool 업로드용 자격(포털에서 발급) |
| Variable | `EPIC_ORG_ID`·`EPIC_PRODUCT_ID`·`EPIC_ARTIFACT_ID` | 조직·제품·아티팩트 아이디. 아티팩트는 오퍼(기본 게임)마다 하나가 자동으로 생기고, Windows·macOS 바이너리를 같은 아티팩트에 올린 뒤 포털에서 바이너리마다 플랫폼을 고른다(Manage Platforms). `BuildVersion`은 아티팩트 안에서 플랫폼까지 겹치면 안 돼서 workflow가 끝에 `-windows`·`-macos`를 붙인다(Epic Manage Artifacts·BPT 원문, 2026-11-06 — 처음엔 Steam 데포처럼 OS마다 따로라고 짐작해 둘로 나눴다) |

Epic 포털에서 받는 큰 파일 둘(EOS **C SDK** zip·BuildPatchTool zip)은 비공개 에셋 레포(`ASSETS_REPO`)의 **Release `eos`에 첨부**한다 — 공개 레포에 두지 않고, 파일이 웹 업로드 25MB·git 100MB 한도를 넘어 레포 대신 Release(첨부 하나에 2GB까지)에 둔다(2026-11-06). zip 이름은 아무거나, 새 판을 받으면 같은 Release의 첨부를 바꾼다(태그 `eos`는 그대로). `tools/epic/fetch_eos.sh`가 `ASSETS_TOKEN`으로 받아 풀고 찾는다: Windows Epic 빌드는 SDK 뿌리(`Include`·`Lib/EOSSDK-Win64-Shipping.lib`·`Bin/EOSSDK-Win64-Shipping.dll`)를 `EOS_SDK_DIR`로 링크하고 `.dll`을 실행 파일 옆에 둔다(`tools/pc/package.sh`), `store-deploy`의 epic 잡은 경로에 `Linux`가 든 `BuildPatchTool`을 쓴다. 푼 파일은 작업 폴더 `.eos/`에만 — 묶음에 실리지 않는다. Steam 재배포 파일은 크레이트가 빌드 때 내놓은 것을 실행 파일 옆에 둔다.

Steam 파트너 사이트에서 사람이: 데포 셋(OS마다), 실행 옵션(Windows `Nail.exe`, macOS `Nail.app`, Linux `nail`), 업로드 뒤 빌드 목록에서 기본 갈래로 옮기기. Epic 포털에서: 올린 바이너리마다 플랫폼(Windows·macOS)과 업데이트 종류, 실행 파일·인자, 활성화·라벨, EOS·Epic 상점 업적 일괄 올리기(`python3 tools/epic/make_eos_zip.py` → EOS 탭 zip, `--egs` → Epic Games Store 탭 zip — 정의 표에 XP 칸 `user_epic_achievements_xp`, 값은 `docs/epic/epic-store-xp.csv`). 한국에 내면 업데이트마다 content notes.
