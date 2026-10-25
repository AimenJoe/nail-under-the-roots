//! 모험 구조(docs/decisions/0019): 아레나 / 필드 / 던전 사이를 오가고, 카메라를 움직이고, 저장 지점에서 다시 선다.
//!
//! - 필드: 연속 스크롤. 카메라가 Naïl을 따라가고 지도 끝에서 멈춘다. 적은 자다가 가까이 오면 깬다.
//! - 던전: 굴 입구(D)에 닿으면 어두워졌다가 새로 생성한 던전 시작방으로. 카메라는 지금 방 안에만 있고, 문을 지나면
//!   옆 방으로 미끄러진다. 전투방은 안에 들어서면 문이 닫히고 다 잡으면 열린다. 출구(U)로 필드의 입구 앞에 돌아온다.
//! - 쓰러지고 목숨(지렁이)도 없으면: 앱이 확인을 받으면 마지막 저장 지점(필드)에서 체력 가득으로. 반딧불 30%를 잃는다.
//!   던전은 버린다(다음에 새로). 장신구·씨앗·무기·가방은 남는다.
//! - 아레나: 화면 하나짜리 시험장(웨이브·개발 시나리오). 카메라는 움직이지 않는다.

use macroquad::prelude::*;

use crate::biome::Biome;
use crate::dungeon::{self, Dungeon, RoomKind};
use crate::enemy::Enemy;
use crate::field;
use crate::loot::{Loot, LootKind};
use crate::map::Cell;
use crate::room::{PropKind, Room};
use crate::sfx::{self, Sfx};
use crate::tuning as t;
use crate::world::{Gift, World};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Place {
    #[default]
    Arena,
    Field,
    Dungeon,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Transition {
    EnterDungeon,
    ExitDungeon,
    Respawn,
}

/// 어두워졌다(반) → 장소를 바꾸고 → 밝아진다(반). 그동안 로직은 멈춘다.
#[derive(Clone, Copy, Debug)]
pub struct Fade {
    pub kind: Transition,
    pub t: f32,
    swapped: bool,
}

/// 던전에 있는 동안 맡겨 둔 필드.
struct Stash {
    room: Room,
    enemies: Vec<Enemy>,
    loot: Vec<Loot>,
}

#[derive(Default)]
pub struct Adventure {
    pub place: Place,
    pub dungeon: Option<Dungeon>,
    field: Option<Stash>,
    /// 다시 설 곳(필드 좌표). 저장 지점에 닿으면 바뀐다.
    pub save_pos: Vec2,
    /// 필드의 굴 입구(던전에서 나오면 그 앞).
    pub entrance_pos: Vec2,
    /// 다음에 만들 던전의 깊이. 보스를 잡고 나오면 +1.
    pub depth: u32,
    pub current_room: (i32, i32),
    pub fade: Option<Fade>,
    /// 카메라 왼쪽 위(월드 좌표)와 직전 틱 값(보간).
    pub cam: Vec2,
    pub prev_cam: Vec2,
    boss_cleared: bool,
    seed: u64,
}

impl World {
    /// 필드에서 시작하는 모험 월드.
    pub fn field() -> World {
        let p = field::parse();
        let mut w = World::empty();
        let entrance = p.grid.find(Cell::Entrance).first().copied();
        w.room = Room::from_grid(p.grid, p.props, p.spawn_points);
        w.player.pos = p.start;
        w.player.prev_pos = p.start;
        for (kind, pos) in p.enemies {
            let mut e = Enemy::new(kind, pos);
            e.asleep = true;
            w.enemies.push(e);
        }
        w.adv.place = Place::Field;
        w.adv.save_pos = p.start;
        w.adv.entrance_pos = entrance.unwrap_or(p.start);
        w.adv.seed = (macroquad::miniquad::date::now() * 1000.0) as u64;
        w.biome = Biome::Meadow;
        w.snap_cam();
        w
    }

    /// 틱 앞: 장소를 바꾸는 중이면 진행하고 true(이번 틱 로직은 건너뛴다).
    pub fn adventure_pre(&mut self, sfx: &Sfx) -> bool {
        let Some(mut f) = self.adv.fade else {
            return false;
        };
        f.t += t::TICK_DT;
        if !f.swapped && f.t >= t::FADE_HALF {
            f.swapped = true;
            match f.kind {
                Transition::EnterDungeon => self.enter_dungeon(),
                Transition::ExitDungeon => self.exit_dungeon(),
                Transition::Respawn => self.respawn_at_save(),
            }
            sfx.play(sfx::Id::Emerge);
        }
        self.adv.fade = if f.t >= t::FADE_HALF * 2.0 {
            None
        } else {
            Some(f)
        };
        self.adv.prev_cam = self.adv.cam;
        true
    }

    /// 틱 뒤: 칸 위의 것(입구·출구·저장), 던전 방(잠금·클리어·보상), 카메라.
    pub fn adventure_post(&mut self, sfx: &Sfx) {
        if self.adv.place == Place::Arena {
            return;
        }
        if self.player.is_alive() && self.adv.fade.is_none() {
            match self.room.cell_at(self.player.pos) {
                Some(Cell::Entrance) if self.adv.place == Place::Field => {
                    self.start_fade(Transition::EnterDungeon)
                }
                Some(Cell::Exit) if self.adv.place == Place::Dungeon => {
                    self.start_fade(Transition::ExitDungeon)
                }
                Some(Cell::Save) => self.touch_save(sfx),
                _ => {}
            }
        }
        if self.adv.place == Place::Dungeon {
            self.dungeon_rooms(sfx);
        }
        self.update_cam();
    }

    pub fn start_fade(&mut self, kind: Transition) {
        if self.adv.fade.is_none() {
            self.adv.fade = Some(Fade {
                kind,
                t: 0.0,
                swapped: false,
            });
        }
    }

    /// 화면을 덮는 검정(0..1). 렌더가 월드 위에 그린다.
    pub fn fade_alpha(&self) -> f32 {
        match self.adv.fade {
            Some(f) => {
                let k = f.t / t::FADE_HALF;
                if k < 1.0 {
                    k
                } else {
                    (2.0 - k).max(0.0)
                }
            }
            None => 0.0,
        }
    }

    /// 앱이 부른다: 쓰러진 뒤 확인을 누르면 저장 지점으로(모험일 때).
    pub fn begin_respawn(&mut self) {
        self.start_fade(Transition::Respawn);
    }

    fn touch_save(&mut self, sfx: &Sfx) {
        // 저장 지점 칸 가운데.
        let at = (self.player.pos / 16.0).floor() * 16.0 + vec2(8.0, 8.0);
        if (self.adv.save_pos - at).length() < 1.0 {
            return;
        }
        self.adv.save_pos = at;
        self.player.heal(self.player.stats.max_hp);
        sfx.play(sfx::Id::Heal);
        self.toast = Some((|s| s.toast_save, 2.5));
    }

    /// 개발 시나리오용: 어두워지는 연출 없이 바로 던전으로.
    pub fn enter_dungeon_now(&mut self) {
        self.enter_dungeon();
    }

    fn enter_dungeon(&mut self) {
        self.adv.seed = self.adv.seed.wrapping_add(0x9E37_79B9);
        let (d, room) = dungeon::generate(self.adv.seed, self.adv.depth);
        let field_room = std::mem::replace(&mut self.room, room);
        self.adv.field = Some(Stash {
            room: field_room,
            enemies: std::mem::take(&mut self.enemies),
            loot: std::mem::take(&mut self.loot),
        });
        self.projectiles.clear();
        self.player.pos = d.start_pos();
        self.player.prev_pos = self.player.pos;
        self.adv.current_room = d.start;
        self.adv.dungeon = Some(d);
        self.adv.place = Place::Dungeon;
        self.adv.boss_cleared = false;
        self.biome = if self.adv.depth == 0 {
            Biome::Roots
        } else {
            Biome::Deep
        };
        self.snap_cam();
    }

    fn restore_field(&mut self) {
        if let Some(s) = self.adv.field.take() {
            self.room = s.room;
            self.enemies = s.enemies;
            self.loot = s.loot;
        }
        self.projectiles.clear();
        self.adv.dungeon = None;
        self.adv.place = Place::Field;
        self.biome = Biome::Meadow;
    }

    fn exit_dungeon(&mut self) {
        if self.adv.boss_cleared {
            self.adv.depth += 1;
        }
        self.restore_field();
        self.player.pos = self.adv.entrance_pos + vec2(0.0, 22.0);
        self.player.prev_pos = self.player.pos;
        self.snap_cam();
    }

    fn respawn_at_save(&mut self) {
        if self.adv.place == Place::Dungeon {
            self.restore_field();
        }
        self.player.revive();
        self.player.hp = self.player.stats.max_hp;
        self.player.pos = self.adv.save_pos;
        self.player.prev_pos = self.player.pos;
        self.glowbugs = self.glowbugs * 7 / 10;
        self.toast = Some((|s| s.toast_respawn, 2.5));
        self.snap_cam();
    }

    /// 던전: 방이 바뀌면 들르고, 전투방이면 안에 들어선 순간 잠그고 적을 깨운다. 다 잡으면 열고 보상.
    fn dungeon_rooms(&mut self, sfx: &Sfx) {
        let at = dungeon::room_at(self.player.pos);
        let Some(d) = self.adv.dungeon.as_mut() else {
            return;
        };
        if at != self.adv.current_room && d.room(at).is_some() {
            self.adv.current_room = at;
        }
        let Some(room) = d.room_mut(self.adv.current_room) else {
            return;
        };
        if !room.visited {
            room.visited = true;
            if room.kind == RoomKind::Rest {
                let c = room.rect().center();
                for k in 0..3 {
                    self.loot.push(Loot::place(
                        LootKind::Berry,
                        c + vec2(-20.0 + k as f32 * 20.0, 8.0),
                    ));
                }
            }
        }
        let fights = matches!(room.kind, RoomKind::Combat | RoomKind::Boss);
        if fights
            && !room.cleared
            && !self.room.doors_locked()
            && room.interior().contains(self.player.pos)
        {
            self.room.set_doors_locked(true);
            for (kind, pos) in room.enemies.drain(..) {
                let mut e = Enemy::new(kind, pos);
                e.wake_delay = t::ROOM_WAKE_DELAY;
                self.enemies.push(e);
            }
            sfx.play(sfx::Id::WallHit);
            self.feel.add_trauma(0.25);
        }
        if self.room.doors_locked() && self.enemies.is_empty() {
            self.room.set_doors_locked(false);
            room.cleared = true;
            sfx.play(sfx::Id::Emerge);
            if room.kind == RoomKind::Boss {
                let c = room.rect().center();
                if let Some(g) = self.room.grid.as_mut() {
                    g.set((c.x / 16.0) as usize, (c.y / 16.0) as usize, Cell::Exit);
                }
                self.loot
                    .push(Loot::place(LootKind::LifeSeed, c + vec2(0.0, 24.0)));
                self.adv.boss_cleared = true;
                self.toast = Some((|s| s.toast_boss, 3.0));
            }
        }
        // 보물 상자: 닿으면 연다 → 아직 없는 장신구 하나(다 있으면 반딧불).
        let ppos = self.player.pos;
        let near: Option<usize> = self.room.props.iter().position(|p| {
            matches!(p.kind, PropKind::Chest { opened: false })
                && (p.rect.center() - ppos).length() < 18.0
        });
        if let Some(i) = near {
            self.room.open_chest(i);
            let missing: Vec<usize> = (0..crate::equipment::CHARMS.len())
                .filter(|c| !self.charms_found.contains(c))
                .collect();
            if missing.is_empty() {
                for _ in 0..12 {
                    self.loot.push(Loot::spawn(
                        LootKind::Glowbug,
                        self.room.props[i].rect.center(),
                    ));
                }
            } else {
                let c = missing[rand::gen_range(0, missing.len())];
                self.charms_found.push(c);
                self.gifts.push(Gift::Charm(c));
            }
            sfx.play(sfx::Id::Emerge);
        }
    }

    /// 카메라가 머무는 사각형.
    pub fn cam_bounds(&self) -> Rect {
        match self.adv.place {
            Place::Arena => Rect::new(0.0, 0.0, t::VIEW_W, t::VIEW_H),
            Place::Field => self.room.inner,
            Place::Dungeon => dungeon::room_rect(self.adv.current_room),
        }
    }

    fn cam_target(&self) -> Vec2 {
        let b = self.cam_bounds();
        let view = vec2(t::VIEW_W, t::VIEW_H);
        let want = self.player.pos - view / 2.0;
        let axis = |w: f32, lo: f32, size: f32, v: f32| {
            if size <= w {
                lo + (size - w) / 2.0
            } else {
                v.clamp(lo, lo + size - w)
            }
        };
        vec2(
            axis(view.x, b.x, b.w, want.x),
            axis(view.y, b.y, b.h, want.y),
        )
    }

    fn update_cam(&mut self) {
        self.adv.prev_cam = self.adv.cam;
        let target = self.cam_target();
        let k = 1.0 - (-t::CAM_FOLLOW * t::TICK_DT).exp();
        self.adv.cam += (target - self.adv.cam) * k;
        if (target - self.adv.cam).length() < 0.25 {
            self.adv.cam = target;
        }
    }

    pub fn snap_cam(&mut self) {
        self.adv.cam = self.cam_target();
        self.adv.prev_cam = self.adv.cam;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dungeon_round_trip_and_respawn() {
        let mut w = World::field();
        assert_eq!(w.adv.place, Place::Field);
        let field_enemies = w.enemies.len();
        assert!(field_enemies > 0 && w.enemies.iter().all(|e| e.asleep));
        w.enter_dungeon_now();
        assert_eq!(w.adv.place, Place::Dungeon);
        assert!(w.enemies.is_empty(), "던전 시작방엔 적이 없다");
        assert_eq!(w.room.cell_at(w.player.pos), Some(Cell::Floor));
        // 쓰러져서 저장 지점으로: 필드가 돌아오고, 체력 가득, 반딧불 30% 잃음.
        w.glowbugs = 10;
        w.player.hp = 0;
        w.respawn_at_save();
        assert_eq!(w.adv.place, Place::Field);
        assert!(w.adv.dungeon.is_none());
        assert_eq!(w.enemies.len(), field_enemies, "맡겨 둔 필드 적이 돌아온다");
        assert_eq!(w.player.hp, w.player.stats.max_hp);
        assert_eq!(w.player.pos, w.adv.save_pos);
        assert_eq!(w.glowbugs, 7);
        // 다시 들어갔다 나오면 굴 입구 앞.
        w.enter_dungeon_now();
        w.exit_dungeon();
        assert_eq!(w.adv.place, Place::Field);
        assert!((w.player.pos - w.adv.entrance_pos).length() < 40.0);
        assert_eq!(w.adv.depth, 0, "보스를 안 잡았으면 깊이 그대로");
    }

    #[test]
    fn camera_stays_in_bounds() {
        let mut w = World::field();
        for p in [vec2(0.0, 0.0), vec2(10_000.0, 10_000.0), w.room.center()] {
            w.player.pos = p;
            w.snap_cam();
            let b = w.cam_bounds();
            assert!(w.adv.cam.x >= b.x && w.adv.cam.x + t::VIEW_W <= b.right() + 0.01);
            assert!(w.adv.cam.y >= b.y && w.adv.cam.y + t::VIEW_H <= b.bottom() + 0.01);
        }
    }
}
