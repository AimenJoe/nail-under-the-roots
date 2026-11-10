# 0290. 공식 페이지 SEO — canonical·hreflang x-default·VideoGame 구조화 데이터·사이트맵

- 날짜: 2026-11-10
- 상태: **채택·구현**(사장 2026-11-10: "나일 소개 페이지도 SEO 최적화해줘")
- 기준: Google 검색 센터 문서 — 현지화 버전(hreflang), 사이트맵 만들기, 소프트웨어 앱 구조화 데이터(2026-11-10에 읽음). 회사 페이지(beardworks.io)와 같은 꼴.

## 결정
1. **머리말**(한·영): 자기 주소 `canonical`, 이미 있던 ko·en 짝에 `x-default`(한국어판), `og:url`·`og:site_name`·`og:locale`, 하는 일이 드러나는 제목·설명(웹·Android·PC, 11개 언어, 무료).
2. **링크 미리보기 1280×720** `site/img/og.png` — 타이틀 그림을 점 그대로 2배(`make_images.py` `og_image`). 640×360은 SNS 권장(1200px 이상)보다 작았다.
3. **구조화 데이터(JSON-LD)**: `VideoGame`(이름·다른 이름·장르·플랫폼·언어 11·출시일 2026-11-10·무료 `Offer`·만든 곳 = beardworks.io의 Organization `@id`) + `WebSite`.
   - **`SoftwareApplication`은 넣지 않는다**: Google의 앱 리치 결과는 `aggregateRating`이나 `review`가 필수다. 실제 평점이 없고 지어내지 않는다 — 필수 칸이 빈 채 넣으면 Search Console에 오류로 뜬다. `VideoGame`만이면 리치 결과 대상이 아니라 설명으로만 읽힌다. 상점에 실제 평가가 생기면 다시 본다.
4. **`site/sitemap.xml`**(두 주소 + 언어 짝, `lastmod`) + **`site/robots.txt`**(`Sitemap:` 줄). `privacy.html` 둘은 beardworks.io로 넘기는 페이지라 사이트맵에 넣지 않는다.

## 확인
- JSON-LD `json.loads`·sitemap XML 파싱, 배포본(`--stamp`) 한·영 × 1280·390·320px 넘침 없음.
- 확인 못 한 것: Google 리치 결과 테스트·Search Console(소유 확인이 필요 — 사장).
