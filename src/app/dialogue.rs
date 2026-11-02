//! 대화 검수(0269 R34, 요구 2.8B, 개발판만): 한마디 표(`story::LINES`)·말 걸기 갈림(`story::talk_route`)·빈 자리 말을
//! 기록·캐릭터마다 미리 보고, 고른 것을 실제 대화창으로 틀어 본다.
//!
//! - 기록은 화면이 가진 **복사본**(미리 짠 기록 `story::presets` 또는 지금 슬롯을 복사)이다. 틀면 그 복사본에만 "봄"이 남아
//!   한 번짜리 줄을 차례로 넘겨 볼 수 있다. 진짜 기록은 빌리지 않고, 이 화면에 있는 동안은 개발 판이라 저장소에도 쓰지 않는다.
//! - 개발 화면이라 번역하지 않는다(0013). 줄의 글은 지금 언어로 보인다(설정에서 언어를 바꿔 다시 열면 그 언어).

use super::*;
use crate::cutscene::{Face, Speaker};
use crate::profile::Profile;
use crate::story::{self, Talk, When};
use crate::world::Who;

/// 검수할 것 하나.
#[derive(Clone, Copy)]
enum Query {
    When(When),
    Talk(Who),
    Seat,
}

const QUERIES: [Query; 16] = [
    Query::When(When::Wake),
    Query::When(When::RunStart),
    Query::When(When::Village),
    Query::When(When::TalkBori),
    Query::When(When::TalkMomo),
    Query::When(When::TalkDoto),
    Query::Talk(Who::Popo),
    Query::Talk(Who::Bori),
    Query::Talk(Who::Halmae),
    Query::Talk(Who::Momo),
    Query::Talk(Who::Doto),
    Query::Talk(Who::Susu),
    Query::Talk(Who::Kongi),
    Query::Talk(Who::Clover),
    Query::Talk(Who::Nail),
    Query::Seat,
];

const CHARS: [Character; 3] = [Character::Mole, Character::Momo, Character::Doto];

/// 맨 위 두 줄(기록·캐릭터) 다음이 검수 줄.
const HEAD: usize = 2;

/// 화면 상태.
#[derive(Default)]
pub struct DialogueUi {
    cursor: usize,
    view: crate::ui::ListView,
    /// 미리 짠 기록 번호(마지막 = 지금 슬롯 복사본).
    preset: usize,
    ch: usize,
    /// 화면이 가진 복사본 기록.
    profile: Profile,
    /// 이웃마다 늘 하는 말 차례(복사본).
    turn: [u32; story::WHO_COUNT],
    /// 마을에 있는 것으로 볼까(포포: 꾸미기 / 가게).
    village: bool,
    note: String,
}

impl DialogueUi {
    fn preset_count() -> usize {
        story::presets().len() + 1
    }
    fn preset_name(&self) -> String {
        story::presets()
            .get(self.preset)
            .map(|(n, _)| n.to_string())
            .unwrap_or_else(|| "지금 슬롯(복사본)".to_string())
    }
}

impl App {
    /// 개발 메뉴에서 연다: 미리 짠 첫 기록 + 나일.
    pub(super) fn open_dialogue(&mut self) {
        self.dialogue = DialogueUi {
            village: true,
            ..Default::default()
        };
        self.load_dialogue_preset();
        self.scene = Scene::Dialogue;
    }

    fn load_dialogue_preset(&mut self) {
        let d = &mut self.dialogue;
        d.profile = story::presets()
            .into_iter()
            .nth(d.preset)
            .map(|(_, p)| p)
            .unwrap_or_else(|| self.profile.clone());
        d.turn = [0; story::WHO_COUNT];
        d.note = format!("기록: {}", d.preset_name());
    }

    fn dialogue_ch(&self) -> Character {
        CHARS[self.dialogue.ch % CHARS.len()]
    }

    pub(super) fn update_dialogue(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let n = HEAD + QUERIES.len();
        {
            let d = &mut self.dialogue;
            if a.up {
                d.cursor = (d.cursor + n - 1) % n;
            }
            if a.down {
                d.cursor = (d.cursor + 1) % n;
            }
            if let Some(Hit::DialogueItem(i)) = tap_hit {
                d.cursor = i;
            }
            d.view.scroll(self.scroll_in, &mut d.cursor, n);
        }
        if a.cancel {
            self.scene = Scene::Dev;
            return;
        }
        let dir = a.right as i32 - a.left as i32;
        match self.dialogue.cursor {
            0 if dir != 0 => {
                let k = DialogueUi::preset_count() as i32;
                self.dialogue.preset = (self.dialogue.preset as i32 + dir).rem_euclid(k) as usize;
                self.load_dialogue_preset();
            }
            1 if dir != 0 => {
                let k = CHARS.len() as i32;
                self.dialogue.ch = (self.dialogue.ch as i32 + dir).rem_euclid(k) as usize;
            }
            i if i >= HEAD && dir != 0 => self.dialogue.village = !self.dialogue.village,
            i if i >= HEAD && (a.confirm || tap_hit == Some(Hit::DialogueItem(i))) => {
                self.play_query(QUERIES[i - HEAD]);
            }
            _ => {}
        }
    }

    /// 고른 검수 줄을 틀어 본다: 빈 방 월드 + 지금 캐릭터, 끝나면 이 화면으로.
    fn play_query(&mut self, q: Query) {
        let ch = self.dialogue_ch();
        let line: Option<(Speaker, Tr, Face)> = match q {
            Query::When(w) => {
                story::take(&mut self.dialogue.profile, w, ch).map(|l| (l.who, l.text, Face::Happy))
            }
            Query::Seat => {
                let wins = self.dialogue.profile.count(crate::profile::key::MOMO_WINS);
                let (t, f) = story::seat_line(ch, wins);
                Some((Speaker::Nail, t, f))
            }
            Query::Talk(who) => match self.route(who) {
                Talk::Story(w) => story::take(&mut self.dialogue.profile, w, ch)
                    .map(|l| (l.who, l.text, Face::Happy)),
                Talk::Chatter(t, f) => {
                    self.dialogue.turn[story::who_index(who)] += 1;
                    Some((story::speaker(who), t, f))
                }
                Talk::Gift(g) => Some((story::speaker(who), g.line, Face::Happy)),
                _ => None,
            },
        };
        let Some(line) = line else {
            self.dialogue.note = "틀 대사가 없다(창을 여는 갈래이거나 맞는 줄 없음)".to_string();
            return;
        };
        let mut w = World::empty();
        w.character = ch;
        w.player.kit = ch;
        self.world = w;
        self.cutscene = Some(cutscene::talk(&self.world, &[line]));
        self.after_cutscene = AfterCut::Dialogue;
        self.scene = Scene::Cutscene;
    }

    fn route(&self, who: Who) -> Talk {
        let d = &self.dialogue;
        // 무기를 가졌는지는 복사본 기록의 깃발로(가방은 보지 않는다).
        let owns = |w: usize| {
            crate::equipment::WEAPONS[w]
                .key
                .is_some_and(|k| d.profile.flag(k))
        };
        story::talk_route(
            who,
            &story::TalkCtx {
                profile: &d.profile,
                ch: self.dialogue_ch(),
                village: d.village,
                owns: &owns,
                turn: d.turn[story::who_index(who)],
            },
        )
    }

    pub(super) fn draw_dialogue(&mut self, ui: &Ui) {
        let base = ui.with_base_font();
        let ui = &base;
        let panel = Rect::new(4.0, 2.0, 312.0, 176.0);
        ui.panel(panel, Some("대화 검수"));
        let row_h = FONT_PX + 3.0;
        let list = Rect::new(panel.x + 4.0, panel.y + 18.0, 118.0, panel.h - 36.0);
        let n = HEAD + QUERIES.len();
        let fits = ((list.h / row_h).floor() as usize).max(1);
        let first = self
            .dialogue
            .view
            .window(self.dialogue.cursor, n, fits, row_h);
        let ch = self.dialogue_ch();
        for (row, i) in (first..n).take(fits).enumerate() {
            let r = Rect::new(list.x, list.y + row as f32 * row_h, list.w, row_h - 1.0);
            let on = i == self.dialogue.cursor;
            if on {
                ui.rect(r, Color::new(0.25, 0.18, 0.22, 0.9));
            }
            let label = match i {
                0 => format!("◀ {} ▶", self.dialogue.preset_name()),
                1 => format!("◀ {:?} ▶", ch),
                _ => match QUERIES[i - HEAD] {
                    Query::When(w) => format!("한마디 {w:?}"),
                    Query::Talk(who) => format!("말 걸기 {who:?}"),
                    Query::Seat => "빈 자리".to_string(),
                },
            };
            ui.text(
                &ui.fit(&label, r.w - 4.0, FONT_PX),
                r.x + 2.0,
                ui::mid_y(r),
                FONT_PX,
                if on { ui::ACCENT } else { ui::TEXT },
            );
            self.hits.push((r, Hit::DialogueItem(i)));
        }
        // 오른쪽: 고른 검수 줄의 결과.
        let x = list.right() + 6.0;
        let w = panel.right() - 4.0 - x;
        let mut y = list.y;
        let mut put = |ui: &Ui, s: &str, c: Color| {
            for l in ui::wrap(ui, s, w, FONT_PX) {
                if y < panel.bottom() - 24.0 {
                    ui.text(&l, x, y, FONT_PX, c);
                }
                y += FONT_PX + 1.0;
            }
        };
        let cur = self.dialogue.cursor;
        if cur < HEAD {
            let flags: Vec<&str> = self.dialogue.profile.flags().collect();
            put(ui, &format!("깃발 {}개", flags.len()), ui::ACCENT);
            put(ui, &flags.join(" · "), ui::TEXT_DIM);
            let counts: Vec<String> = self
                .dialogue
                .profile
                .counts()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            put(ui, &counts.join(" · "), ui::TEXT_DIM);
        } else {
            match QUERIES[cur - HEAD] {
                Query::When(wh) => {
                    for c in story::candidates(&self.dialogue.profile, wh, ch) {
                        let mark = if c.chosen {
                            "▶"
                        } else if c.ok {
                            "○"
                        } else {
                            "×"
                        };
                        let why: Vec<String> = c
                            .conds
                            .iter()
                            .filter(|(_, ok)| !ok)
                            .map(|(k, _)| k.describe())
                            .collect();
                        let seen = if c.fresh { "" } else { " (봄)" };
                        let head = format!("{mark} {} [{}]{seen}", c.line.id, c.specificity);
                        put(ui, &head, if c.chosen { ui::ACCENT } else { ui::TEXT });
                        if c.chosen {
                            put(ui, tr(c.line.text), ui::TEXT);
                        } else if !why.is_empty() {
                            put(ui, &format!("  안 맞음: {}", why.join(", ")), ui::TEXT_DIM);
                        }
                    }
                }
                Query::Talk(who) => {
                    let v = if self.dialogue.village {
                        "마을"
                    } else {
                        "쉼터"
                    };
                    put(ui, &format!("{who:?} · {v}(좌우로 바꿈)"), ui::ACCENT);
                    let route = self.route(who);
                    let desc = match route {
                        Talk::Decor => "꾸미기·짐 창을 연다".to_string(),
                        Talk::Shop => "가게를 연다".to_string(),
                        Talk::Story(wh) => match story::pick(&self.dialogue.profile, wh, ch) {
                            Some(l) => format!("한마디 {wh:?} → {}: {}", l.id, tr(l.text)),
                            None => format!("한마디 {wh:?} → 없음"),
                        },
                        Talk::SusuMeet => "수수 첫 만남 장면(새싹 발톱)".to_string(),
                        Talk::Gift(g) => format!(
                            "무기 선물 {}: {}",
                            tr(crate::equipment::WEAPONS[g.weapon].name),
                            tr(g.line)
                        ),
                        Talk::Chatter(t, _) => format!(
                            "늘 하는 말 {}번째: {}",
                            self.dialogue.turn[story::who_index(who)] + 1,
                            tr(t)
                        ),
                        Talk::KnotPick => "실뜨기 고르기 창".to_string(),
                        Talk::Nothing => "아무 일도 없다".to_string(),
                    };
                    put(ui, &desc, ui::TEXT);
                }
                Query::Seat => {
                    let wins = self.dialogue.profile.count(crate::profile::key::MOMO_WINS);
                    let (t, _) = story::seat_line(ch, wins);
                    put(ui, &format!("모모를 이긴 수 {wins}"), ui::ACCENT);
                    put(ui, tr(t), ui::TEXT);
                }
            }
        }
        let foot = if self.dialogue.note.is_empty() {
            "위아래 고르기 · 좌우 바꾸기 · 확인 틀기(복사본에 봄) · 뒤로 개발 메뉴"
        } else {
            &self.dialogue.note
        };
        ui.text(
            &ui.fit(foot, panel.w - 8.0, FONT_PX),
            panel.x + 4.0,
            panel.bottom() - 12.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
    }
}
