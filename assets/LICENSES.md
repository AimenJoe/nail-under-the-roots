# 에셋 라이선스

| 파일 | 출처 | 라이선스 | 비고 |
|---|---|---|---|
| `fonts/Galmuri11-Condensed.ttf` | [quiple/galmuri](https://github.com/quiple/galmuri) `dist/Galmuri11-Condensed.ttf` (2026-10-25) | SIL Open Font License 1.1 (폰트 파일 name 테이블과 레포 README에 명시) | 용량을 줄이려고 pyftsubset으로 글리프를 추려 담았다(ASCII, Latin-1, KS X 1001 한글 음절 2,350자·호환 자모, 화살표·도형 기호 일부). README의 "Legal Stuff"에 따르면 Galmuri11 Bold·Condensed는 모든 글리프가 제작자의 순수 창작이다(Regular 계열은 일부 글리프가 닌텐도 DS 폰트에서 유래). 그래서 Condensed를 골랐다. |

| `fonts/NailCJK-ja.ttf`, `fonts/NailCJK-zh-Hans.ttf` | [TakWolf/ark-pixel-font](https://github.com/TakWolf/ark-pixel-font) develop `816fb3cc6df3`(2026-10-25)의 12px 글자 원본 PNG | SIL Open Font License 1.1 (원문 `fonts/OFL-ArkPixel.txt`, 예약 폰트 이름 없음) | `tools/fonts/build_cjk.py`가 그 언어 표에 쓰인 글자만 뽑아 TTF로 다시 지은 수정본. OFL에 맞춰 이름을 바꿨다(Nail CJK 12 JA/SC). Ark는 제작자 창작(README). 표의 기호가 Ark에 없으면 Galmuri11 Condensed(OFL)에서 빌린다(지금은 없음) |
| `fonts/OFL-ArkPixel.txt` | Ark Pixel 레포 `LICENSE-OFL` | — | Ark Pixel의 OFL 원문. 일본어·간체 폰트와 함께 배포 |
| `art/nail.png` | 직접 그림. 원본 `tools/art/nail.py` | 프로젝트 소유 | Naïl 스프라이트 v2(idle·walk·attack·hurt·dead·흙더미·그림자 + 흰 실루엣) |
| `art/portraits.png` | 직접 그림. 원본 `tools/art/portraits.py` | 프로젝트 소유 | 대화창 초상화 88×88(Naïl 표정 4 + 이웃 그림자) |
| `art/tiles.png` | 직접 그림. 원본 `tools/art/tiles.py` | 프로젝트 소유 | Sunnyside가 들어오기 전 대체 타일(풀·꽃·흙·벼랑·그루터기·바위·버섯·뿌리·얼룩) |
| `vendor/sunnyside/` (레포 밖) | [Sunnyside World](https://danieldiggle.itch.io/sunnyside) by danieldiggle, ASSET PACK V2.1 | 아래 "Sunnyside World 라이선스 원문" | 공개 레포라 커밋하지 않는다(`.gitignore`, docs/decisions/0010). Name your own price |

| `fonts/OFL.md` | Galmuri 레포 `ofl.md` | — | Galmuri 폰트의 OFL 원문. 폰트와 함께 배포해야 한다(예약 폰트 이름 없음) |
| `THIRD_PARTY_LICENSES.txt` | `tools/licenses.py`가 생성 | — | Rust 크레이트·웹 JS·폰트 고지문 모음. 배포물(웹 `dist/`, APK assets)에 함께 들어간다 |

## 상업적 사용 (2026-10-25 점검)
| 대상 | 라이선스 | 상업 사용 | 지킬 것 |
|---|---|---|---|
| 직접 그린 그림(`art/`)·합성 효과음(`src/sfx.rs`)·코드 | 프로젝트 소유 | 가능 | — |
| Galmuri11 Condensed | SIL OFL 1.1 | 가능(게임에 넣어 판매 가능) | 폰트 파일만 따로 팔지 않는다. OFL 원문과 저작권 고지를 함께 배포 |
| Ark Pixel 서브셋(NailCJK) | SIL OFL 1.1 | 가능 | 위와 같음. 수정본이라 원래 이름(Ark Pixel)을 폰트 이름으로 쓰지 않는다 |
| Rust 크레이트 192개 | MIT·Apache-2.0·BSD·ISC·Zlib·Unlicense·Unicode-3.0(전부 허용형, GPL 등 카피레프트 없음) | 가능 | 저작권 고지와 라이선스 원문을 배포물에 포함(`THIRD_PARTY_LICENSES.txt`). 게임 안 크레딧 화면은 나중에 |
| `web/mq_js_bundle.js`, `web/macroquad-gamepads-0.1.js` | MIT OR Apache-2.0 | 가능 | 위와 같음(고지 파일에 포함) |
| Sunnyside World(레포 밖) | 작가 라이선스 V1(2026-10-25 페이지에서 원문 확인) | 가능 | 재포장·재판매 금지(수정해도), 튜토리얼·교육 외 재배포 금지, AI 학습 금지. 공개 레포에 넣지 않는다. 크레딧은 필수 아님(넣기로 함) |

## Sunnyside World 라이선스 원문 (V1, 2026-10-25 https://danieldiggle.itch.io/sunnyside 에서 옮김)
> This asset pack can be used in free and commercial projects. You can modify the assets as you need.
> Credit is not necessary, but always appreciated.
> You can redistribute the assets as part of tutorials or educational courses as long as a link to the Sunnyside World itch.io page is included: https://danieldiggle.itch.io/sunnyside.
> You may not repackage and resell the assets, no matter how much they are modified - this includes as NFT’s.
> You may not use these assets to train AI.

규칙(CLAUDE.md): 라이선스를 확인하지 않은 에셋은 커밋하지 않는다. 새 에셋을 넣을 때 이 표에 행을 추가한다.
