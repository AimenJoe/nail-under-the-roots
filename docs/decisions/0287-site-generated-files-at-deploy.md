# 0287: 공식 페이지의 생성물(글꼴 서브셋·캐시 해시)은 레포에 두지 않고 배포 때 만든다

- 상태: 채택·구현 (2026-11-08)
- 사장: "이 main을 1.0 브랜치에 머지하고 저 충돌문제를 미연에 방지할 수 있겠지?"

## 전
- `make_images.py`가 `site/`에 바로 `fonts/galmuri11-sub.woff2`(페이지 글자 서브셋)를 쓰고, HTML·CSS 안의 `style.css?v=…`·`site.js?v=…`·`img/*.png?v=…`·글꼴 주소와 `site.js`의 `SPRITE_V`에 내용 해시를 적었다(캐시 깨기, 0155). 그 결과를 커밋했다.
- 페이지는 main에서만 배포되는데(`pages.yml`), 페이지 글은 main 패치와 release 갈래 양쪽에서 고친다. 그러면 글이 서로 다른 곳을 고쳐도 **글꼴 파일(바이너리)·해시 줄은 늘 둘 다 바뀌어** main → release 합치기마다 충돌했다(#16 뒤 합치기에서 woff2·`style.css`·`index.html` 두 쪽).

## 결정
1. **원본만 커밋**: `site/`의 HTML·CSS·JS에는 `?v=`가 없고 `SPRITE_V = ""`, `site/fonts/galmuri11-sub.woff2`는 `.gitignore`.
2. **배포 때 찍기**: `make_images.py --stamp <폴더>` = `site/`를 그 폴더에 복사하고 거기에만 글꼴 서브셋 + 해시. `pages.yml`이 venv에 `fonttools`·`brotli`만 깔아 `--stamp _site` → `_site`를 올린다(Pillow 불필요 — 그림 만들기 `main()`만 PIL을 늦게 부른다). `pages.yml`은 `make_images.py`·Galmuri 원본이 바뀌어도 돈다.
3. 로컬에서 볼 때도 같은 명령으로 찍은 폴더를 띄운다. 원본을 그대로 띄우면 글꼴이 없어 대체 글꼴로 보인다(글 고치기에는 충분).
4. 그림(`img/`)은 그대로 커밋 — 사람이 고르고 바뀔 때만 바뀌는 원본이다.

## 왜
- 해시·서브셋은 원본(HTML·글꼴 원본·그림)에서 늘 똑같이 다시 나오는 값이라 레포에 둘 이유가 없다. 둘 이유가 없는 생성물이 갈래마다 다르게 커밋되면 충돌만 만든다.
- 해시가 하는 일(새 HTML이 옛 CSS와 만나지 않게)은 배포본에만 있으면 된다. 같은 원본이면 같은 해시가 나와 캐시 동작은 전과 같다(찍어 본 `style.css?v=10b0920e`가 합치기 전 main 판과 같음).

## 확인
- `--stamp`로 찍은 배포본: 글꼴·해시 생성, 1280·390·320px × 한국어·영어 넘침 없음, Galmuri 불러옴(헤드리스 Chromium). GitHub API 403(요청 제한)은 적어 둔 값이 남는 기존 동작.
- 확인 못 한 것: Actions에서의 첫 실행(main에 들어간 뒤 pages 실행을 본다).

## 다시 볼 조건
GitHub Pages 대신 다른 호스팅으로 옮기면 그 배포 단계에서 같은 `--stamp`를 부른다.
