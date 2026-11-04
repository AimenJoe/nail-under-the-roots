# 0277 CI 다지기 — 액션·Rust 고정, 운영판도 띄워 보기, main에 빨간 CI 막기

- 상태: 채택(사장 2026-11-04 "응 진행해" — CI/CD 검토 뒤)
- 관련: 0193(갈래), 0269(조립 스크립트), 0276(상점 업로드), `docs/release.md` "파이프라인 지키기"

## 왜
CI/CD 검토(2026-11-04)에서 구조는 범위에 맞게 단순했지만 빠진 것이 셋 나왔다.
1. **main에 빨간 CI가 들어갈 수 있다**: 규칙 `main-only`에 "status check 통과 필수"가 없어 PR 빌드·시험이 실패해도 Merge가 된다. release.yml은 태그·APK·PC 묶음을 시험 결과를 기다리지 않고 만든다.
2. **액션이 태그로만 걸려 있다**: `dtolnay/rust-toolchain@stable`·`Swatinem/rust-cache@v2`·`softprops/action-gh-release@v3`이 비공개 에셋 토큰·`contents: write`·EOS 아이디(Epic 판 빌드)와 같은 잡에서 돈다. 태그가 다른 커밋으로 옮겨지면 그 코드가 시크릿 곁에서 돈다(GitHub "Secure use reference": 커밋 SHA 고정 권장, 잠금 파일 `actions.lock`이 더 강하다고 적음).
3. **운영판 빌드를 아무도 띄워 보지 않는다**: 연기 시험은 개발 메뉴로 장면에 들어가서 개발판만 본다. 운영 배포는 개발판과 달리 올라간 커밋을 확인하지 않는다.
덤: `rust-toolchain.toml`이 `stable`이라 6주마다 새 clippy 경고가 `-D warnings`를 아무 커밋(릴리스 날 포함)에서 빨갛게 만들 수 있다. 상점·릴리스 판은 `ASSETS_REPO`가 없으면 유료 팩 그림 없이 조용히 빌드됐다.

## 정한 것
- **액션은 모두 커밋 SHA로 고정**하고 `# v태그` 주석을 단다. 올리기는 Dependabot(github-actions만, 달마다 묶음 PR 하나, `release/v1.0`으로). cargo는 Dependabot 올리기에 넣지 않는다 — 라이선스 파일·`web/*.js` 복사본·패치한 miniquad가 같이 움직여야 해서 손으로 한다. 취약점 알림만 레포 설정(Dependabot alerts)으로.
  - 잠금 파일(`.github/workflows/actions.lock`, `gh actions-lock`)은 이 레포에서 쓸 수 있는지 확인하지 못해 미뤘다. 쓸 수 있게 되면 SHA 고정 대신 그쪽으로.
- **Rust 버전은 `rust-toolchain.toml`의 숫자 한 곳**(지금 1.99.1). CI는 묶은 액션 `.github/actions/rust`가 그 줄을 읽어 `dtolnay/rust-toolchain`(SHA 고정이면 rev가 버전을 못 정해 `toolchain`을 넘긴다 — 그 레포 README, 고정 SHA는 master 역사 안)과 `rust-cache`를 부른다. 워크플로 다섯 곳의 같은 두 단계가 여기로 모였다.
- **운영판도 띄워 본다**: smoke-web이 `--no-default-features` 빌드를 따로 조립해 시나리오 없이 PWA·모바일·타이틀만(`smoke.js … none`). release/**에서 먼저 도니 main에 합치기 전에 걸린다. 배포를 막지 않는 것은 그대로.
- **운영 배포도 올라간 커밋을 확인**: `--prod --json`의 `url`(netlify-cli 23 소스로 확인)로 `tools/web/check_live.sh`. 개발판 확인도 같은 스크립트.
- **운영·상점 판은 `ASSETS_REPO`가 없으면 실패**(deploy-web main, build-android 운영판, build-desktop 릴리스·상점).
- **main에 빨간 CI 막기**는 사람이 규칙에 켠다: `main-only` → Require status checks → `build`(deploy-web). `docs/release.md` "파이프라인 지키기" 6.

## 하지 않은 것
- main에서 smoke를 건너뛰기: PR로 합치면 SHA가 늘 달라 걸리지 않고, 트리 해시 비교는 복잡도에 비해 얻는 것이 없다(smoke는 배포를 막지 않는다).
- `--with-deps` 빼기: 설치 18~41초 중 10초 남짓이라 그대로(KB `playwright-smoke-test-ci-time`, 빼도 뜨는 것은 확인함).
- 준비 단계 전체(에셋 받기·갈래 고르기)를 묶은 액션 하나로: 다음에 워크플로를 고칠 때.
