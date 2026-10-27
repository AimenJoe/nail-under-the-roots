# tools/art/legacy/

나노바나나 원본으로 바뀌기 전의 **코드 그림** 원본이다(0169 R8에서 옮김, 2026-10-27). 지금 게임은 이 파일들을 쓰지 않는다.

- `illust.py`: 컷씬 일러스트 열둘·타이틀 바탕을 코드로 그리던 것(v1·v2·v4, 파스텔·수채 바탕 `pastel_bg`, 0039·0072·0073·0083). 살아 있는 `tools/art/illust.py`에는 타이틀 안개 `title_fog`만 남았다.
- `portraits.py`: 초상화 칸 0~37을 코드로 그리던 것(0015·0071). 살아 있는 `tools/art/portraits.py`에는 칸 4(이웃 그림자)만 남았다.

돌리려면(원본이 없는 장면을 코드 그림으로 만들고 싶을 때): `cd tools/art && python3 legacy/illust.py <출력 폴더> <장면>` — `ai_illust`·`portraits`를 `tools/art`에서 import하므로 거기서 돌린다. 옮긴 뒤 고치지 않는다: 다시 쓰게 되면 살아 있는 파일로 되돌린다.
