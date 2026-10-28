# 0193. 마일스톤은 release 갈래에서, 운영판은 개발 기능 없이 · 개발판은 따로 열고 따로 깐다

- 날짜: 2026-10-28
- 상태: 채택. CONTRIBUTING "main = 배포"는 그대로, 그 앞에 마일스톤 갈래를 둔다. 0169 R8(작업 브랜치 APK는 수동 실행)은 그대로.
- 사장: "마일스톤 작업은 release/v-0-5 같은 브랜치에 머지하는 식으로 하자. 운영 중인 게임에 영향 가지 않도록. CI/CD도 이제 main 브랜치에서는 디버그 플래그 빼 주고, 개발 중인 버전에서만 디버그 넣어 주고 별도 접속 경로 및 다운로드 경로 만들자."

## 왜
- **v0.5는 큰 마일스톤이다**(도토 플레이어블 + 그림 갈아입기, 0191). 반쯤 된 그림·캐릭터가 운영 웹·APK에 섞이면 공개 데모(v0.6)를 앞둔 게임의 첫인상이 흔들린다. 운영판은 패치(v0.4.x)만 받고, 마일스톤은 따로 모아 다 됐을 때 한 번에 넘긴다.
- **개발 메뉴가 운영판에 들어가 있었다.** `dev` 기능이 기본값이라 main의 웹·APK에도 타이틀 "개발"(시나리오 바로 가기)·F3 넘침 검사·가짜 언어가 들어 있었다. 플레이어가 길을 건너뛰고 기록을 꼬이게 할 수 있다.
- **개발판을 운영판과 나란히 봐야 한다.** 같은 주소·같은 패키지면 개발판을 여는 순간 운영판 기록(브라우저 저장소·앱 폴더)을 덮는다. 주소와 패키지를 다르게 하면 저장도 저절로 갈린다.

## 결정
### 갈래
| 갈래 | 무엇 | 나가는 곳 |
|---|---|---|
| `main` | 운영판. 패치(v0.x.y)만. 버전 올리기 = 릴리스(0079) | 웹 https://nail-under-the-roots.netlify.app · Release `v*`의 `nail.apk` · 공식 페이지 |
| `release/v0.5` | 마일스톤 개발판. 작업 브랜치(`claude/…`)를 여기로 합친다 | 웹 https://dev--nail-under-the-roots.netlify.app · 미리 보기 Release `dev`의 `nail-dev.apk` · iOS 아티팩트 `nail-ios-unsigned-dev` |
| 작업 브랜치 | 하루하루의 커밋 | 없음(수동 실행하면 개발판 APK 아티팩트) |

- main에 들어간 패치는 `release/v0.5`에도 합친다(main → release, 거꾸로는 마일스톤을 닫을 때만).
- 마일스톤을 닫으면 `release/v0.5`를 main에 합치고 버전을 0.5.0으로 올린다(`docs/release.md`). 다음 마일스톤은 `release/v0.6`.
- **배포 물음은 main만.** release 갈래에 합치는 것은 개발판을 새로 고치는 일이라 묻지 않는다(사장이 바로 보게). main push·버전 올리기는 전처럼 사장에게 묻는다.

### 빌드 갈래(flavor)
- **운영판** = `--no-default-features`(개발 기능 없음). main push·`v*` 태그·`release.yml`이 부를 때. 웹·APK·iOS 모두.
- **개발판** = 기본값(`dev`). 그 밖의 모든 빌드(release/** push, 수동 실행, PR).
- `cargo clippy`는 두 갈래 다 `-D warnings`(개발 기능에만 쓰이는 것이 운영판에서 죽은 코드가 되지 않게). 처음 돌렸을 때 `Scene::Dev`·`ui::plain` 둘이 걸렸다 — 앞은 `cfg_attr(not(dev), allow(dead_code))`(운영판엔 들어가는 길이 없다), 뒤는 `cfg(any(dev, test))`.
- 타이틀 버전 글자에 개발판이면 `· dev`가 붙는다.
- 연기 시험(`smoke-web`)은 "개발" 메뉴로 장면에 들어가므로 main에서도 개발 기능을 넣은 빌드로 돈다(시험용, 나가지 않는다).

### 개발판의 따로 열기·따로 깔기
- **웹**: 같은 Netlify 사이트의 별칭 `dev`(`netlify deploy --alias=dev`, `--prod` 없이). 주소(오리진)가 달라 브라우저 기록이 운영판과 섞이지 않는다. 탭 이름에 `(dev)`, `<meta name="robots" content="noindex, nofollow">`. 같은 별칭을 다시 쓰면 그 주소가 새 배포를 가리킨다는 설명과, 옛 배포에 머문다는 포럼 답이 엇갈려서(공식 문서는 이 세션 네트워크에서 열리지 않았다) 배포 뒤 `dev--…/build.txt`가 이 커밋인지 읽어 보고 아니면 CI를 실패시킨다.
- **Android**: 패키지 `io.github.aimenjoe.nail.dev`, 이름 "Naïl dev"(CI가 빌드 안에서만 `Cargo.toml`을 바꾼다). 운영 앱과 나란히 깔리고 기록 폴더도 따로다. 기록 폴더는 `/data/data/<패키지>/files`인데 패키지 이름이 박혀 있었다 — 이제 `/proc/self/cmdline`(앱 프로세스 이름 = 패키지 이름)에서 읽는다(`profile::android_package`, 우리 두 이름 말고는 운영 이름). cargo-quad-apk는 `package_name`으로 Java 패키지와 JNI 함수 이름을 함께 짓는다(소스 확인).
- **내려받기**: 미리 보기(prerelease) Release `dev` 하나를 release/** push마다 갈아 끼운다 — 태그 `dev`를 그 커밋으로 옮기고 `nail-dev.apk`를 `--clobber`로. 주소 https://github.com/AimenJoe/nail-under-the-roots/releases/download/dev/nail-dev.apk. `releases/latest`와 공식 페이지(`site.js`, 미리 보기를 건너뛰고 넷만)는 운영판만 본다.
- **iOS**: 번들 아이디 `….dev`, 이름 "Naïl dev"로 아티팩트만.

## 확인
- 로컬: 두 갈래 clippy(wasm·Android), `cargo check`(Android·iOS, 운영판), `cargo test`(+ `android_save_dir_follows_the_package`).
- CI: 첫 `release/v0.5` push에서 개발판 웹 주소의 `build.txt`, Release `dev`의 `nail-dev.apk`, main push에서 운영판 타이틀에 "개발"이 없는지.

## 사람이 볼 것
- 개발판 주소와 APK가 운영판과 따로 열리고 따로 깔리는지(운영 앱 기록이 그대로인지).
