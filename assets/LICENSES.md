# 에셋 라이선스

| 파일 | 출처 | 라이선스 | 비고 |
|---|---|---|---|
| `fonts/Galmuri11-Condensed.ttf` | [quiple/galmuri](https://github.com/quiple/galmuri) `dist/Galmuri11-Condensed.ttf` (2026-10-25) | SIL Open Font License 1.1 (폰트 파일 name 테이블과 레포 README에 명시) | 용량을 줄이려고 pyftsubset으로 글리프를 추려 담았다(ASCII, Latin-1, KS X 1001 한글 음절 2,350자·호환 자모, 화살표·도형 기호 일부). README의 "Legal Stuff"에 따르면 Galmuri11 Bold·Condensed는 모든 글리프가 제작자의 순수 창작이다(Regular 계열은 일부 글리프가 닌텐도 DS 폰트에서 유래). 그래서 Condensed를 골랐다. |

| `art/nail.png` | 직접 그림. 원본 `tools/art/nail.py` | 프로젝트 소유 | Naïl 스프라이트 v1(idle·walk·attack·hurt·dead·흙더미·그림자 + 흰 실루엣) |
| `art/tiles.png` | 직접 그림. 원본 `tools/art/tiles.py` | 프로젝트 소유 | Sunnyside가 들어오기 전 대체 타일(풀·꽃·흙·벼랑·그루터기·바위·버섯·뿌리·얼룩) |
| `vendor/sunnyside/` (레포 밖) | [Sunnyside World](https://danieldiggle.itch.io/sunnyside) by danieldiggle | **원문 확인 필요.** 검색 요약: 상업 사용·수정 가능, 크레딧 선택, 재판매·재배포 금지, AI 학습 금지 | 공개 레포라 커밋하지 않는다(`.gitignore`). 구매 후 라이선스 원문을 이 행에 옮겨 적는다(docs/decisions/0010) |

규칙(CLAUDE.md): 라이선스를 확인하지 않은 에셋은 커밋하지 않는다. 새 에셋을 넣을 때 이 표에 행을 추가한다.
