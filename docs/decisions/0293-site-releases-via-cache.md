# 0293. 공식 페이지의 최신 버전·소식은 소식 캐시(api.beardworks.io)부터

- 날짜: 2026-11-18
- 상태: **채택**(사장 2026-11-17: Cloudflare Worker 아이디어 중 "1, 2 만들자" — 2가 이것, 1은 매일 점검)
- 짝: beardworks 레포 `ops/`(Worker `beardworks-ops`, 결정·설명은 그 README)

## 왜
- `site.js`가 방문자 브라우저에서 `api.github.com`을 직접 부른다. 익명 한도는 **IP당 시간 60회**(GitHub 문서)라 학교·회사·통신사 공유 IP에서는 이미 바닥나 있을 수 있다 — 2026-11-18 이 작업을 하던 세션의 출구 IP도 남은 횟수 0(`API rate limit exceeded`)이었다. 그러면 페이지에 적어 둔 옛 버전·옛 APK 주소가 뜬다.
- 출시·홍보 날처럼 방문이 몰릴 때 더 잘 걸린다.

## 정한 것
- `site.js`가 읽는 차례: **`https://api.beardworks.io/releases/nail-under-the-roots`**(Worker가 토큰으로 받아 10분 캐시, GitHub가 실패하면 마지막 값 최대 7일) → GitHub 직접 → 적어 둔 값.
- 응답은 GitHub Releases 목록과 같은 꼴(페이지가 쓰는 칸만: `tag_name`·`name`·`published_at`·`html_url`·`draft`·`prerelease`·`body`·`assets[].name/browser_download_url`) — `site.js`의 나머지는 그대로.
- CSP는 없다(확인함) — 새 출처를 막을 것이 없다.

## 확인
- Playwright로 세 경우(캐시 응답 / 캐시 실패 → GitHub / 둘 다 실패 → 적어 둔 v1.0.0)에서 `[data-latest-version]`·`[data-apk]`가 맞게 채워지는 것을 봤다(2026-11-18).
- 운영 페이지에는 `pages.yml`이 main에서 올리므로 다음 main 합치기 때 나간다. 그 전에 Worker가 배포돼 있어야 의미가 있다(안 돼 있어도 GitHub 직접으로 넘어가니 나빠지지는 않는다).
