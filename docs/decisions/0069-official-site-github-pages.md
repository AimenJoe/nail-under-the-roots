# 0069: 공식 페이지는 GitHub Pages, 게임은 그대로 Netlify

- 상태: 채택 (2026-10-25).
- 사장: "깃헙 페이지로 공식 페이지 만들자."

## 결정
1. **주소**: https://aimenjoe.github.io/nail-under-the-roots/ (영어 `/en/`). 원본은 이 레포의 `site/`다.
   - `pages.yml`이 main의 `site/**`가 바뀔 때 올린다.
   - 쓰는 액션: `configure-pages@v6`·`upload-pages-artifact@v5`·`deploy-pages@v5`. 모두 2주 넘은 판이고 Node 24다(태그 날짜는 각 레포에서 확인).
   - 처음 한 번 사람이 Settings → Pages → Source를 "GitHub Actions"로 둔다. `GITHUB_TOKEN`으로는 켤 수 없다(`configure-pages`의 `enablement`는 다른 토큰이 필요).
2. **게임은 Netlify에 그대로 둔다**(0001). 페이지의 "브라우저에서 하기"가 Netlify로 간다.
   - 옮기지 않은 이유: 게임 배포는 헤더·PR 미리보기까지 이미 돌고 있다. 페이지와 게임은 바뀌는 속도가 다르다.
3. **내용**(한국어·영어):
   - 첫 화면: 오프닝 일러스트(시든 큰 나무)와 오프닝 글. 단추는 브라우저에서 하기, Android APK.
   - 특징 셋: 싸움, 갈래 길, 마을(스크린숏).
   - 이웃 여덟(초상화). 더 있는 것. 해 보기(조작 표). 소식.
   - 스포일러는 뺐다. 엔딩 일러스트, 모모의 끝, 도토의 기억은 넣지 않았다.
4. **그림**:
   - 직접 그린 그림은 `tools/site/make_images.py`로 만든다(초상화 88×88 자르기, 일러스트 복사).
   - 스크린숏은 `tools/site/shots.js`로 찍는다(헤드리스 Chromium, 960×540 = 3배).
   - 모두 `image-rendering: pixelated`로 키운다.
   - 스크린숏에는 Sunnyside 그림이 게임 화면으로 들어 있다. 원본 파일이 아니라 합성 화면이라 소개 용도로 둔다(`assets/LICENSES.md`). 페이지 아래에 팩 이름과 링크를 적는다.
5. **글꼴**: 게임 UI의 Galmuri11 Condensed를 페이지에 나오는 글자만 남겨 woff2로 줄였다(9KB). OFL이고 예약 이름이 없다. 본문은 시스템 글꼴이다.
6. **움직이는 값**: 최신 버전·날짜·APK 직접 주소·소식 목록은 `site.js`가 GitHub Releases API에서 읽는다.
   - 못 읽으면(요청 제한 시간당 60회/IP, 오프라인) 페이지에 적어 둔 값과 릴리스 목록 링크가 남는다.
   - 영어 소식은 릴리스 본문 끝 "English (short)"의 첫 문장을 쓴다.

## 왜
- 공개 레포라 Pages가 무료다. 페이지 원본이 게임 옆에 있으니 릴리스와 같이 고칠 수 있다.
- 릴리스마다 페이지를 손으로 고치지 않게, 버전·소식은 릴리스에서 읽는다(0059의 "버전은 한 곳에서"와 같은 생각).
- 한 장짜리 정적 페이지라 빌드 도구(Jekyll 등)를 두지 않았다. `site/`를 그대로 올린다.

## 확인
- 로컬(`python3 -m http.server`로 `site/`) + 헤드리스 Chromium, 1280×800·390×844(폰)·영어:
  - 가로 넘침 없음(scrollWidth = 화면 폭).
  - 글꼴 불러옴(`document.fonts.check`).
  - 콘솔 오류 없음. 세션에서는 GitHub API가 403(요청 제한)일 때가 있었고, 그때는 적어 둔 값으로 남았다.
- 데스크톱 첫 화면: 일러스트(왼쪽)와 제목·단추(오른쪽)가 한 화면에 들어온다.
- 확인 못 한 것: 실제 Pages 주소. 이 세션에서는 github.io에 접속이 막혀 있어 배포 로그로 본다.
