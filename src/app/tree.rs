//! 큰 나무(0047 → 0246 → 0247): 재주 칸을 깨우고 끼는 창, 그 칸 그림(일시정지 상태 탭도 같이 쓴다),
//! 그리고 기록 → 이번 런의 재주. 칸마다 그림(덤의 종류), 잠긴 칸을 고르면 거기까지 가는 길·값, 목표 칸(0247).

use macroquad::prelude::*;

use super::{App, Hit, Scene};
use crate::equipment::Character;
use crate::i18n::{self, fill, tr, Tr};
use crate::input::{Actions, Verb};
use crate::talents::BRANCHES;
use crate::ui::{self, Ui, FONT_PX};

impl App {
    /// 기록 → 이번 런에 낄 재주(깨운 갈래만, 지금 캐릭터의 표, 0142).
    pub(super) fn talents(&self) -> crate::talents::Talents {
        use crate::talents::{key_awake, key_pick, Talents, SLOTS};
        let ch = self.world.character;
        let mut k = Talents {
            who: ch,
            ..Talents::default()
        };
        for (i, slot) in SLOTS.iter().enumerate() {
            let side = self.profile.count(&key_pick(ch, *slot)).min(2) as u8;
            if side > 0 && self.profile.flag(&key_awake(ch, *slot, side as usize - 1)) {
                k.pick[i] = side;
            }
        }
        // 갈래 셋 × 깊이 여덟의 덤(0246): 깨운 것은 늘 켜진다.
        for (b, (_, nodes)) in crate::talents::BRANCHES.iter().enumerate() {
            for (d, n) in nodes.iter().enumerate() {
                if let crate::talents::Node::Plus(boon) = n {
                    if self.profile.flag(&crate::talents::key_plus(ch, b, d)) {
                        k.boons[boon.index()] += 1;
                    }
                }
            }
        }
        // 고유 재주(0247): 캐릭터마다 한 칸, 같은 자리.
        k.sig = self.profile.flag(ch.def().signature.key());
        k
    }

    /// 큰 나무 한 칸이 깨었나(고르기 칸은 어느 갈래든, 0246).
    fn node_awake(&self, ch: Character, branch: usize, depth: usize) -> bool {
        use crate::talents::{key_awake, key_plus, Node, BRANCHES};
        match BRANCHES[branch].1[depth] {
            Node::Pick(slot) => (0..2).any(|side| self.profile.flag(&key_awake(ch, slot, side))),
            Node::Plus(_) => self.profile.flag(&key_plus(ch, branch, depth)),
            Node::Sig => self.profile.flag(ch.def().signature.key()),
        }
    }

    /// 깨울 때 세우는 기록 깃발(덤·고유 재주 칸). 고르기 칸은 쪽마다 따로라 없음.
    fn node_key(ch: Character, branch: usize, depth: usize) -> Option<String> {
        use crate::talents::{key_plus, Node, BRANCHES};
        match BRANCHES[branch].1[depth] {
            Node::Pick(_) => None,
            Node::Plus(_) => Some(key_plus(ch, branch, depth)),
            Node::Sig => Some(ch.def().signature.key().to_string()),
        }
    }

    /// 지금 깨울 수 있는 깊이인가: 맨 앞이거나 앞 칸이 깨었다(0246).
    fn node_open(&self, ch: Character, branch: usize, depth: usize) -> bool {
        depth == 0 || self.node_awake(ch, branch, depth - 1)
    }

    /// 갈래 한 줄의 칸들: (깊이, 갈래 A/B). 고르기 칸은 두 칸, 덤은 한 칸.
    fn tree_cells(branch: usize) -> Vec<(usize, usize)> {
        use crate::talents::{Node, BRANCHES};
        let mut v = Vec::new();
        for (d, n) in BRANCHES[branch].1.iter().enumerate() {
            match n {
                Node::Pick(_) => v.extend([(d, 0), (d, 1)]),
                Node::Plus(_) | Node::Sig => v.push((d, 0)),
            }
        }
        v
    }

    /// 그 칸의 값과 이름·설명(0246).
    fn tree_node(
        &self,
        ch: Character,
        branch: usize,
        depth: usize,
        side: usize,
    ) -> (u32, String, Tr, bool) {
        use crate::talents::{key_awake, plus_cost, plus_rank, table, Node, BRANCHES};
        match BRANCHES[branch].1[depth] {
            Node::Pick(slot) => {
                let tal = &table(ch)[slot.index()].1[side];
                (
                    tal.cost,
                    tr(tal.name).to_string(),
                    tal.desc,
                    self.profile.flag(&key_awake(ch, slot, side)),
                )
            }
            Node::Plus(b) => {
                let rank = plus_rank(branch, depth);
                let name = match rank {
                    0 => tr(b.name()).to_string(),
                    n => format!("{} {}", tr(b.name()), ["II", "III", "IV"][(n - 1).min(2)]),
                };
                (
                    plus_cost(depth),
                    name,
                    b.desc(),
                    self.node_awake(ch, branch, depth),
                )
            }
            Node::Sig => {
                let sig = ch.def().signature;
                (
                    plus_cost(depth),
                    tr(sig.name()).to_string(),
                    sig.desc(),
                    self.node_awake(ch, branch, depth),
                )
            }
        }
    }

    /// 큰 나무에 탭이 있나: 고를 수 있는 캐릭터가 둘 이상이면 Naïl/모모/도토(0142·0195).
    fn tree_tabs(&self) -> bool {
        crate::characters::available(&self.profile).count() > 1
    }

    /// 큰 나무 탭의 캐릭터들(고를 수 있는 것, 표 순서).
    fn tree_chars(&self) -> Vec<Character> {
        crate::characters::available(&self.profile)
            .map(|d| d.id)
            .collect()
    }

    /// 맡긴 반딧불로 지금 깨울 수 있는 재주 칸이 있나. 나무 밑동 빛·HUD가 본다(0061).
    pub(super) fn tree_affordable(&self) -> bool {
        use crate::talents::KEY_LIGHT;
        let have = self.profile.count(KEY_LIGHT);
        self.tree_chars().iter().any(|&ch| {
            (0..BRANCHES.len()).any(|b| {
                Self::tree_cells(b).into_iter().any(|(d, side)| {
                    let (cost, _, _, awake) = self.tree_node(ch, b, d, side);
                    !awake && self.node_open(ch, b, d) && cost <= have
                })
            })
        })
    }

    /// 큰 나무의 줄 수(갈래 셋 — 모든 캐릭터 같은 꼴, 0247).
    pub(super) fn tree_rows(&self) -> usize {
        BRANCHES.len()
    }

    /// 그 줄의 칸 수.
    fn tree_width(row: usize) -> usize {
        Self::tree_cells(row).len()
    }

    /// 그 칸(줄·칸 번호)을 깨웠나. 고르기 칸은 그 쪽을.
    fn tree_cell_awake(&self, ch: Character, row: usize, cell: usize) -> bool {
        let cells = Self::tree_cells(row);
        let (d, side) = cells[cell.min(cells.len() - 1)];
        self.tree_node(ch, row, d, side).3
    }

    /// 목표 칸(0247): 기록 `tree_goal.<캐릭터>` = 줄 × 16 + 칸 + 1(0 = 없음). 없는 칸이면 없음.
    pub(super) fn tree_goal(&self, ch: Character) -> Option<(usize, usize)> {
        let v = self.profile.count(&goal_key(ch));
        if v == 0 {
            return None;
        }
        let (r, c) = (((v - 1) / 16) as usize, ((v - 1) % 16) as usize);
        (r < self.tree_rows() && c < Self::tree_width(r)).then_some((r, c))
    }

    fn set_tree_goal(&mut self, ch: Character, goal: Option<(usize, usize)>) {
        let v = goal.map_or(0, |(r, c)| (r * 16 + c + 1) as u32);
        self.profile.set_count(&goal_key(ch), v);
    }

    /// 그 칸까지 아직 안 깬 칸 수와 드는 반딧불(0247). 지나가는 고르기 깊이는 싼 쪽, 그 칸이면 그 쪽.
    pub(super) fn tree_path(&self, ch: Character, row: usize, cell: usize) -> (usize, u32) {
        let cells = Self::tree_cells(row);
        let (d, side) = cells[cell.min(cells.len() - 1)];
        let (mut n, mut cost) = (0, 0);
        for k in 0..=d {
            if k == d {
                let (c, _, _, awake) = self.tree_node(ch, row, d, side);
                if !awake {
                    n += 1;
                    cost += c;
                }
            } else if !self.node_awake(ch, row, k) {
                n += 1;
                cost += cells
                    .iter()
                    .filter(|(cd, _)| *cd == k)
                    .map(|&(_, sd)| self.tree_node(ch, row, k, sd).0)
                    .min()
                    .unwrap_or(0);
            }
        }
        (n, cost)
    }

    /// 칸이 목표(또는 고른 칸)까지 가는 길 위에 있나: 같은 갈래, 그 깊이 이하, 아직 안 깸.
    fn tree_on_path(&self, ch: Character, row: usize, cell: usize, target: (usize, usize)) -> bool {
        let (tr_, tc) = target;
        if row != tr_ {
            return false;
        }
        let cells = Self::tree_cells(row);
        let (d, _) = cells[cell.min(cells.len() - 1)];
        let (td, _) = cells[tc.min(cells.len() - 1)];
        if d > td || self.tree_cell_awake(ch, row, cell) {
            return false;
        }
        // 그 깊이의 고르기 짝 중 하나가 깼으면 지나간 깊이다. 목표 칸 자신은 늘 길.
        cell == tc || (d < td && !self.node_awake(ch, row, d))
    }

    /// 목표 한 줄(일시정지 상태 탭): "목표 이름: 반딧불 가진 것/드는 것". 목표가 없거나 다 깼으면 없음.
    pub(super) fn tree_goal_text(&self, ch: Character) -> Option<String> {
        let (r, c) = self.tree_goal(ch)?;
        let (n, cost) = self.tree_path(ch, r, c);
        if n == 0 {
            return None;
        }
        let (d, side) = Self::tree_cells(r)[c];
        let name = self.tree_node(ch, r, d, side).1;
        let have = self.profile.count(crate::talents::KEY_LIGHT);
        Some(fill(i18n::s().tree_goal_line, &[&name, &have, &cost]))
    }

    /// 목표로 정하거나(다른 칸이면) 푼다(그 칸이면).
    fn toggle_tree_goal(&mut self, ch: Character, here: (usize, usize)) {
        if self.tree_goal(ch) == Some(here) {
            self.set_tree_goal(ch, None);
            self.shop_msg = Some((|s| s.tree_goal_clear, 2.0));
        } else {
            self.set_tree_goal(ch, Some(here));
            self.shop_msg = Some((|s| s.tree_goal_set, 2.5));
        }
        crate::profile::save(&self.profile);
    }

    /// 큰 나무(0047 → 0246 → 0247): 줄 = 갈래 셋(뿌리·가지·잎, 발톱 캐릭터는 독 발톱 줄 하나 더), 좌우 = 깊이(고르기 칸은 두 칸),
    /// 확인 = 깨우기(반딧불, 앞 칸을 깨워야)·고르기 칸은 끼기·빼기, 아직 못 깨우는 칸은 목표로 정하기·풀기.
    /// 닫기: 뒤로·✕·바깥. 캐릭터 탭은 LB·RB(0142·0195).
    pub(super) fn update_tree(&mut self, a: &Actions, tap_hit: Option<Hit>, dt: f32) {
        use crate::talents::{key_awake, key_pick, Node, KEY_LIGHT};
        if let Some((_, t)) = self.shop_msg.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                self.shop_msg = None;
            }
        }
        if self.tree_tabs() {
            let tab = match tap_hit {
                Some(Hit::TreeTab(i)) => Some(i),
                _ if a.tab_prev || a.tab_next => {
                    // 셋 이상(도토, 0195)이면 방향이 있다: LB = 앞, RB = 뒤.
                    let chars = self.tree_chars();
                    let n = chars.len().max(1);
                    let at = chars.iter().position(|c| *c == self.tree_char).unwrap_or(0);
                    Some(if a.tab_prev {
                        (at + n - 1) % n
                    } else {
                        (at + 1) % n
                    })
                }
                _ => None,
            };
            let chars = self.tree_chars();
            let ch = chars[tab.unwrap_or(0).min(chars.len() - 1)];
            if tab.is_some() && ch != self.tree_char {
                self.tree_char = ch;
                self.shop_msg = None;
                self.sfx.play(crate::sfx::Id::UiMove);
            }
        }
        let ch = self.tree_char;
        let ((mut row, mut cell), _) = self.tree_nav(self.tree_cursor, a, true);
        let width = Self::tree_width(row);
        let mut act = a.confirm;
        if let Some(Hit::TreeCell(r, c)) = tap_hit {
            (row, cell) = (r, c);
            act = true;
        }
        cell = cell.min(width - 1);
        if (row, cell) != self.tree_cursor {
            self.shop_msg = None;
        }
        self.tree_cursor = (row, cell);
        let goal = self.tree_goal(ch);
        if act {
            let (depth, side) = Self::tree_cells(row)[cell];
            let (cost, _, _, awake) = self.tree_node(ch, row, depth, side);
            let have = self.profile.count(KEY_LIGHT);
            match BRANCHES[row].1[depth] {
                // 아직 못 깨우는 칸(앞 칸이 잠들었거나 반딧불이 모자란다)은 목표로(0247): 길이 밝게 남는다.
                _ if !awake && (!self.node_open(ch, row, depth) || have < cost) => {
                    self.toggle_tree_goal(ch, (row, cell));
                }
                Node::Pick(slot) => {
                    let pick = key_pick(ch, slot);
                    if !awake {
                        self.profile.set_count(KEY_LIGHT, have - cost);
                        self.profile.set(&key_awake(ch, slot, side));
                        self.profile.set_count(&pick, side as u32 + 1);
                        self.shop_msg = Some((|s| s.tree_awake, 2.5));
                        self.sfx.play(crate::sfx::Id::Heal);
                    } else if self.profile.count(&pick) == side as u32 + 1 {
                        self.profile.set_count(&pick, 0);
                        self.shop_msg = Some((|s| s.tree_unequip, 2.0));
                    } else {
                        self.profile.set_count(&pick, side as u32 + 1);
                        self.shop_msg = Some((|s| s.tree_equip, 2.0));
                        self.sfx.play(crate::sfx::Id::Emerge);
                    }
                }
                Node::Plus(_) | Node::Sig if awake => {
                    self.shop_msg = Some((|s| s.tree_always, 1.5));
                }
                Node::Plus(_) | Node::Sig => {
                    self.profile.set_count(KEY_LIGHT, have - cost);
                    if let Some(k) = Self::node_key(ch, row, depth) {
                        self.profile.set(&k);
                    }
                    self.shop_msg = Some((|s| s.tree_awake, 2.5));
                    self.sfx.play(crate::sfx::Id::Heal);
                }
            }
            // 목표 칸을 깨웠으면 목표는 끝.
            if goal == Some((row, cell)) && self.tree_cell_awake(ch, row, cell) {
                self.set_tree_goal(ch, None);
            }
            crate::profile::save(&self.profile);
            if ch == self.world.character {
                let before = self.world.player.stats.max_hp;
                self.world.player.talents = self.talents();
                self.world.refresh_stats();
                let gained = self.world.player.stats.max_hp - before;
                if gained > 0 {
                    self.world.player.heal(gained);
                }
            }
        }
        if a.cancel || matches!(tap_hit, Some(Hit::Close)) {
            self.shop_msg = None;
            self.scene = Scene::Game;
        }
    }

    /// 큰 나무 칸 사이 옮기기(창·일시정지 재주 탭이 같이 쓴다). 위아래는 같은 깊이를 잇고(칸 번호는 갈래마다 다르다),
    /// 좌우는 줄 안에서. `wrap`이면 줄 끝에서 반대 끝으로, 아니면 그대로 두고 "끝에서 밀었다"(true)를 돌려준다.
    fn tree_nav(&self, cursor: (usize, usize), a: &Actions, wrap: bool) -> ((usize, usize), bool) {
        let rows = self.tree_rows();
        let (mut row, mut cell) = cursor;
        row = row.min(rows - 1);
        cell = cell.min(Self::tree_width(row) - 1);
        let depth_of = |r: usize, c: usize| -> usize {
            let cells = Self::tree_cells(r);
            cells[c.min(cells.len() - 1)].0
        };
        let cell_at_depth = |r: usize, d: usize| -> usize {
            Self::tree_cells(r)
                .iter()
                .position(|(cd, _)| *cd >= d)
                .unwrap_or(0)
        };
        if a.up || a.down {
            let d = depth_of(row, cell);
            row = if a.up {
                (row + rows - 1) % rows
            } else {
                (row + 1) % rows
            };
            cell = cell_at_depth(row, d);
        }
        let width = Self::tree_width(row);
        let mut pushed = false;
        if a.left {
            if cell == 0 && !wrap {
                pushed = true;
            } else {
                cell = (cell + width - 1) % width;
            }
        }
        if a.right {
            if cell + 1 == width && !wrap {
                pushed = true;
            } else {
                cell = (cell + 1) % width;
            }
        }
        ((row, cell), pushed)
    }

    /// 일시정지 재주 탭(0247): 칸을 옮겨 보고, 확인(또는 같은 칸 다시 누르기)으로 아직 못 깨운 칸을 목표로 정하거나 푼다.
    /// 깨우기·끼기는 마을 큰 나무에서만. `lr`: 이번 입력 전부터 이 탭에 있었다(들어온 프레임의 좌우는 칸을 옮기지 않는다).
    /// 줄 끝에서 좌우로 밀었으면 true(옆 탭으로).
    pub(super) fn update_pause_tree(
        &mut self,
        a: &Actions,
        tap_hit: Option<Hit>,
        lr: bool,
    ) -> bool {
        let ch = self.world.character;
        let mut nav = *a;
        if !lr {
            nav.left = false;
            nav.right = false;
        }
        let (mut cur, pushed) = self.tree_nav(self.pause_tree_cursor, &nav, false);
        if pushed {
            return true;
        }
        let mut act = a.confirm;
        if let Some(Hit::TreeCell(r, c)) = tap_hit {
            act = cur == (r, c);
            cur = (r, c);
        }
        if cur != self.pause_tree_cursor {
            self.shop_msg = None;
        }
        self.pause_tree_cursor = cur;
        if act && !self.tree_cell_awake(ch, cur.0, cur.1) {
            self.toggle_tree_goal(ch, cur);
        }
        false
    }

    /// 일시정지 재주 탭 그림: 큰 나무 칸 그대로 + 고른 칸의 이름·설명·상태.
    pub(super) fn draw_pause_tree(&mut self, ui: &Ui, c: Rect) {
        let ch = self.world.character;
        let row_h = 16.0;
        let cursor = self.pause_tree_cursor;
        self.draw_tree_grid(ui, ch, (c.x, c.x + 52.0), row_h, Some(cursor), &|y| {
            Some(c.y + y)
        });
        let dy = c.y + self.tree_rows() as f32 * row_h + 3.0;
        ui.rect(Rect::new(c.x - 4.0, dy - 2.0, c.w + 8.0, 1.0), ui::BORDER);
        self.draw_tree_detail(
            ui,
            ch,
            cursor,
            Rect::new(c.x, dy, c.w, c.bottom() - dy),
            false,
        );
    }

    /// 큰 나무 칸 그림(0247): 큰 나무 창과 일시정지 재주 탭이 같이 쓴다. `xs` = (줄 이름 x, 첫 칸 x),
    /// `y_at(줄 위 y)`가 그릴 화면 y(다 안 보이면 None). `cursor`가 있으면 그 칸을 밝히고 누를 자리를 남긴다.
    /// 칸 = 덤의 그림(고르기는 갈림 그림, 고유 재주는 캐릭터마다). 밝기: 켜짐 > 깨었지만 안 낌 > 지금 깨울 수 있음 > 잠김.
    /// 금빛 테두리 = 고른 칸(아직 안 깬)까지, 없으면 목표 칸까지 깨워야 할 길. 목표 칸은 오른쪽 위 금빛 귀.
    pub(super) fn draw_tree_grid(
        &mut self,
        ui: &Ui,
        ch: Character,
        (label_x, x0): (f32, f32),
        row_h: f32,
        cursor: Option<(usize, usize)>,
        y_at: &dyn Fn(f32) -> Option<f32>,
    ) {
        use crate::talents::{key_pick, Node, KEY_LIGHT};
        let (cell_w, gap) = (14.0, 4.0);
        let have = self.profile.count(KEY_LIGHT);
        let goal = self.tree_goal(ch);
        let target = cursor
            .filter(|&(r, c)| !self.tree_cell_awake(ch, r, c))
            .or(goal);
        let label_w = x0 - label_x - 4.0;
        for (b, (bname, nodes)) in BRANCHES.iter().enumerate() {
            let Some(y) = y_at(b as f32 * row_h) else {
                continue;
            };
            let label = ui.fit(tr(*bname), label_w, FONT_PX);
            ui.text(&label, label_x, y + 2.0, FONT_PX, ui::TEXT_DIM);
            let mut x = x0;
            let mut prev_end: Option<(f32, bool)> = None;
            for (c, (d, side)) in Self::tree_cells(b).into_iter().enumerate() {
                let pick_pair = matches!(nodes[d], Node::Pick(_));
                if side == 1 {
                    // 고르기 두 칸은 붙여 그린다(한 깊이).
                    x -= gap - 1.0;
                }
                let cell = Rect::new(x, y, cell_w, cell_w);
                if cursor.is_some() {
                    self.hits.push((cell, Hit::TreeCell(b, c)));
                }
                let (cost, _, _, awake) = self.tree_node(ch, b, d, side);
                let open = self.node_open(ch, b, d);
                let node_awake = self.node_awake(ch, b, d);
                // 앞 깊이와 잇는 줄(둘 다 깨었으면 밝게).
                if side == 0 {
                    if let Some((px, lit)) = prev_end {
                        let col = if lit && node_awake {
                            ui::ACCENT
                        } else {
                            ui::BORDER
                        };
                        ui.rect(Rect::new(px, y + cell_w / 2.0, x - px, 1.0), col);
                    }
                }
                let on = match nodes[d] {
                    Node::Pick(slot) => {
                        awake && self.profile.count(&key_pick(ch, slot)) == side as u32 + 1
                    }
                    Node::Plus(_) | Node::Sig => awake,
                };
                let look = Look {
                    focus: cursor == Some((b, c)),
                    on,
                    awake,
                    open,
                    path: target.is_some_and(|tg| self.tree_on_path(ch, b, c, tg)),
                    goal: goal == Some((b, c)),
                    ready: open && !awake && cost <= have,
                };
                let pic = match nodes[d] {
                    Node::Pick(_) => Pic::Pick,
                    Node::Plus(boon) => Pic::Boon(boon),
                    Node::Sig => Pic::Sig(ch.def().signature),
                };
                draw_cell(ui, cell, pic, look);
                if pick_pair && side == 1 {
                    // 고르기 짝: 아래에 괄호 줄.
                    ui.rect(
                        Rect::new(
                            cell.x - cell_w - 1.0,
                            cell.bottom() + 1.0,
                            cell_w * 2.0 + 1.0,
                            1.0,
                        ),
                        ui::TEXT_DIM,
                    );
                }
                if side == 0 || pick_pair {
                    prev_end = Some((cell.right(), node_awake));
                }
                x += cell_w + gap;
            }
        }
    }

    /// 고른 칸의 이름 · 값, 설명 두 줄, 상태(깨우지 않은 칸은 거기까지의 길과 값, 0247). 큰 나무 창·일시정지 재주 탭.
    /// `at_tree`: 큰 나무 앞(끼고 빼는 안내), 아니면 일시정지(끼기는 큰 나무에서).
    fn draw_tree_detail(
        &self,
        ui: &Ui,
        ch: Character,
        cursor: (usize, usize),
        r: Rect,
        at_tree: bool,
    ) {
        use crate::talents::{key_pick, Node, KEY_LIGHT};
        let s = i18n::s();
        let have = self.profile.count(KEY_LIGHT);
        let (cur_r, cur_c) = cursor;
        let (path_n, path_cost) = self.tree_path(ch, cur_r, cur_c);
        let path_line = || {
            let mut l = fill(s.tree_path, &[&path_n, &path_cost]);
            if have < path_cost {
                l = format!("{l} · {}", fill(s.tree_short, &[&(path_cost - have)]));
            }
            l
        };
        let mut cost_tag: Option<u32> = None;
        let (name, desc, state) = {
            let cells = Self::tree_cells(cur_r);
            let (d, side) = cells[cur_c.min(cells.len() - 1)];
            let (cost, name, desc, awake) = self.tree_node(ch, cur_r, d, side);
            if !awake {
                cost_tag = Some(cost);
            }
            let state = match (self.shop_msg, BRANCHES[cur_r].1[d]) {
                (Some((m, _)), _) => tr(m).to_string(),
                _ if !awake => path_line(),
                (None, Node::Pick(slot))
                    if self.profile.count(&key_pick(ch, slot)) == side as u32 + 1 =>
                {
                    if at_tree { s.tree_on } else { s.tree_on_far }.to_string()
                }
                (None, Node::Pick(_)) => {
                    if at_tree { s.tree_off } else { s.tree_off_far }.to_string()
                }
                (None, Node::Plus(_) | Node::Sig) => s.tree_always.to_string(),
            };
            (name, tr(desc), state)
        };
        let goal_here = self.tree_goal(ch) == Some(cursor);
        let tag = cost_tag.map(|c| c.to_string()).unwrap_or_default();
        let tag_w = if tag.is_empty() {
            0.0
        } else {
            ui.text_w(&tag, FONT_PX) + 8.0
        };
        let mark_w = if goal_here { 9.0 } else { 0.0 };
        let name = ui.fit(&name, r.w - tag_w - mark_w, FONT_PX);
        if goal_here {
            draw_goal_mark(ui, vec2(r.x, r.y + 2.0));
        }
        ui.text(&name, r.x + mark_w, r.y, FONT_PX, ui::ACCENT);
        if let Some(c) = cost_tag {
            let col = if have >= c { ui::ACCENT } else { ui::TEXT_DIM };
            ui.text_right(&tag, r.right(), r.y, FONT_PX, col);
        }
        // 설명은 칸이 허락하는 만큼(이름 줄·상태 줄을 빼고 2~4줄), 상태 줄은 그 아래 같은 자리.
        let lines = (((r.h - 13.0 - 14.0) / 12.0).floor() as usize).clamp(2, 4);
        for (k, l) in ui::wrap(ui, desc, r.w, FONT_PX)
            .iter()
            .take(lines)
            .enumerate()
        {
            ui.text(l, r.x, r.y + 13.0 + k as f32 * 12.0, FONT_PX, ui::TEXT);
        }
        let state = ui.fit(&state, r.w, FONT_PX);
        let sy = r.y + 15.0 + lines as f32 * 12.0;
        ui.text(&state, r.x, sy, FONT_PX, ui::ACCENT);
    }

    pub(super) fn draw_tree(&mut self, ui: &Ui) {
        use crate::talents::KEY_LIGHT;
        let s = i18n::s();
        let panel = Rect::new(20.0, 8.0, 280.0, 156.0);
        let have = self.profile.count(KEY_LIGHT);
        let light = fill(s.tree_light, &[&have]);
        // 탭 이름이 제목과 반딧불 수 사이에 다 안 들어가면(캐릭터 셋 + 긴 언어, 0195) 탭이 제목 자리를 쓴다 —
        // 나무 앞에서 연 창이라 제목이 없어도 어디인지 안다.
        let chars = self.tree_chars();
        let tabs_w: f32 = chars
            .iter()
            .map(|c| ui.text_w(c.name(), FONT_PX) + 8.0)
            .sum::<f32>()
            + 2.0 * chars.len().saturating_sub(1) as f32;
        let right = panel.right() - 20.0 - ui.text_w(&light, FONT_PX) - 6.0;
        let titled = !self.tree_tabs()
            || panel.x + 6.0 + ui.text_w(s.tree_title, FONT_PX) + 6.0 + tabs_w <= right;
        ui.panel(panel, titled.then_some(s.tree_title));
        self.close_button(ui, panel);
        ui.text_right(
            &light,
            panel.right() - 20.0,
            panel.y + 3.0,
            FONT_PX,
            ui::TEXT,
        );
        let ch = self.tree_char;
        if self.tree_tabs() {
            // 제목과 반딧불 수 사이 제목 줄에 Naïl/모모/도토 탭(0142).
            let x = if titled {
                panel.x + 6.0 + ui.text_w(s.tree_title, FONT_PX) + 6.0
            } else {
                panel.x + 6.0
            };
            let names: Vec<&str> = chars.iter().map(|c| c.name()).collect();
            let active = chars.iter().position(|c| *c == ch).unwrap_or(0);
            for (i, r) in ui
                .tabs(&names, active, x, panel.y + 1.0, right - x)
                .into_iter()
                .enumerate()
            {
                self.hits.push((r, Hit::TreeTab(i)));
            }
        }
        let rows = self.tree_rows();
        let row_h = 18.0;
        let y0 = panel.y + 21.0;
        let cursor = self.tree_cursor;
        self.draw_tree_grid(
            ui,
            ch,
            (panel.x + 6.0, panel.x + 58.0),
            row_h,
            Some(cursor),
            &|y| Some(y0 + y),
        );
        // 고른 칸: 이름 · 값, 설명 두 줄, 상태(깨우지 않은 칸은 거기까지의 길과 값, 0247).
        let dy = y0 + rows as f32 * row_h + 2.0;
        ui.rect(
            Rect::new(panel.x + 4.0, dy - 2.0, panel.w - 8.0, 1.0),
            ui::BORDER,
        );
        self.draw_tree_detail(
            ui,
            ch,
            cursor,
            Rect::new(panel.x + 6.0, dy, panel.w - 12.0, panel.bottom() - dy),
            true,
        );
        let mut hint = super::vfill(s.tree_hint, &[Verb::Confirm, Verb::Back]);
        if self.tree_tabs() {
            // 탭 안내를 앞에. 다 안 들어가는 언어(일본어)는 원래 안내만(탭은 제목 줄에 보이고 누를 수 있다).
            let tabs = super::device_hint(s.tab_hint, s.tab_hint_touch, &[Verb::Tabs]);
            let both = format!("{tabs} · {hint}");
            if ui.text_w(&both, FONT_PX) <= panel.w - 8.0 {
                hint = both;
            }
        }
        ui.hint_strip(&hint, panel, panel.w - 8.0);
    }
}

/// 목표 칸의 기록 키(0247, 캐릭터마다).
fn goal_key(ch: Character) -> String {
    format!("tree_goal.{ch:?}")
}

/// 금빛: 목표 칸·거기까지 깨울 길(0247).
pub(super) const GOLD: Color = Color::new(0.98, 0.84, 0.36, 1.0);

/// 칸 하나의 상태(그리기용).
#[derive(Clone, Copy)]
struct Look {
    focus: bool,
    on: bool,
    awake: bool,
    open: bool,
    path: bool,
    goal: bool,
    /// 지금 가진 반딧불로 깨울 수 있다.
    ready: bool,
}

/// 칸 속 그림(0247): 덤의 종류마다 하나, 고르기 칸은 갈림, 고유 재주는 캐릭터마다(독 방울·가시·갈라진 열매).
#[derive(Clone, Copy)]
enum Pic {
    Boon(crate::talents::Boon),
    Pick,
    Sig(crate::talents::Signature),
}

/// 7×7 점 그림(`#` = 칠함)과 색.
fn pic_pattern(p: Pic) -> ([&'static str; 7], Color) {
    use crate::talents::{Boon, Signature};
    match p {
        Pic::Boon(Boon::Hp) => (
            [
                ".##.##.", "#######", "#######", "#######", ".#####.", "..###..", "...#...",
            ],
            Color::new(0.93, 0.36, 0.45, 1.0),
        ),
        Pic::Boon(Boon::Berry) => (
            [
                "..#.#..", "...#...", ".#####.", "#######", "#######", ".#####.", "..###..",
            ],
            Color::new(0.86, 0.22, 0.30, 1.0),
        ),
        Pic::Boon(Boon::Stun) => (
            [
                "...#...", ".#.#.#.", "..###..", "#######", "..###..", ".#.#.#.", "...#...",
            ],
            Color::new(0.98, 0.90, 0.40, 1.0),
        ),
        Pic::Boon(Boon::Lives) => (
            [
                "...#...", "...#...", "..##...", "..#.#..", ".#...#.", ".#...#.", "#.....#",
            ],
            Color::new(0.80, 0.62, 0.42, 1.0),
        ),
        Pic::Boon(Boon::Knock) => (
            [
                "...#...", "...##..", "#######", "#######", "...##..", "...#...", ".......",
            ],
            Color::new(0.96, 0.62, 0.30, 1.0),
        ),
        Pic::Boon(Boon::Haste) => (
            [
                "#..#...", ".#..#..", "..#..#.", "...#..#", "..#..#.", ".#..#..", "#..#...",
            ],
            Color::new(0.55, 0.80, 0.98, 1.0),
        ),
        Pic::Boon(Boon::Beat) => (
            [
                "..####.", "..#..#.", "..#..#.", "..#..#.", "###.##.", "###.##.", ".......",
            ],
            Color::new(0.78, 0.60, 0.98, 1.0),
        ),
        Pic::Boon(Boon::Charge) => (
            [
                "....##.", "...##..", "..####.", ".####..", "..##...", ".##....", ".#.....",
            ],
            Color::new(0.98, 0.80, 0.30, 1.0),
        ),
        Pic::Boon(Boon::Ki) => (
            [
                "..###..", ".#...#.", "#..#..#", "#.###.#", "#..#..#", ".#...#.", "..###..",
            ],
            Color::new(0.45, 0.90, 0.90, 1.0),
        ),
        Pic::Boon(Boon::Edge) => (
            [
                ".....##", "....###", "...###.", "..###..", ".###...", "##.....", "#......",
            ],
            Color::new(0.90, 0.92, 0.96, 1.0),
        ),
        Pic::Boon(Boon::Speed) => (
            [
                ".......", "####...", ".......", "######.", ".......", "###....", ".......",
            ],
            Color::new(0.58, 0.90, 0.52, 1.0),
        ),
        Pic::Boon(Boon::Dodge) => (
            [
                "...#...", "..###..", ".#.#.#.", "...#...", ".......", ".#####.", "#######",
            ],
            Color::new(0.74, 0.58, 0.40, 1.0),
        ),
        Pic::Boon(Boon::Bargain) => (
            [
                "...#...", "..###..", ".#####.", ".##.##.", ".#####.", "..###..", "...#...",
            ],
            Color::new(0.88, 0.76, 0.50, 1.0),
        ),
        Pic::Boon(Boon::Charm) => (
            [
                ".#...#.", "..#.#..", "...#...", "..###..", ".##.##.", ".#####.", "..###..",
            ],
            Color::new(0.96, 0.62, 0.84, 1.0),
        ),
        Pic::Pick => (
            [
                "#.....#", ".#...#.", "..#.#..", "...#...", "...#...", "...#...", "...#...",
            ],
            ui::ACCENT,
        ),
        Pic::Sig(Signature::Thorns) => (
            [
                "#..#..#", ".#.#.#.", "..###..", "###.###", "..###..", ".#.#.#.", "#..#..#",
            ],
            Color::new(0.62, 0.80, 0.40, 1.0),
        ),
        Pic::Sig(Signature::Split) => (
            [
                "..#.#..", ".##.##.", "###.###", "###.###", "###.###", ".##.##.", "..#.#..",
            ],
            Color::new(0.80, 0.58, 0.34, 1.0),
        ),
        Pic::Sig(Signature::Venom) => (
            [
                "...#...", "...#...", "..###..", ".#####.", ".##.##.", ".#####.", "..###..",
            ],
            Color::new(0.52, 0.86, 0.36, 1.0),
        ),
    }
}

/// 칸 하나: 바탕·테두리·그림·표시.
fn draw_cell(ui: &Ui, cell: Rect, pic: Pic, l: Look) {
    ui.rect(cell, if l.focus { ui::PANEL_LIGHT } else { ui::PANEL });
    let edge = if l.focus || l.on {
        ui::ACCENT
    } else if l.path {
        GOLD
    } else if l.open && !l.awake {
        ui::TEXT
    } else {
        ui::BORDER
    };
    ui.rect_lines(cell, edge);
    let (rows, col) = pic_pattern(pic);
    let alpha = if l.on {
        1.0
    } else if l.awake {
        0.6
    } else if l.open {
        0.45
    } else {
        0.2
    };
    let col = Color::new(col.r, col.g, col.b, alpha);
    let (ox, oy) = (cell.x + 3.5, cell.y + 3.5);
    for (y, line) in rows.iter().enumerate() {
        for (x, ch) in line.bytes().enumerate() {
            if ch == b'#' {
                ui.rect(Rect::new(ox + x as f32, oy + y as f32, 1.0, 1.0), col);
            }
        }
    }
    if l.goal {
        draw_goal_mark(ui, vec2(cell.right() - 5.0, cell.y + 1.0));
    }
    if l.ready {
        // 지금 깨울 수 있다: 오른쪽 아래 작은 빛.
        ui.rect(
            Rect::new(cell.right() - 3.0, cell.bottom() - 3.0, 2.0, 2.0),
            ui::ACCENT,
        );
    }
}

/// 목표 표시(금빛 작은 깃발, 4×5).
fn draw_goal_mark(ui: &Ui, at: Vec2) {
    ui.rect(Rect::new(at.x, at.y, 1.0, 5.0), GOLD);
    ui.rect(Rect::new(at.x + 1.0, at.y, 3.0, 1.0), GOLD);
    ui.rect(Rect::new(at.x + 1.0, at.y + 1.0, 2.0, 1.0), GOLD);
    ui.rect(Rect::new(at.x + 1.0, at.y + 2.0, 1.0, 1.0), GOLD);
}
