# 새 세션 시작 프롬프트

아래를 그대로 붙여 넣는다. 세션에는 **두 레포**(`AimenJoe/nail-under-the-roots`, `AimenJoe/KB`)를 붙인다.

---

Naïl: Under the Roots 개발을 이어간다. 레포 `AimenJoe/nail-under-the-roots`(게임)와 `AimenJoe/KB`(지식 베이스)를 둘 다 사용한다.

시작 순서:
1. 게임 레포의 `CLAUDE.md`, `docs/concept.md`, `docs/phase-1-combat-prototype.md`(끝의 "구현 상태" 포함), `docs/decisions/*`를 읽고 현재 합의 사항을 파악한다.
2. KB의 `CLAUDE.md`와 `maps/gamedev.md`를 읽고, 작업 중 참조할 노트를 확인한다. KB에 없는 지식은 외부 자료로 교차검증하고, 남길 가치가 있으면 KB 노트로 제안한다.
3. 현재 상태: 1단계 전투 프로토타입 코드가 **전부 들어가 있다**(모듈 분리, 고정 타임스텝, 발톱 3연타, 굴파기, 적 2종, 피격·사망·재시작, 전투감 레이어, 임시 합성음, 게임패드). `cargo fmt --check`, `cargo clippy --target wasm32-unknown-unknown -D warnings`, `cargo build --release --target wasm32-unknown-unknown`은 통과. 데스크톱(Xvfb)·헤드리스 Chromium에서 실행 확인됨. **사람이 아직 플레이하지 않았다.**

이번 세션의 목표: `docs/phase-1-combat-prototype.md`의 **판정 4개**를 통과시키는 손맛 조정이다.
(a) 사람이 플레이한 피드백(판정 1~4 각각 예/아니오와 이유)을 받는다. 피드백이 없으면 먼저 요청하고, 그동안 코드 리뷰·리팩터·KB 보강만 한다.
(b) 피드백에 따라 `src/tuning.rs` 수치부터 바꾼다. 수치로 안 되면 메커닉을 바꾸고 `docs/decisions/`에 새 번호로 기록한다.
(c) 실기 게임패드(데스크톱·웹) 확인 결과를 받아 `input.rs` 매핑·데드존을 조정한다.
(d) 판정 통과 뒤에만 2단계(에셋 팩 고정, Android 스파이크) 계획을 세운다. 범위 밖(에셋, 허브, 저장, 두 번째 캐릭터)은 만들지 않는다.

규칙: 코드 어디에서도 `KeyCode`를 직접 보지 않는다(`Actions`만, `input.rs` 안에서만 번역). 외부 JS는 핫링크하지 않는다(`web/` 복사본, `web/README.md`의 패치 메모 유지). 결정이 바뀌면 `docs/decisions/`에 새 번호로 기록한다. 프로젝트 기록은 게임 레포에, 일반 지식은 KB에. 커밋은 항목마다.

Netlify 배포는 레포 Secrets(`NETLIFY_AUTH_TOKEN`, `NETLIFY_SITE_ID`)가 등록되면 CI가 자동으로 한다. 시크릿이 없으면 빌드 검증만 한다. 사이트는 Git 연동 없이 만든다.

---

## 세션 시작 전 사람이 할 일
- [x] Netlify 사이트 생성 완료 (2026-10-24, Git 연동 없음): `nail-under-the-roots`, Site ID `a0bc4242-4bdd-4fc8-8266-adeaf143e59c`, https://nail-under-the-roots.netlify.app , 관리 https://app.netlify.com/projects/nail-under-the-roots
- [ ] Netlify User settings → Applications → Personal access token 발급
- [ ] GitHub 레포 Settings → Secrets and variables → Actions → `NETLIFY_AUTH_TOKEN`(위 토큰), `NETLIFY_SITE_ID`(`a0bc4242-4bdd-4fc8-8266-adeaf143e59c`) 등록
- [ ] 등록 후 Actions에서 `deploy-web` 워크플로를 Re-run 하거나 아무 커밋을 push → 사이트가 뜬다
- [ ] **플레이해 보기**(웹 또는 `cargo run`). `docs/phase-1-combat-prototype.md` 판정 4개에 예/아니오와 한 줄 이유를 적어 다음 세션에 전달. 가능하면 본인 외 1명 더.
- [ ] 게임패드를 꽂고 데스크톱·웹에서 이동/공격/굴파기/재시작이 되는지, 스틱 데드존이 적당한지 확인
- [ ] (데스크톱 실행용) 로컬 Rust 툴체인. Linux면 `libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev libudev-dev pkg-config`
