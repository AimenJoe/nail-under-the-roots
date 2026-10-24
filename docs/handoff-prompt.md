# 새 세션 시작 프롬프트

아래를 그대로 붙여 넣는다. 세션에는 **두 레포**(`AimenJoe/nail-under-the-roots`, `AimenJoe/KB`)를 붙인다.

---

Naïl: Under the Roots 개발을 이어간다. 레포 `AimenJoe/nail-under-the-roots`(게임)와 `AimenJoe/KB`(지식 베이스)를 둘 다 사용한다.

시작 순서:
1. 게임 레포의 `CLAUDE.md`, `docs/concept.md`, `docs/phase-1-combat-prototype.md`, `docs/decisions/*`를 읽고 현재 합의 사항을 파악한다.
2. KB의 `CLAUDE.md`와 `maps/gamedev.md`를 읽고, 작업 중 참조할 노트를 확인한다. KB에 없는 지식은 외부 자료로 교차검증하고, 남길 가치가 있으면 KB 노트로 제안한다.
3. `src/main.rs`는 320×180 렌더 타깃 + `Actions` 입력 레이어 + 움직이는 사각형만 있는 뼈대다. `cargo fmt --check`, `cargo clippy --target wasm32-unknown-unknown -D warnings`, `cargo build --release --target wasm32-unknown-unknown`은 통과한 상태다. **데스크톱에서 실행해 렌더 타깃이 상하 반전 없이 나오는지 먼저 확인**하고, 결과를 KB `reference/macroquad/pixel-perfect-rendering.md`의 "확인 못 함" 항목에 반영한다.

이번 세션의 목표: `docs/phase-1-combat-prototype.md`의 1단계를 진행한다. 순서는
(a) 모듈 분리(`input`, `player`, `enemy`, `combat`, `room`, `render`, `tuning`)와 고정 타임스텝,
(b) 발톱 3연타 + 굴파기(명세는 문서의 초안, 손맛으로 조정),
(c) 적 2종(돌격형, 원거리형)과 피격·사망·재시작,
(d) 전투감 레이어(히트스톱, 흔들림, 피격 플래시, 넉백, 임시 사운드),
(e) `gamepads` 크레이트로 게임패드 입력(데스크톱·웹, JS 플러그인은 `web/`에 복사).
각 항목이 끝날 때마다 커밋한다. 수치는 `tuning.rs`에 모은다. 범위 밖(에셋, 허브, 저장, 두 번째 캐릭터)은 만들지 않는다.

규칙: 코드 어디에서도 `KeyCode`를 직접 보지 않는다(`Actions`만). 외부 JS는 핫링크하지 않는다. 결정이 바뀌면 `docs/decisions/`에 새 번호로 기록한다. 프로젝트 기록은 게임 레포에, 일반 지식은 KB에.

Netlify 배포는 레포 Secrets(`NETLIFY_AUTH_TOKEN`, `NETLIFY_SITE_ID`)가 등록되면 CI가 자동으로 한다. 시크릿이 없으면 빌드 검증만 한다. 사이트는 Git 연동 없이 만든다.

---

## 세션 시작 전 사람이 할 일
- [x] Netlify 사이트 생성 완료 (2026-10-24, Git 연동 없음): `nail-under-the-roots`, Site ID `a0bc4242-4bdd-4fc8-8266-adeaf143e59c`, https://nail-under-the-roots.netlify.app , 관리 https://app.netlify.com/projects/nail-under-the-roots
- [ ] Netlify User settings → Applications → Personal access token 발급
- [ ] GitHub 레포 Settings → Secrets and variables → Actions → `NETLIFY_AUTH_TOKEN`(위 토큰), `NETLIFY_SITE_ID`(`a0bc4242-4bdd-4fc8-8266-adeaf143e59c`) 등록
- [ ] 등록 후 Actions에서 `deploy-web` 워크플로를 Re-run 하거나 아무 커밋을 push → 사이트가 뜬다
- [ ] (데스크톱 실행 확인용) 로컬에 Rust 툴체인. Linux면 `libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev`
