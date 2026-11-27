# 코드로 그린 그림 목록 (2026-11-27 조사)

사장: "성의 없는 그림들 다 찾아내줘"(2026-11-27, 판정 2차 뒤). 그림이 다 있는 운영판에서도 **코드의 네모·타원·줄로 그리는 그림**을
렌더 코드에서 찾은 목록이다. 그림이 없을 때만 나오는 대신 그림(폴백)과 효과·UI는 뺐다.

- 조사 방법: 그리기 함수의 부르는 자리를 읽어 "그림이 있을 때 타는 갈래"만 셌다(빌드·실행으로 본 것이 아니다).
  끝까지 읽은 파일은 `render/props.rs`·`sorted.rs`·`actors.rs`·`deep.rs`·`help.rs`·`art.rs`·`sunny.rs`, 나머지(`fx.rs`·`lights.rs`·
  `hud.rs`·`render/mod.rs`·`app/*`)는 그리기 함수 부분만 읽었다 — 빠진 것이 있을 수 있다.
- 크기는 코드 좌표에서 어림한 값(게임 px, Naïl 키 약 21).
- "프롬프트"는 `docs/art/hub-sprite-prompts.md`의 시트 번호. 빈 칸은 아직 없다 — 사장이 바꾸기로 한 묶음부터 쓴다.

## 마을
| 무엇 | 함수 | 크기 | 프롬프트 |
|---|---|---|---|
| 이웃 등불(이웃 곁, 수첩) | `props::draw_neighbor_lamp` | 6×10 + 빛 | E1 |
| 부탁의 흔적: 샘 그릇 · 갈대 피리 | `sorted::draw_quest_trace` 0·1 | 11×5 · 3×10 | E1 |
| 부탁의 흔적: 큰 깍정이 · 풀 방석 · 버섯 등 | 〃 3·4·5 | 11~13×5~10 | E2 |
| 부탁의 흔적: 솜털 잠자리 · 모모의 꽃밭 | 〃 6·2 | 15×5 · 15×6 | E3 |
| 꾸미기 바람종 | `props::draw_decor` 4 | 12×18 | E3 |
| 꾸미기 깃발 줄 · 실 그네(저널 보상) | 〃 5·8 | 34×18 · 18×22 | E4 |
| 실 고리(할매 곁, 실의 꿈 입구) | `props::draw_dream_loop` | 16×20 | E4 |
| 텃밭 흙 칸 · 갓 심은 씨앗 · 다 익은 산딸기의 빨간 알 | `props::draw_plot_soil` · `draw_crop` | 14×10 · 점 셋 · 2×2 셋 | E5 |
| 물웅덩이(나일로 걸을 때) · 다시 솟는 샘(무무 뒤) | `props::draw_puddle` · `draw_village_spring` | 22×10 · 26×12 | |
| 큰 나무 뿌리 끝 풀 포기 · 빈 자리 위 꽃잎 셋 | `sorted::draw_root_grass` · `props::seat_petals` | 1~2px | |

## 쉼터·길
| 무엇 | 함수 | 크기 |
|---|---|---|
| 저장 고리(버섯 일곱의 균환) | `props::draw_save_ring` | 28×17 |
| 길의 샘(밟으면 회복) | `props::draw_spring` | 44×24 |
| 끝방 출구 구멍(내려가는·올라가는) | `props::draw_exit` | 26×15 |
| 필드의 굴 입구 | `props::draw_grid_art`의 `Cell::Entrance` | 24×14 |
| 잠긴 문을 감는 보라 실 | `props::draw_root_gate` | 칸당 점선 |
| 문가의 작은 야광버섯 | `lights::draw_world_lights` | 5×5 + 빛 |
| 곁방 보상 바닥 표지 넷(씨앗·반딧불·산딸기·상자) | `props::draw_floor_sign` | 23×15 — 0259에서 "바닥에 새긴 것처럼" 정한 그림 |

## 굴: 줍는 것·이야기 물건
| 무엇 | 함수 | 크기 |
|---|---|---|
| 반딧불 · 씨앗 · 뿌리 심장 | `props::draw_loot` · `draw_seed` | 4×5 · 4×7 · 9×8 |
| 산딸기 · 쥬시 지렁이(바닥·퀵 슬롯·마을 창 짐) | `props::draw_berry` · `actors::draw_worm` | 9×10 · 9×5 |
| 바닥 무기 셋(발톱·발목 띠·열매 무더기) · 바닥 장신구 | `props::draw_pickup` · `draw_charm_drop` | 9×9 · 9×12 |
| 시든 꽃잎 · 실의 기억(흰 실타래) | `fx::draw_left_petal` · `draw_memory_thread` | 7×6 · 13×8 |
| 부탁 물건 일곱(조약돌·갈대 피리·꽃씨 주머니·깍정이·풀 다발·버섯갓·솜털) | `fx::quest_item_shape` | 9×7 |

## 굴: 장애물·날아가는 것
| 무엇 | 함수 | 크기 |
|---|---|---|
| 가시뿌리 덤불 · 흰 실 그물 · 무너지는 돌 | `fx::draw_thorn` · `draw_web` · `draw_falling_rocks` | 18×14 · 20×12 · 10×10 |
| 아기들의 고치(토끼풀 싸움) · 묻힌 도토리 | `fx::draw_cocoon` · `draw_buried_at` | 20×16 · 10×6 |
| 도토 무기의 가시 덤불 · 적이 숨은 흙 둔덕 | `actors::draw_thorn_patch` · `draw_dig_mound` | 20×11 · 14×7 |
| 도토리(다람쥐 탄·도토 기본 탄) · 도토가 던지는 것 여섯 | `actors::draw_acorn` · `draw_shot` | 5×6 · 3~9 |
| 두꺼비 혀 · 되받아친 탄 | `actors::draw_tongue` · `draw_projectile` | 굵기 2 줄 · 지름 6 |

## 가장 깊은 곳
| 무엇 | 함수 | 크기 |
|---|---|---|
| 실매듭 셋(멀쩡한 것·끊긴 것) | `deep::draw_knot` | 20×20 |
| 할매의 실(매듭 연결·당기기·둘레 쓸기) | `deep::draw_thread` | 굵기 1 물결 줄 |

## 아이콘(메뉴와 HUD)
| 무엇 | 함수 | 크기 |
|---|---|---|
| 무기 아이콘 세 꼴(무기 열넷이 종류별 한 꼴에 색만 다름) | `pause::draw_weapon_icon`·`draw_claw_icon` | 12×12 |
| 장신구 아이콘 한 꼴(색만 다름) | `pause::draw_charm_icon` | 10×13 |
| 재주 나무 칸 그림(서른여섯쯤, 한 색 격자) | `tree::pic_pattern`·`draw_cell` | 7×7 |
| 저널 보상 아이콘 여섯 | `pause::draw_reward_icon` | 7×7 |
| 길 지도 칸 기호 일곱 | `stage::draw_node_icon` | 지름 13~17 |
| HUD 반딧불 아이콘(하트·새싹은 팩 그림) · 실뿌리 셋 | `render::draw_hud` · `hud::draw_hud_bars` | 5×5 · 6×3 |
| 획득 창의 뿌리 심장·시든 꽃잎·부탁 물건 | `App::draw_found_icon` | 10~18 |
| 상태 탭의 체력 하트(HUD는 팩 하트) | `pause::draw_status` | 4×4 |
| 안내 그림의 쉼터(모닥불·좌판)·큰 나무와 작은 소품 | `help::pic_camp`·`pic_tree` 등 | 16~34 |

## 그 밖
| 무엇 | 함수 | 크기 |
|---|---|---|
| Naïl 필살기의 드릴 원뿔 | `actors::draw_player_fx` | 14×14 세모 |
| 먹는 중 입 앞의 산딸기 | `actors::draw_eating` | 5×5 네모 |

## 조사하다 고친 것 — 그림이 있는데 대신 그림이 나오던 곳
- 마을 창 꾸미기 탭의 돌 우물·해바라기·작은 소나무 아이콘: 마을에는 팩 그림이 놓이는데 창은 따로 적은 목록(0·2·3·7)만 팩으로 보내
  타원·세모가 나왔다 → `decor_uses_pack` 한 곳을 같이 본다(화면으로 확인).
- 수첩 "빈 자리" 그림: 통나무 자리 그림이 있어도 대신 그림을 불렀다 → `draw_seat_hub`(이 줄은 모모가 돌아오기 전에만 보여 화면으로는 못 봤다).
- **남은 것**: 안내의 쉼터 그림(`pic_camp`)은 대신 그림(`draw_campfire`·`draw_stall`)을, 큰 나무 그림(`pic_tree`)은 원 셋을 그린다 —
  진짜 그림은 안내 칸보다 커서 줄이거나 칸을 다시 짜야 한다.

## 코드가 맞는 것(바꿀 대상이 아님)
입자·빛 무리·그림자·공격 예고·고리와 표시(`draw_pickup_mark`·`draw_tree_motes`·`draw_well_fireflies` …)·물결·화면 효과·패널과 막대,
터치 버튼 픽토그램(0172), 땅 단면 전환(`draw_descent`).
