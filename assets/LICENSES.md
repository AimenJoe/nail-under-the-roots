# 에셋 라이선스

| 파일 | 출처 | 라이선스 | 비고 |
|---|---|---|---|
| `fonts/Galmuri11-Condensed.ttf` | [quiple/galmuri](https://github.com/quiple/galmuri) `dist/Galmuri11-Condensed.ttf` (2026-10-25) | SIL Open Font License 1.1 (폰트 파일 name 테이블과 레포 README에 명시) | 용량을 줄이려고 pyftsubset으로 글리프를 추려 담았다(ASCII, Latin-1, œ Œ Ÿ(프랑스어, 0084), 키릴·„ “ ‚ ‘ – —(러시아어·독일어, 0085), KS X 1001 한글 음절 2,350자·호환 자모, 화살표·도형 기호 일부). README의 "Legal Stuff"에 따르면 Galmuri11 Bold·Condensed는 모든 글리프가 제작자의 순수 창작이다(Regular 계열은 일부 글리프가 닌텐도 DS 폰트에서 유래). 그래서 Condensed를 골랐다. |

| `fonts/NailCJK-ja.ttf`, `fonts/NailCJK-zh-Hans.ttf` | [TakWolf/ark-pixel-font](https://github.com/TakWolf/ark-pixel-font) develop `816fb3cc6df3`(2026-10-25)의 12px 글자 원본 PNG | SIL Open Font License 1.1 (원문 `fonts/OFL-ArkPixel.txt`, 예약 폰트 이름 없음) | `tools/fonts/build_cjk.py`가 그 언어 표에 쓰인 글자만 뽑아 TTF로 다시 지은 수정본. OFL에 맞춰 이름을 바꿨다(Nail CJK 12 JA/SC). Ark는 제작자 창작(README). 표의 기호가 Ark에 없으면 Galmuri11 Condensed(OFL)에서 빌린다(지금은 없음) |
| `fonts/OFL-ArkPixel.txt` | Ark Pixel 레포 `LICENSE-OFL` | — | Ark Pixel의 OFL 원문. 일본어·간체 폰트와 함께 배포 |
| `art/nail.png` | 직접 그림. 원본 `tools/art/nail.py` | © 2026 Beardworks | Naïl 스프라이트 v2(idle·walk·attack·hurt·dead·흙더미·그림자 + 흰 실루엣) |
| `art/nail_fur1~3.png` | 직접 그림. 원본 `tools/art/nail.py`(털 색만 바꿈) | © 2026 Beardworks | 털빛 잿빛·밤빛·모래빛(저널 보상, docs/decisions/0048) |
| `art/portraits.png` | 직접 그림. 원본 `tools/art/portraits.py`. **Naïl 칸 0~3·18~29, 포포 칸 5~6, 물든 모모 칸 7~8, 들쥐 대장(물든 보리) 칸 9, 보리 칸 10~11, 돌아온 모모 칸 12~14, 누룩 할매 칸 15~16, 엄마 칸 17, 도토 칸 30~32, 수수 칸 33~34, 콩이 칸 36~37**은 나노바나나(Google Gemini 이미지 모델)로 사장이 그린 원본 `tools/art/sources/ai/*.png`를 `tools/art/ai_portraits.py`가 칸으로 만든 것(2026-10-26, docs/decisions/0108·0109·0110·0112·0113·0114·0115·0116·0117·0118·0119·0120·0121). 참고 그림은 직접 그린 초상화만 넣었다(팩 그림 없음, 0064). 모든 생성 그림에 SynthID 워터마크가 들어 있다 | © 2026 Beardworks | 대화창 초상화 88×88, 한 줄 12칸(Naïl 털빛 3 × 표정 4 포함, Naïl 표정 4 + 이웃 그림자 + 라쿤 포포 2 + 물든 모모 2 + 들쥐 대장·보리 3 + 돌아온 모모 3 + 누룩 할매 2 + 엄마 두더지 1 + 도토 3 + 수수 2 + 콩이 3) |
| `art/critters.png` | 직접 그림. 원본 `tools/art/critters.py` | © 2026 Beardworks | 적(물든 들쥐·생쥐·시궁쥐·다람쥐·청설모)·허수아비·라쿤 포포·물든 모모·돌아온 모모·도토·돌아온 도토·수수·콩이·돌아온 콩이 16×16 + 흰 실루엣. v2 손으로 찍은 자세 |
| `art/chief.png` | 직접 그림. 원본 `tools/art/chief.py` | © 2026 Beardworks | 튜토리얼 보스 들쥐 대장(물든 모습 8칸)·돌아온 보리(5칸) 32×32 + 흰 실루엣 |
| `art/illust/*.png`(타이틀 바탕 `title.png`·`title_fog.png` 포함, 0083 — 타이틀은 바탕 그림 없이 코드로만) | 직접 그림(임시). 원본 `tools/art/illust.py` | © 2026 Beardworks(molehill·great_tree·last_bloom·mother_burrow·bori_freed·momo_bloom 바탕은 아래 그림들, CC0·퍼블릭 도메인) | 컷씬 일러스트 9장 640×360(2배 밀도, 일부는 패닝용으로 더 큼). 사람이 그린 그림으로 바꾸면 이 행을 작가·계약으로 고친다(docs/art/cutscene-illustrations.md) |
| `tools/art/sources/robert_mouth_of_cave.jpg` → `art/illust/mother_burrow.png` 흙벽 질감 | Hubert Robert, *The Mouth of a Cave*, 유화, 1784, The Met 17.190.25 [소장품 페이지](https://www.metmuseum.org/art/collection/search/437470) (web-large, 2026-10-26) | CC0(Met Open Access, `isPublicDomain: true`) | 엄마 굴 일러스트(0073). 오른쪽 바위벽만 잘라 좌우 대칭으로 넓히고 **밝기 질감으로만** 쓴다(색은 코드의 굴 색) |
| `tools/art/sources/meadow_at_sunset_1845.jpg` → `art/illust/bori_freed.png` 바탕 | *A Meadow at Sunset*, ca. 1845, National Gallery of Art 2013.196.3 소장. [rawpixel](https://www.rawpixel.com/image/7727995/image-art-public-domain-artworks) 이미지를 Openverse 썸네일(600px)로 받음(2026-10-26). 작가 이름은 rawpixel 제목에 'Paul'까지만 보여 확인 못 함 | CC0 1.0(Openverse 표기, NGA Open Access) | 보리 일러스트 바탕(0073) |
| `tools/art/sources/landscape_background_1846.jpg` → `art/illust/momo_bloom.png` 바탕 | *Landscape background*(1846–1848), 수채. [rawpixel](https://www.rawpixel.com/image/8544138/image-background-aesthetic-cloud) 이미지를 Openverse 썸네일(600px)로 받음(2026-10-26). 작가·소장처 미확인 | CC0 1.0(Openverse 표기) | 모모 일러스트 바탕(0073). 원 소장처를 확인하면 이 줄을 고친다 |
| `tools/art/sources/blum_spring_landscape.jpg` → `art/illust/last_bloom.png` 바탕 | Robert Frederick Blum(1857–1903), *Spring Landscape*, 파스텔(사포), The Met 67.155.6 [소장품 페이지](https://www.metmuseum.org/art/collection/search/10188) (`images.metmuseum.org` web-large, 2026-10-26) | CC0(Met Open Access, API `isPublicDomain: true`) | 마지막 꽃 일러스트 바탕(0073). 황토 종이색을 봄빛으로 옮기고(`spring_tone`) 도트화 |
| `tools/art/sources/friedrich_lone_tree.jpg` → `art/illust/great_tree.png` 바탕 | Caspar David Friedrich, *Der einsame Baum*(외로운 나무), 유화, 1822, Alte Nationalgalerie 소장. 복제 사진 [Wikimedia Commons: Google Art Project](https://commons.wikimedia.org/wiki/File:Caspar_David_Friedrich_-_Der_einsame_Baum_-_Google_Art_Project.jpg) 1280px 썸네일을 960px로 줄임(2026-10-26) | 퍼블릭 도메인(작가 1840년 사망, Commons 표기 Public domain. 2차원 작품의 충실한 복제라 새 저작권 없음) | 큰 나무 일러스트 위쪽 하늘·산·들판(0073). 그림의 오른쪽 부분만 잘라 좌우로 이어 붙이고 도트화. 이음매는 큰 나무 줄기 뒤 |
| `tools/art/sources/korb_danube.jpg` → `art/illust/molehill.png` 바탕 | Erzsébet Korb, *Untitled (Landscape with the Danube)*, 파스텔, ca. 1920–22, The Met 2026.21.19 [소장품 페이지](https://www.metmuseum.org/art/collection/search/927415) (`images.metmuseum.org` web-large, 2026-10-26) | CC0(Met Open Access, API `isPublicDomain: true`) | 흙더미 일러스트 바탕(0073). `illust.py`가 도트화(단순화 → 면 나누기 → 1px 외곽선)하고 그 위에 마을·어린 나무·둔덕·엄마 두더지를 그린다. 출처 표시 의무는 없지만 남긴다 |
| `sfx/*.wav` | [Kenney](https://kenney.nl) — [Interface Sounds](https://kenney.nl/assets/interface-sounds) 1.0(`tick_004`·`select_002`·`back_002`·`error_004`), [Impact Sounds](https://kenney.nl/assets/impact-sounds) 1.0(`impactPunch_medium_000·001·003`·`impactSoft_heavy_000·002·004`·`impactPunch_heavy_002·004`), [RPG Audio](https://kenney.nl/assets/rpg-audio)(`handleCoins2`·`creak3`). 2026-10-26 받음 | CC0 1.0(각 팩의 `License.txt`: "Creative Commons Zero, CC0 … free to use in personal, educational and commercial projects", 크레딧은 의무 아님) | `tools/audio/make_audio.py`가 자르고(꼬리) 봉우리를 맞춰 44.1kHz 모노 WAV로(0081). 어느 게임 소리가 어느 파일인지는 그 스크립트의 `SFX` 표 |
| `music/*.ogg`, `music/*.mp3` | Aureolus_Omicron, [15 Melodic RPG Chiptunes](https://opengameart.org/content/15-melodic-rpg-chiptunes)(OpenGameArt) — 01 title_screen·02 bittersweet_story·03 town·05 pavane·06 dungeon·07 the_shrine_of_mysteries·14 battle_2. 2026-10-26 받음 | CC0 1.0(OGA 페이지 라이선스 칸 "CC0", 설명 "CC0. No credit required.") | `tools/audio/make_audio.py`가 모노로 줄이고 처음·끝 5ms를 줄여 고리 딸깍을 없애고 크기를 맞춰 ogg(네이티브)·mp3(웹)로(0081). 어느 곡이 어디인지는 `MUSIC` 표 |
| `art/tiles.png` | 직접 그림. 원본 `tools/art/tiles.py` | © 2026 Beardworks | Sunnyside가 들어오기 전 대체 타일(풀·꽃·흙·벼랑·그루터기·바위·버섯·뿌리·얼룩) |
| `site/img/s_*.png` | 게임 스크린숏(`tools/site/shots.js`) | 게임 화면 | 공식 페이지용(0069). Sunnyside 그림이 **게임 화면 안에** 들어 있다. 팩 원본 파일이 아니고 칸을 꺼낼 수 없는 합성 화면이라, 게임 소개 용도로 둔다. `site/img/p_*`·`i_*`는 직접 그린 그림을 자르거나 복사한 것(`tools/site/make_images.py`) |
| `site/fonts/galmuri11-sub.woff2` | Galmuri11 Condensed를 페이지 글자만 남겨 줄임 | SIL OFL 1.1(예약 이름 없음, `site/fonts/OFL.md`) | 공식 페이지 제목용 |
| `vendor/sunnyside/` (레포 밖) | [Sunnyside World](https://danieldiggle.itch.io/sunnyside) by danieldiggle, ASSET PACK V2.1 | 아래 "Sunnyside World 라이선스 원문" | 공개 레포라 커밋하지 않는다(`.gitignore`, docs/decisions/0010). CI는 비공개 레포 `AimenJoe/nail-assets`(게임이 쓰는 파일만, 2026-10-25)를 Secrets `ASSETS_REPO`·`ASSETS_TOKEN`으로 받는다. Name your own price |

| `fonts/OFL.md` | Galmuri 레포 `ofl.md` | — | Galmuri 폰트의 OFL 원문. 폰트와 함께 배포해야 한다(예약 폰트 이름 없음) |
| `THIRD_PARTY_LICENSES.txt` | `tools/licenses.py`가 생성 | — | Rust 크레이트·웹 JS·폰트 고지문 모음. 배포물(웹 `dist/`, APK assets)에 함께 들어간다 |

## 상업적 사용 (2026-10-25 점검)
| 대상 | 라이선스 | 상업 사용 | 지킬 것 |
|---|---|---|---|
| 직접 그린 그림(`art/`)·합성 효과음(`src/sfx.rs`)·코드 | © 2026 Beardworks | 가능 | — |
| Kenney 효과음·Aureolus_Omicron 음악(`sfx/`·`music/`) | CC0 1.0 | 가능 | 지킬 것 없음. 크레딧은 고마움으로 남긴다(게임 크레딧 화면이 생기면 넣는다) |
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
