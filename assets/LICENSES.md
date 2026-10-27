# 에셋 라이선스

| 파일 | 출처 | 라이선스 | 비고 |
|---|---|---|---|
| `fonts/Galmuri11-Condensed.ttf` | [quiple/galmuri](https://github.com/quiple/galmuri) `dist/Galmuri11-Condensed.ttf` (2026-10-25) | SIL Open Font License 1.1 (폰트 파일 name 테이블과 레포 README에 명시) | 용량을 줄이려고 pyftsubset으로 글리프를 추려 담았다(ASCII, Latin-1, œ Œ Ÿ(프랑스어, 0084), 키릴·„ “ ‚ ‘ – —(러시아어·독일어, 0085), 라틴 확장 A U+0100–017F(폴란드어 ą ć ę ł ń ś ź ż 등, 0144), KS X 1001 한글 음절 2,350자·호환 자모, 화살표·도형 기호 일부). README의 "Legal Stuff"에 따르면 Galmuri11 Bold·Condensed는 모든 글리프가 제작자의 순수 창작이다(Regular 계열은 일부 글리프가 닌텐도 DS 폰트에서 유래). 그래서 Condensed를 골랐다. |

| `fonts/NailCJK-ja.ttf`, `fonts/NailCJK-zh-Hans.ttf`, `fonts/NailCJK-zh-Hant.ttf`(번체, 대만 모양 zh_tw → zh_hant → 기본, 0144) | [TakWolf/ark-pixel-font](https://github.com/TakWolf/ark-pixel-font) develop `816fb3cc6df3`(2026-10-25)의 12px 글자 원본 PNG | SIL Open Font License 1.1 (원문 `fonts/OFL-ArkPixel.txt`, 예약 폰트 이름 없음) | `tools/fonts/build_cjk.py`가 그 언어 표에 쓰인 글자만 뽑아 TTF로 다시 지은 수정본. OFL에 맞춰 이름을 바꿨다(Nail CJK 12 JA/SC/TC). Ark는 제작자 창작(README). 표의 기호가 Ark에 없으면 Galmuri11 Condensed(OFL)에서 빌린다(지금은 없음) |
| `fonts/NailCJK-zh-Hant.ttf`의 한자 일부(Ark에 없는 번체 한자, 0144) | [ACh-K/Cubic-11](https://github.com/ACh-K/Cubic-11) 俐方體11號 `fonts/web/Cubic_11.woff2`(레포 `4c566f7`, 2026-06-27, 폰트 버전 1.451) | SIL Open Font License 1.1 (원문과 원 고지 `fonts/OFL-Cubic11.txt`, 예약 폰트 이름 없음. Cubic 11은 JF Dot M+H 12·M+ BITMAP FONTS(자유 라이선스)에서 이어진 폰트) | `build_cjk.py`가 Ark에 없는 한자만 12px로 찍어(fusion-pixel-font와 같은 방법) Nail CJK 12 TC에 넣는다. 수정본이라 원래 이름을 쓰지 않는다 |
| `fonts/OFL-ArkPixel.txt` | Ark Pixel 레포 `LICENSE-OFL` | — | Ark Pixel의 OFL 원문. 일본어·간체 폰트와 함께 배포 |
| `art/nail.png` | 직접 그림. 원본 `tools/art/nail.py` | © 2026 Beardworks | Naïl 스프라이트 v2(idle·walk·attack·hurt·dead·흙더미·그림자 + 흰 실루엣) |
| `art/nail_fur1~3.png` | 직접 그림. 원본 `tools/art/nail.py`(털 색만 바꿈) | © 2026 Beardworks | 털빛 잿빛·밤빛·모래빛(저널 보상, docs/decisions/0048) |
| `art/portraits.png` | 직접 그림. 원본 `tools/art/portraits.py`. **Naïl 칸 0~3·18~29, 포포 칸 5~6, 물든 모모 칸 7~8, 들쥐 대장(물든 보리) 칸 9, 보리 칸 10~11, 돌아온 모모 칸 12~14, 누룩 할매 칸 15~16, 엄마 칸 17, 도토 칸 30~32, 수수 칸 33~34, 콩이 칸 35~37**은 나노바나나(Google Gemini 이미지 모델)로 사장이 그린 원본 `tools/art/sources/ai/*.png`를 `tools/art/ai_portraits.py`가 칸으로 만든 것(2026-10-26, docs/decisions/0108·0109·0110·0112·0113·0114·0115·0116·0117·0118·0119·0120·0121·0122). 참고 그림은 직접 그린 초상화와, 앞서 만든 나노바나나 시트·초상화만 넣었다(팩 그림 없음, 0064). 보이는 ✦ 로고는 원본에서 지웠다(0133). 모든 생성 그림에 SynthID 워터마크가 들어 있고, 원본에 있던 C2PA 콘텐츠 자격 정보도 그대로 둔다(0138) | © 2026 Beardworks | 대화창 초상화 88×88, 한 줄 12칸(Naïl 털빛 3 × 표정 4 포함, Naïl 표정 4 + 이웃 그림자 + 라쿤 포포 2 + 물든 모모 2 + 들쥐 대장·보리 3 + 돌아온 모모 3 + 누룩 할매 2 + 엄마 두더지 1 + 도토 3 + 수수 2 + 콩이 3) |
| `art/critters.png` | 직접 그림. 원본 `tools/art/critters.py` | © 2026 Beardworks | 적(물든 들쥐·생쥐·시궁쥐·다람쥐·청설모)·허수아비·라쿤 포포·물든 모모·돌아온 모모·도토·돌아온 도토·수수·콩이·돌아온 콩이 16×16 + 흰 실루엣. v2 손으로 찍은 자세 |
| `art/dig.png` | 직접 그림. 원본 `tools/art/dig.py` | © 2026 Beardworks | 파 내려가기 32×32 × 6칸(Naïl 털빛 넷 + 돌아온 모모, 땅 단면 전환 전용, 0156) |
| `art/chief.png` | 직접 그림. 원본 `tools/art/chief.py` | © 2026 Beardworks | 튜토리얼 보스 들쥐 대장(물든 모습 8칸)·돌아온 보리(5칸) 32×32 + 흰 실루엣 |
| `art/illust/*.png`(타이틀 바탕 `title.png`·`title_fog.png` 포함, 0083 — 안개 `title_fog`는 코드로만) | 안개 `title_fog`만 직접 그림(`tools/art/illust.py`, 코드). **`great_tree`·`mother_burrow`·`bori_freed`·`momo_bloom`·`halmae_deep`·`last_bloom`·`sprout`·`molehill`·`doto_memory`·`momo_deep`·`kongi_door`·`title`(타이틀 바탕)**은 나노바나나(Google Gemini 이미지 모델)로 사장이 그린 원본 `tools/art/sources/ai/illust/*.png`를 `tools/art/ai_illust.py`가 게임 격자로 만든 것(2026-10-26, docs/decisions/0125~0132·0134~0137, 참고 그림은 나노바나나 캐릭터 시트·초상화와 (처음 셋 `great_tree`·`mother_burrow`·`bori_freed`는) 구도 그림만, 팩 그림 없음, 보이는 ✦ 로고는 원본에서 지움(0133), SynthID 워터마크 있음, 원본에 있던 C2PA 콘텐츠 자격 정보는 그대로 둠(0138)) | © 2026 Beardworks(아래 바탕 그림들은 코드 그림으로 되돌릴 때만 쓴다, CC0·퍼블릭 도메인) | 컷씬 일러스트 11장 + 타이틀 바탕, 640×360(2배 밀도, 일부는 패닝용으로 더 큼). 사람이 그린 그림으로 바꾸면 이 행을 작가·계약으로 고친다(docs/art/cutscene-illustrations.md) |
| `tools/art/sources/robert_mouth_of_cave.jpg` → `art/illust/mother_burrow.png` 흙벽 질감 | Hubert Robert, *The Mouth of a Cave*, 유화, 1784, The Met 17.190.25 [소장품 페이지](https://www.metmuseum.org/art/collection/search/437470) (web-large, 2026-10-26) | CC0(Met Open Access, `isPublicDomain: true`) | 엄마 굴 일러스트(0073). 오른쪽 바위벽만 잘라 좌우 대칭으로 넓히고 **밝기 질감으로만** 쓴다(색은 코드의 굴 색). 지금은 나노바나나 원본이 그 장면을 대신해 게임 그림에 쓰이지 않는다(코드 그림으로 되돌릴 때만, 0126) |
| `tools/art/sources/meadow_at_sunset_1845.jpg` → `art/illust/bori_freed.png` 바탕 | *A Meadow at Sunset*, ca. 1845, National Gallery of Art 2013.196.3 소장. [rawpixel](https://www.rawpixel.com/image/7727995/image-art-public-domain-artworks) 이미지를 Openverse 썸네일(600px)로 받음(2026-10-26). 작가 이름은 rawpixel 제목에 'Paul'까지만 보여 확인 못 함 | CC0 1.0(Openverse 표기, NGA Open Access) | 보리 일러스트 바탕(0073). 지금은 나노바나나 원본이 그 장면을 대신해 게임 그림에 쓰이지 않는다(0127) |
| `tools/art/sources/landscape_background_1846.jpg` → `art/illust/momo_bloom.png` 바탕 | *Landscape background*(1846–1848), 수채. [rawpixel](https://www.rawpixel.com/image/8544138/image-background-aesthetic-cloud) 이미지를 Openverse 썸네일(600px)로 받음(2026-10-26). 작가·소장처 미확인 | CC0 1.0(Openverse 표기) | 모모 일러스트 바탕(0073). 원 소장처를 확인하면 이 줄을 고친다. 지금은 나노바나나 원본이 그 장면을 대신해 게임 그림에 쓰이지 않는다(0128) |
| `tools/art/sources/blum_spring_landscape.jpg` → `art/illust/last_bloom.png` 바탕 | Robert Frederick Blum(1857–1903), *Spring Landscape*, 파스텔(사포), The Met 67.155.6 [소장품 페이지](https://www.metmuseum.org/art/collection/search/10188) (`images.metmuseum.org` web-large, 2026-10-26) | CC0(Met Open Access, API `isPublicDomain: true`) | 마지막 꽃 일러스트 바탕(0073). 황토 종이색을 봄빛으로 옮기고(`spring_tone`) 도트화. 지금은 나노바나나 원본이 그 장면을 대신해 게임 그림에 쓰이지 않는다(0130) |
| `tools/art/sources/friedrich_lone_tree.jpg` → `art/illust/great_tree.png` 바탕 | Caspar David Friedrich, *Der einsame Baum*(외로운 나무), 유화, 1822, Alte Nationalgalerie 소장. 복제 사진 [Wikimedia Commons: Google Art Project](https://commons.wikimedia.org/wiki/File:Caspar_David_Friedrich_-_Der_einsame_Baum_-_Google_Art_Project.jpg) 1280px 썸네일을 960px로 줄임(2026-10-26) | 퍼블릭 도메인(작가 1840년 사망, Commons 표기 Public domain. 2차원 작품의 충실한 복제라 새 저작권 없음) | 큰 나무 일러스트 위쪽 하늘·산·들판(0073). 그림의 오른쪽 부분만 잘라 좌우로 이어 붙이고 도트화. 이음매는 큰 나무 줄기 뒤. 지금은 나노바나나 원본이 그 장면을 대신해 게임 그림에 쓰이지 않는다(0125) |
| `tools/art/sources/korb_danube.jpg` → `art/illust/molehill.png` 바탕 | Erzsébet Korb, *Untitled (Landscape with the Danube)*, 파스텔, ca. 1920–22, The Met 2026.21.19 [소장품 페이지](https://www.metmuseum.org/art/collection/search/927415) (`images.metmuseum.org` web-large, 2026-10-26) | CC0(Met Open Access, API `isPublicDomain: true`) | 흙더미 일러스트 바탕(0073). `illust.py`가 도트화(단순화 → 면 나누기 → 1px 외곽선)하고 그 위에 마을·어린 나무·둔덕·엄마 두더지를 그린다. 출처 표시 의무는 없지만 남긴다. 지금은 나노바나나 원본이 그 장면을 대신해 게임 그림에 쓰이지 않는다(0132) |
| `sfx/*.wav` | [Kenney](https://kenney.nl) — [Interface Sounds](https://kenney.nl/assets/interface-sounds) 1.0(`tick_004`·`select_002`·`back_002`·`error_004`), [Impact Sounds](https://kenney.nl/assets/impact-sounds) 1.0(`impactPunch_medium_000·001·003`·`impactSoft_heavy_000·002·004`·`impactPunch_heavy_002·004`), [RPG Audio](https://kenney.nl/assets/rpg-audio)(`handleCoins2`·`creak3`). 2026-10-26 받음 | CC0 1.0(각 팩의 `License.txt`: "Creative Commons Zero, CC0 … free to use in personal, educational and commercial projects", 크레딧은 의무 아님) | `tools/audio/make_audio.py`가 자르고(꼬리) 봉우리를 맞춰 44.1kHz 모노 WAV로(0081). 어느 게임 소리가 어느 파일인지는 그 스크립트의 `SFX` 표 |
| `music/*.ogg`, `music/*.mp3` | Aureolus_Omicron, [15 Melodic RPG Chiptunes](https://opengameart.org/content/15-melodic-rpg-chiptunes)(OpenGameArt) — 02 bittersweet_story·06 dungeon·07 the_shrine_of_mysteries·14 battle_2(01 title_screen·03 town·05 pavane은 0154부터 안 씀). 2026-10-26 받음 | CC0 1.0(OGA 페이지 라이선스 칸 "CC0", 설명 "CC0. No credit required.") | `tools/audio/make_audio.py`가 모노로 줄이고 처음·끝 5ms를 줄여 고리 딸깍을 없애고 크기를 맞춰 ogg(네이티브)·mp3(웹)로(0081). 어느 곡이 어디인지는 `MUSIC` 표 |
| `music/title.*`·`village.*`·`camp.*` (피아노·바이올린) | Cleyton Kauffman, [Prologue Theme](https://opengameart.org/content/prologue-theme)(바이올린·하프, zip 안 `Prologue Theme.wav`) → 타이틀 · fluffclipse, [Lovely town](https://opengameart.org/content/lovely-town)(피아노·바이올린) → 마을 · Kistol, [Snowfall](https://opengameart.org/content/snowfall)(피아노 왈츠, `Snowfall (Looped ver.)`) → 쉼터. 모두 OpenGameArt, 2026-10-27 받음 | CC0 1.0(세 페이지 모두 라이선스 칸 "CC0"). 크레딧 의무 없음: Kistol 페이지 "Music by Kistol, but credit is not required.", Prologue Theme 페이지·zip(mp3·ogg·wav뿐)과 Lovely town 페이지에는 크레딧 요구가 없다. 같은 작가의 다른 묶음(Forest Whisper Theme) readme에 고지 문구 "Music by Cleyton Kauffman - https://soundcloud.com/cleytonkauffman"가 있어 그 꼴로 적는다 | `make_audio.py`의 `ACOUSTIC` 표. 스테레오로 두고 칩튠 크기(−19dB RMS)로 올려 넘는 봉우리만 리미터로 누른다(0154). 원래 칩튠 01 title_screen·03 town·05 pavane은 이제 안 쓴다 |
| `art/tiles.png` | 직접 그림. 원본 `tools/art/tiles.py` | © 2026 Beardworks | Sunnyside가 들어오기 전 대체 타일(풀·꽃·흙·벼랑·그루터기·바위·버섯·뿌리·얼룩) |
| `site/img/s_*.png` | 게임 스크린숏(`tools/site/shots.js`) | 게임 화면 | 공식 페이지용(0069). Sunnyside 그림이 **게임 화면 안에** 들어 있다. 팩 원본 파일이 아니고 칸을 꺼낼 수 없는 합성 화면이라, 게임 소개 용도로 둔다. `site/img/p_*`·`i_*`는 게임 초상화·일러스트(나노바나나, 위 행)를 자르거나 복사한 것, `favicon.png`는 `art/nail.png` 첫 칸(`tools/site/make_images.py`) |
| `site/fonts/galmuri11-sub.woff2` | Galmuri11 Condensed를 페이지 글자만 남겨 줄임 | SIL OFL 1.1(예약 이름 없음, `site/fonts/OFL.md`) | 공식 페이지 제목용 |
| `vendor/sunnyside/` (레포 밖) | [Sunnyside World](https://danieldiggle.itch.io/sunnyside) by danieldiggle, ASSET PACK V2.1 | 아래 "Sunnyside World 라이선스 원문" | 공개 레포라 커밋하지 않는다(`.gitignore`, docs/decisions/0010). CI는 비공개 레포 `AimenJoe/nail-assets`(게임이 쓰는 파일만, 2026-10-25)를 Secrets `ASSETS_REPO`·`ASSETS_TOKEN`으로 받는다. Name your own price |

| `fonts/OFL.md` | Galmuri 레포 `ofl.md` | — | Galmuri 폰트의 OFL 원문. 폰트와 함께 배포해야 한다(예약 폰트 이름 없음) |
| `THIRD_PARTY_LICENSES.txt` | `tools/licenses.py`가 생성 | — | Rust 크레이트·웹 JS·폰트 고지문 모음. 배포물(웹 `dist/`, APK assets)에 함께 들어간다 |

## 상업적 사용 (2026-10-25 점검)
| 대상 | 라이선스 | 상업 사용 | 지킬 것 |
|---|---|---|---|
| 직접 그린 그림(`art/`의 스프라이트·타일·`title_fog`)·합성 효과음(`src/sfx.rs`)·코드 | © 2026 Beardworks | 가능 | — |
| 나노바나나 생성 그림(초상화 칸 4 말고 모두·컷씬 일러스트·타이틀 바탕) | Google 생성형 AI 이용 약관(Claude가 원문을 확인하지 못함) | 확인 필요 | SynthID·C2PA를 남긴다(0138). 상업 출시 전에 약관을 확인한다 |
| Kenney 효과음·Aureolus_Omicron·Cleyton Kauffman·fluffclipse·Kistol 음악(`sfx/`·`music/`) | CC0 1.0 | 가능 | 법적으로 지킬 것 없음. 크레딧은 고마움으로 남긴다(`THIRD_PARTY_LICENSES.txt`의 소리·음악 칸, 게임 크레딧 화면이 생기면 거기에도) |
| Galmuri11 Condensed | SIL OFL 1.1 | 가능(게임에 넣어 판매 가능) | 폰트 파일만 따로 팔지 않는다. OFL 원문과 저작권 고지를 함께 배포 |
| Ark Pixel 서브셋(NailCJK) | SIL OFL 1.1 | 가능 | 위와 같음. 수정본이라 원래 이름(Ark Pixel)을 폰트 이름으로 쓰지 않는다 |
| 俐方體11號(Cubic 11, NailCJK TC 안) | SIL OFL 1.1 | 가능 | OFL 원문·원 고지(`fonts/OFL-Cubic11.txt`)를 함께 배포 |
| Rust 크레이트 192개 | MIT·Apache-2.0·BSD·ISC·Zlib·Unlicense·Unicode-3.0(전부 허용형, GPL 등 카피레프트 없음) | 가능 | 저작권 고지와 라이선스 원문을 배포물에 포함(`THIRD_PARTY_LICENSES.txt`). 게임 안 크레딧 화면은 나중에 |
| `web/mq_js_bundle.js`, `web/macroquad-gamepads-0.1.js` | MIT OR Apache-2.0 | 가능 | 위와 같음(고지 파일에 포함) |
| Sunnyside World(레포 밖) | 작가 라이선스 V1(2026-10-25 페이지에서 원문 확인) | 가능 | 재포장·재판매 금지(수정해도), 튜토리얼·교육 외 재배포 금지, AI 학습 금지. 공개 레포에 넣지 않는다. 크레딧은 필수 아님(넣기로 함) |

## 크레딧 점검 (2026-10-27, 원문 다시 읽음)
게임 화면에 **반드시** 적어야 하는 것은 없다. 지켜야 하는 것은 "배포물에 고지 파일을 넣기"다.

| 대상 | 화면 크레딧 | 배포물에 넣을 것 | 근거(원문) |
|---|---|---|---|
| Rust 크레이트·웹 JS(MIT·Apache-2.0·BSD·ISC·Unicode-3.0 등) | 의무 아님 | 저작권 고지 + 라이선스 원문(Apache는 NOTICE도) | MIT "The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software." `THIRD_PARTY_LICENSES.txt`가 웹 `dist/`·APK·iOS 앱(assets 폴더 통째)에 들어간다 |
| 폰트 셋(Galmuri·Ark Pixel·Cubic 11, OFL 1.1) | 의무 아님 | 저작권 고지 + OFL 원문(`fonts/OFL*.{md,txt}`) | OFL 2항 "each copy contains the above copyright notice and this license", 4항 저작권자 이름으로 수정본을 홍보하지 않는다(고마움 표기는 된다) |
| Kenney 효과음(CC0) | 의무 아님(부탁: "Support us by crediting Kenney or www.kenney.nl (this is not mandatory)") | — | 팩 안 `License.txt` |
| Aureolus_Omicron·fluffclipse·Kistol·Cleyton Kauffman 음악(CC0) | 의무 아님 | — | 위 음악 행 |
| Sunnyside World | 의무 아님("Credit is not necessary, but always appreciated.") | — | 아래 원문(2026-10-25). 오늘은 itch.io가 세션 네트워크에서 막혀 다시 보지 못했다 |
| 나노바나나 그림(Google) | 약관에 표기 의무 없음(2차 출처) | SynthID·C2PA를 지우지 않는다(0138) | Google이 생성물의 소유권을 주장하지 않는다는 조항(검색 요약·뉴스로만 확인, 원문 페이지는 세션 네트워크에서 막힘). 상업 출시 전에 원문 확인 |
| Met·rawpixel·Wikimedia 그림(CC0·퍼블릭 도메인) | 의무 아님 | — | 지금은 게임 그림에 쓰이지 않는다(나노바나나로 바뀜) |

- 고마움 크레딧은 `tools/licenses.py`가 `THIRD_PARTY_LICENSES.txt` 머리에 싣는다. 게임 안 크레딧·라이선스 화면은 아직 없다: 웹은 `/THIRD_PARTY_LICENSES.txt`로 열 수 있지만 APK·iOS는 파일이 앱 안에만 있다. 스토어 출시 전에 설정에 "크레딧" 화면을 두는 것이 좋다(관례).

## Sunnyside World 라이선스 원문 (V1, 2026-10-25 https://danieldiggle.itch.io/sunnyside 에서 옮김)
> This asset pack can be used in free and commercial projects. You can modify the assets as you need.
> Credit is not necessary, but always appreciated.
> You can redistribute the assets as part of tutorials or educational courses as long as a link to the Sunnyside World itch.io page is included: https://danieldiggle.itch.io/sunnyside.
> You may not repackage and resell the assets, no matter how much they are modified - this includes as NFT’s.
> You may not use these assets to train AI.

규칙(CLAUDE.md): 라이선스를 확인하지 않은 에셋은 커밋하지 않는다. 새 에셋을 넣을 때 이 표에 행을 추가한다.
