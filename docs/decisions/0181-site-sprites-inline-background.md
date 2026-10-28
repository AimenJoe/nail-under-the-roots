# 0181. 공식 페이지의 스프라이트 그림은 변수가 아니라 inline background-image로

- 날짜: 2026-10-28
- 상태: 채택. 사장: "영어 버전 홈페이지에 플레이어블 캐릭터가 제대로 표시되지 않음."

## 왜
- 스프라이트(`.sprite`)는 그림 주소를 CSS 변수(`style="--sp: url(img/sp_nail.png)"`)로 받고 `style.css`의 `background: var(--sp)`가 썼다. 변수 속의 상대 `url()`은 페이지가 아니라 **그 변수를 쓰는 스타일시트(사이트 루트의 style.css) 기준**으로 풀린다(Chromium).
- 한국어 페이지는 `img/…`가 style.css 기준으로도 맞아 보였지만, 영어 페이지(`en/`)의 `../img/…`는 `/nail-under-the-roots/` 위로 올라가 `/img/…`(사이트 밖)를 요청해 404 — 두 캐릭터·걷는 띠·적 그림 12개가 모두 비었다. 루트(`/`)에서 띄우면 `../`가 루트에서 멈춰 문제가 숨는다.

## 결정
- 각 요소의 inline style에 `background-image: url(…)`(문서 기준으로 풀린다)를 바로 쓰고, `.sprite`는 위치·크기·반복만 둔다. 변수에는 칸 수(`--n`)·크기(`--s`) 같은 숫자만.

## 확인
- 헤드리스 Chromium으로 `site/`를 `/nail-under-the-roots/` 아래에 띄워(Pages와 같은 경로) 한국어·영어 페이지의 404를 모음: 고치기 전 `/img/sp_*.png` 10종 404, 고친 뒤 0. 두 캐릭터 칸 스크린숏 확인.
