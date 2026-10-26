# 일러스트 바탕 그림 후보(도트화 시험, 2026-10-26)

`pxl.py pixelate`로 도트화해 시험한 원본들이다(0070 추가). 이 가운데 여섯 장이 0073에서 실제 일러스트 바탕이 됐다(원본 `tools/art/sources/`, 쓰는 곳·라이선스 원문 확인은 `assets/LICENSES.md`가 정본). 아래 표는 시험 기록이다.
- 쓰기로 하면 원본 파일을 다시 받아 라이선스 원문을 확인한다.
- `assets/LICENSES.md`에 한 줄씩 적는다.
- 사람이 그린 그림으로 바꾸는 절차는 `cutscene-illustrations.md`를 따른다.

| 장면 | 원본 | 작가·연도 | 라이선스 | 주소 | 시험 결과 |
|---|---|---|---|---|---|
| great_tree | The Lonely Tree | Caspar David Friedrich, 1822 | 퍼블릭 도메인(원작), 복제 사진 PDM 1.0 | https://www.flickr.com/photos/7128956@N05/29366215605 | 가장 좋다. 앙상한 나무·먼 산이 도트로 잘 남는다 |
| great_tree(뿌리) | The Enchanted Tree | Richard Doyle, 1868 | CC0(Met Open Access) | https://www.metmuseum.org/art/collection/search/896751 | 작은 요정 무리가 소음이 된다. 뿌리 부분만 잘라 써야 한다 |
| molehill | Green meadow hills, farm background | U.S. Department of Agriculture | CC0 1.0(rawpixel) | https://www.rawpixel.com/image/8731944/photo-image-public-domain-women-ag-nature | 좋다. 사진이라 덤불 잎이 자글자글 — 14색이면 덩어리 |
| bori_freed | Meadow at Sunset(ca. 1845) | National Gallery of Art 소장 | CC0 1.0(rawpixel) | https://www.rawpixel.com/image/7727995/image-art-public-domain-artworks | 아주 좋다. 파스텔 결이 도트 붓질처럼 남는다. 오른쪽 종이 가장자리는 `--margin` |
| last_bloom | The Flowering Orchard | Vincent van Gogh, 1888 | CC0(Met Open Access) | https://www.metmuseum.org/art/collection/search/436527 | 붓 자국이 잘게 쪼개져 소음. 꽃 덩어리가 안 보인다 |
| (그 밖) | Landscape with an old oak | J. M. W. Turner, ca. 1796 | CC0(Met) | https://www.metmuseum.org/art/collection/search/757364 | 미시험 |
| (그 밖) | A River in a Meadow | Théodore Rousseau, ca. 1840 | CC0(Met) | https://www.metmuseum.org/art/collection/search/437518 | 미시험 |

### 파스텔·수채(2026-10-26 추가 시험, `--cel --colors 16`)
| 장면 | 원본 | 작가·연도 | 라이선스 | 주소 | 시험 결과 |
|---|---|---|---|---|---|
| molehill·bori_freed | Untitled (Landscape with the Danube), 파스텔 | Erzsébet Korb, ca. 1920–22 | CC0(Met) | https://www.metmuseum.org/art/collection/search/927415 | **가장 좋다.** 16비트 게임 배경 같다. v4 보리를 얹으니 길 위에 앉아 보인다(바닥 그림자 필요) |
| bori_freed(하늘) | Sunset, 파스텔 | Eugène Delacroix, ca. 1850 | CC0(Met) | https://www.metmuseum.org/art/collection/search/342992 | 하늘 띠가 아름답다. 땅이 어둡고 좁아 인물이 뜬다 |
| last_bloom | Spring Landscape, 파스텔 | Robert Frederick Blum | CC0(Met) | https://www.metmuseum.org/art/collection/search/10188 | 흰 꽃나무 줄. 종이색(황토)이 강해 색 보정 필요 |
| great_tree | A landscape with an old oak, 수채 | J. M. W. Turner, ca. 1796 | CC0(Met) | https://www.metmuseum.org/art/collection/search/757364 | 밝고 옅은 큰 나무. 가지가 잘 남는다 |
| (숲) | Frauenchiemsee, 1876, 수채 | Ernst Morgenstern | CC0 1.0(rawpixel) | https://www.rawpixel.com/image/18952717/frauenchiemsee-august-15-1876-ernst-morgenstern | 회녹색 나무 덩어리. 차분하다 |
| (하늘) | Landscape background (1846–1848) | 미상 | CC0 1.0(rawpixel) | https://www.rawpixel.com/image/8544138/image-background-aesthetic-cloud | 노랑·하늘 파스텔. 보리 색과 잘 어울린다(땅 없음) |

배운 것:
- 붓 자국이 큰 그림(프리드리히·파스텔)이 사진보다 낫다.
- 작은 형태가 많은 그림(요정 무리·반 고흐 붓질)은 소음이 된다.
- 색 수 24~28은 그림 같고, 14는 더 게임 같다. 다만 하늘에 띠가 생긴다.
- **파스텔·수채가 가장 잘 맞는다.** 작가가 이미 넓은 붓질로 단순화했고, 밝고 대비가 낮은 그림이다. 그래서 진한 외곽선의 인물이 앞으로 나온다(값 위계). 종이색이 비쳐 화면 전체 색이 한 가족이 된다.
- **밀도를 맞출 것**: 인물(일러스트 2배 밀도)을 320×180(게임 1px = 2칸) 배경에 얹으면 배경만 굵은 도트라 따로 논다(믹셀). 640×360으로 도트화한 배경이 인물과 같은 칸 크기라 덜 어색하다. 다만 원본이 600px쯤이면 흐리다.
- 인물 발밑에 바닥 그림자를 깔아야 붙어 보인다. 하늘만 있는 그림은 인물이 뜬다.
