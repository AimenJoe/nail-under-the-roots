//! 월드 상태와 한 틱의 순서. 모듈들을 엮는 유일한 곳.

use macroquad::prelude::*;

use crate::biome::Biome;
use crate::combat::{Feel, Strike};
use crate::enemy::{self, Enemy, Kind, Projectile};
use crate::equipment::Stats;
use crate::i18n::Tr;
use crate::input::Actions;
use crate::items::{Bag, Consumable};
use crate::loot::{self, Loot, LootKind};
use crate::player::{self, Player};
use crate::room::Room;
use crate::sfx::{self, Sfx};
use crate::tuning as t;

#[derive(Debug, Clone, Copy)]
pub struct Debug {
    pub overlay: bool,
    pub hitboxes: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum FxKind {
    Ring { r0: f32, r1: f32, color: Color },
    Spark { color: Color },
}

pub struct Fx {
    pub pos: Vec2,
    pub vel: Vec2,
    pub time: f32,
    pub dur: f32,
    pub kind: FxKind,
}

/// 바닥에 놓인 무기. 닿으면 얻는다.
pub struct Pickup {
    pub pos: Vec2,
    /// equipment::WEAPONS 번호.
    pub weapon: usize,
}

pub struct World {
    pub room: Room,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub fx: Vec<Fx>,
    pub pickups: Vec<Pickup>,
    /// 바닥의 반딧불·산딸기.
    pub loot: Vec<Loot>,
    /// 이번 런에 모은 반딧불(화폐). 상점은 2단계.
    pub glowbugs: u32,
    /// 퀵 아이템(산딸기·쥬시 지렁이).
    pub bag: Bag,
    /// 끼고 있는 장신구(equipment::CHARMS 번호). 바꾸면 `refresh_stats`.
    pub charms: Vec<usize>,
    /// 먹은 생명의 씨앗 수(최대 체력 +1씩).
    pub life_seeds: i32,
    /// 화면 위 한 줄 알림(되살아남, 씨앗)과 남은 시간. 글은 Tr이라 월드는 언어를 모른다.
    pub toast: Option<(Tr, f32)>,
    /// 체력이 깎인 채로 산딸기 없이 지나간 처치 수(불운 보정).
    kills_since_berry: u32,
    /// 이번 틱에 얻은 무기(WEAPONS 번호). 앱이 가방에 넣고 비운다. 획득물·컷씬 둘 다 여기로.
    pub gifts: Vec<usize>,
    pub feel: Feel,
    pub debug: Debug,
    /// 적을 다 잡으면 다음 웨이브를 자동으로 띄운다. 시나리오는 끌 수 있다.
    pub auto_waves: bool,
    /// 지금 환경. `biome_by_wave`면 웨이브가 깊이를 대신해 바뀐다.
    pub biome: Biome,
    pub biome_by_wave: bool,
    pub wave: u32,
    pub kills: u32,
    /// 솟구침으로 잡은 수(저널용).
    pub emerge_kills: u32,
    /// 콤보 마지막 타가 명중한 수(튜토리얼용).
    pub finisher_hits: u32,
    /// 굴파기 횟수(튜토리얼용).
    pub burrows: u32,
    pub time: f32,
    /// 이번 틱의 공격 히트박스(디버그 표시용).
    pub last_strike: Option<Strike>,
    respawn_timer: f32,
    enemy_events: Vec<enemy::Event>,
}

impl World {
    pub fn new() -> Self {
        let room = Room::new();
        let player = Player::new(room.center());
        let mut w = Self {
            room,
            player,
            enemies: Vec::new(),
            projectiles: Vec::new(),
            fx: Vec::new(),
            pickups: Vec::new(),
            loot: Vec::new(),
            glowbugs: 0,
            bag: Bag::default(),
            charms: Vec::new(),
            life_seeds: 0,
            toast: None,
            kills_since_berry: 0,
            gifts: Vec::new(),
            feel: Feel::default(),
            debug: Debug {
                overlay: false,
                hitboxes: false,
            },
            auto_waves: true,
            biome: Biome::Meadow,
            biome_by_wave: true,
            wave: 0,
            kills: 0,
            emerge_kills: 0,
            finisher_hits: 0,
            burrows: 0,
            time: 0.0,
            last_strike: None,
            respawn_timer: t::WAVE_RESPAWN_DELAY,
            enemy_events: Vec::new(),
        };
        w.next_wave();
        w
    }

    /// 적도 웨이브도 없는 방(시나리오·컷씬용).
    pub fn empty() -> Self {
        let mut w = Self::new();
        w.enemies.clear();
        w.wave = 0;
        w.auto_waves = false;
        w.biome = Biome::Meadow;
        w.biome_by_wave = false;
        w
    }

    pub fn spawn_enemy(&mut self, kind: Kind, pos: Vec2) {
        self.enemies.push(Enemy::new(kind, pos));
    }

    pub fn tick(&mut self, a: &Actions, sfx: &Sfx) {
        let dt = t::TICK_DT;
        self.time += dt;
        self.feel.tick(dt);
        if self.feel.hitstop > 0.0 {
            // 히트스톱: 로직 시계만 멈춘다. 입력은 버퍼에 쌓인다.
            self.feel.hitstop -= dt;
            self.player.buffer(a);
            self.freeze_interp();
            return;
        }

        // 1. 플레이어
        let strike = self.player.update(a, &self.room, dt);
        self.last_strike = strike;
        for ev in &self.player.events {
            match ev {
                player::Event::Swing(i) => sfx.play(sfx::Id::Claw(*i)),
                player::Event::Burrow => {
                    self.burrows += 1;
                    sfx.play(sfx::Id::Burrow);
                    self.fx.push(ring(self.player.pos, 4.0, 12.0, 0.2, DIRT));
                }
                player::Event::Emerge => {
                    sfx.play(sfx::Id::Emerge);
                    self.fx.push(ring(
                        self.player.pos,
                        6.0,
                        t::EMERGE_RADIUS + 4.0,
                        0.22,
                        DIRT,
                    ));
                    self.feel.add_trauma(0.15);
                    burst(&mut self.fx, self.player.pos, 10, DIRT, 70.0);
                }
                player::Event::Drill => {
                    sfx.play(sfx::Id::Drill);
                    self.fx
                        .push(ring(self.player.pos, 4.0, 16.0, 0.2, PROJECTILE));
                }
                player::Event::DrillWall => {
                    sfx.play(sfx::Id::DrillWall);
                    let p = self.player.pos + self.player.facing * 6.0;
                    self.fx
                        .push(ring(p, 6.0, t::DRILL_WALL_RADIUS + 6.0, 0.3, WHITE));
                    self.fx.push(ring(p, 4.0, t::DRILL_WALL_RADIUS, 0.25, DIRT));
                    burst(&mut self.fx, p, 18, DIRT, 130.0);
                }
                player::Event::Hurt | player::Event::Die => {}
            }
        }
        // 드릴 중에는 흙이 튄다.
        if let player::State::Drill { dir, .. } = self.player.state {
            if (self.time * 60.0) as i32 % 2 == 0 {
                let back = self.player.pos - dir * 6.0;
                burst(&mut self.fx, back, 1, DIRT, 60.0);
            }
        }
        // 쓰러졌다: 쥬시 지렁이가 있으면 저절로 먹고 일어난다(목숨).
        if self.player.events.contains(&player::Event::Die) && self.bag.take(Consumable::Worm) {
            self.revive(sfx);
        }

        // 1a. 퀵 아이템
        if a.item_cycle {
            self.bag.cycle();
        }
        if a.use_item && self.player.is_alive() {
            self.use_quick_item(sfx);
        }
        if let Some((_, time)) = self.toast.as_mut() {
            *time -= dt;
            if *time <= 0.0 {
                self.toast = None;
            }
        }

        // 1b. 획득물
        let ppos = self.player.pos;
        let mut got = Vec::new();
        self.pickups.retain(|p| {
            if (p.pos - ppos).length() <= t::PICKUP_RADIUS {
                got.push((p.pos, p.weapon));
                false
            } else {
                true
            }
        });
        for (pos, w) in got {
            self.gifts.push(w);
            sfx.play(sfx::Id::Emerge);
            self.fx.push(ring(pos, 4.0, 18.0, 0.3, WHITE));
            burst(&mut self.fx, pos, 14, PROJECTILE, 80.0);
        }

        // 1c. 반딧불·가방 아이템·씨앗. 땅속에서도 줍는다(굴파기로 지나가며 쓸어 담기).
        let alive = self.player.is_alive();
        let mut bag = std::mem::take(&mut self.bag);
        let (mut bugs, mut items, mut seeds) = (0, 0, 0);
        let mut picked_at = Vec::new();
        let room = &self.room;
        self.loot.retain_mut(|l| {
            let can = alive
                && match l.kind {
                    LootKind::Glowbug | LootKind::LifeSeed => true,
                    LootKind::Berry => bag.has_room(Consumable::Berry),
                    LootKind::Worm => bag.has_room(Consumable::Worm),
                };
            if !l.update(dt, ppos, can, room) {
                return true;
            }
            match l.kind {
                LootKind::Glowbug => bugs += 1,
                LootKind::Berry => {
                    bag.add(Consumable::Berry);
                    items += 1;
                }
                LootKind::Worm => {
                    bag.add(Consumable::Worm);
                    items += 1;
                }
                LootKind::LifeSeed => seeds += 1,
            }
            picked_at.push((l.pos, l.kind));
            false
        });
        self.bag = bag;
        self.glowbugs += bugs;
        if bugs > 0 {
            sfx.play(sfx::Id::Glowbug);
        }
        if items > 0 {
            sfx.play(sfx::Id::Claw(0));
        }
        if seeds > 0 {
            self.life_seeds += seeds;
            self.refresh_stats();
            self.player.heal(self.player.stats.max_hp);
            sfx.play(sfx::Id::Heal);
            self.toast = Some((|s| s.toast_seed, 2.5));
        }
        for (pos, kind) in picked_at {
            match kind {
                LootKind::Glowbug => self.fx.push(ring(pos, 1.0, 5.0, 0.15, GLOW)),
                LootKind::Berry | LootKind::Worm => {
                    self.fx.push(ring(pos, 2.0, 10.0, 0.2, WHITE));
                }
                LootKind::LifeSeed => {
                    self.fx.push(ring(pos, 4.0, 22.0, 0.4, POISON));
                    burst(&mut self.fx, pos, 16, POISON, 90.0);
                }
            }
        }

        // 2. 적·투사체
        let targetable = self.player.is_alive() && !self.player.is_burrowed();
        self.enemy_events.clear();
        for e in &mut self.enemies {
            e.update(
                self.player.pos,
                targetable,
                &self.room,
                dt,
                &mut self.projectiles,
                &mut self.enemy_events,
            );
        }
        // 쓰러진 적(독·공격). 떨어뜨릴 것은 루프가 끝난 뒤에.
        let mut deaths: Vec<(Vec2, Kind)> = Vec::new();
        for e in self.enemies.iter_mut().filter(|e| e.died_of_poison) {
            e.died_of_poison = false;
            self.kills += 1;
            deaths.push((e.pos, e.kind));
            sfx.play(sfx::Id::EnemyDie);
            let color = enemy_color(e.kind);
            self.fx
                .push(ring(e.pos, e.radius(), e.radius() * 3.0, 0.25, POISON));
            burst(&mut self.fx, e.pos, 10, color, 90.0);
        }
        for ev in &self.enemy_events {
            match ev {
                enemy::Event::Charge => sfx.play(sfx::Id::Charge),
                enemy::Event::WallHit => {
                    sfx.play(sfx::Id::WallHit);
                    self.feel.add_trauma(0.2);
                }
                enemy::Event::Shoot => sfx.play(sfx::Id::Shoot),
            }
        }
        for p in &mut self.projectiles {
            p.update(&self.room, dt);
            if !p.alive {
                self.fx.push(ring(p.pos, 2.0, 6.0, 0.12, PROJECTILE));
            }
        }

        // 3. 플레이어 공격 → 적·투사체
        if let Some(s) = strike {
            let origin = self.player.pos;
            for e in &mut self.enemies {
                if e.last_hit_by == s.id || !s.shape.overlaps_circle(e.pos, e.radius()) {
                    continue;
                }
                e.last_hit_by = s.id;
                let dir = (e.pos - origin).normalize_or_zero();
                let dir = if dir == Vec2::ZERO {
                    self.player.facing
                } else {
                    dir
                };
                e.hurt(s.damage, dir, s.knockback, s.stun);
                if matches!(s.kind, crate::combat::StrikeKind::Claw(_)) {
                    self.player.gain_sp(s.finisher);
                }
                if s.poison > 0.0 && e.alive {
                    e.apply_poison(s.poison);
                }
                if s.finisher {
                    self.finisher_hits += 1;
                }
                self.feel.add_hitstop(s.hitstop);
                self.feel.add_trauma(s.shake);
                let hit_point = e.pos - dir * e.radius() * 0.5;
                self.fx.push(ring(hit_point, 2.0, 9.0, 0.12, WHITE));
                if e.alive {
                    sfx.play(sfx::Id::EnemyHit);
                } else {
                    self.kills += 1;
                    deaths.push((e.pos, e.kind));
                    if s.kind == crate::combat::StrikeKind::Emerge {
                        self.emerge_kills += 1;
                    }
                    sfx.play(sfx::Id::EnemyDie);
                    self.feel.add_trauma(t::ENEMY_DIE_SHAKE);
                    let color = enemy_color(e.kind);
                    self.fx
                        .push(ring(e.pos, e.radius(), e.radius() * 3.0, 0.25, color));
                    burst(&mut self.fx, e.pos, 12, color, 110.0);
                }
            }
            for p in &mut self.projectiles {
                if p.alive && s.shape.overlaps_circle(p.pos, p.radius()) {
                    p.alive = false;
                    self.fx.push(ring(p.pos, 2.0, 8.0, 0.12, PROJECTILE));
                }
            }
        }

        for (pos, kind) in deaths {
            self.drop_loot(pos, kind);
        }

        // 4. 적·투사체 → 플레이어
        if self.player.is_alive() && !self.player.is_invulnerable() {
            let pr = self.player.radius();
            let ppos = self.player.pos;
            let mut hurt_from = None;
            for e in &mut self.enemies {
                if let Some(dmg) = e.contact_damage() {
                    let rr = pr + e.radius();
                    if (e.pos - ppos).length_squared() <= rr * rr {
                        e.on_hit_player();
                        hurt_from = Some((e.pos, dmg));
                        break;
                    }
                }
            }
            if hurt_from.is_none() {
                for p in &mut self.projectiles {
                    let rr = pr + p.radius();
                    if p.alive && (p.pos - ppos).length_squared() <= rr * rr {
                        p.alive = false;
                        hurt_from = Some((p.pos, t::PROJECTILE_DAMAGE));
                        break;
                    }
                }
            }
            if let Some((from, dmg)) = hurt_from {
                if self.player.take_damage(from, dmg) {
                    self.feel.add_hitstop(t::PLAYER_HURT_HITSTOP);
                    self.feel.add_trauma(t::PLAYER_HURT_SHAKE);
                    self.fx.push(ring(self.player.pos, 4.0, 14.0, 0.2, RED));
                    if self.player.is_alive() {
                        sfx.play(sfx::Id::Hurt);
                    } else {
                        sfx.play(sfx::Id::Die);
                        burst(&mut self.fx, self.player.pos, 16, MOLE, 90.0);
                    }
                }
            }
        }

        // 5. 적끼리 겹침 방지
        for i in 0..self.enemies.len() {
            for j in (i + 1)..self.enemies.len() {
                let (a, b) = self.enemies.split_at_mut(j);
                let (ea, eb) = (&mut a[i], &mut b[0]);
                let d = eb.pos - ea.pos;
                let min = ea.radius() + eb.radius();
                let dist2 = d.length_squared();
                if dist2 < min * min {
                    let push = if dist2 > 1e-4 {
                        d / dist2.sqrt()
                    } else {
                        vec2(1.0, 0.0)
                    } * t::ENEMY_SEPARATION_SPEED
                        * dt;
                    ea.pos -= push;
                    eb.pos += push;
                    let (ra, rb) = (ea.radius(), eb.radius());
                    self.room.resolve_circle(&mut ea.pos, ra);
                    self.room.resolve_circle(&mut eb.pos, rb);
                }
            }
        }

        // 6. 정리
        self.enemies.retain(|e| e.alive);
        self.projectiles.retain(|p| p.alive);
        for f in &mut self.fx {
            f.time += dt;
            f.pos += f.vel * dt;
            f.vel *= 0.9;
        }
        self.fx.retain(|f| f.time < f.dur);

        // 7. 웨이브
        if self.auto_waves && self.player.is_alive() && self.enemies.is_empty() {
            self.respawn_timer -= dt;
            if self.respawn_timer <= 0.0 {
                self.next_wave();
                self.respawn_timer = t::WAVE_RESPAWN_DELAY;
            }
        }
    }

    /// 장신구·씨앗으로 능력치를 다시 계산한다.
    pub fn refresh_stats(&mut self) {
        self.player
            .set_stats(Stats::compute(&self.charms, self.life_seeds));
    }

    /// 퀵 슬롯 아이템을 먹는다. 체력이 가득이면 먹지 않는다(아깝지 않게).
    fn use_quick_item(&mut self, sfx: &Sfx) {
        let p = &mut self.player;
        if p.hp >= p.stats.max_hp {
            return;
        }
        let c = self.bag.selected;
        if !self.bag.take(c) {
            return;
        }
        match c {
            Consumable::Berry => p.heal(t::BERRY_HEAL),
            Consumable::Worm => p.heal(p.stats.max_hp),
        }
        sfx.play(sfx::Id::Heal);
        let pos = p.pos;
        self.fx.push(ring(pos, 3.0, 14.0, 0.3, BERRY));
        burst(&mut self.fx, pos, 8, BERRY, 60.0);
        if self.bag.count(c) == 0 {
            self.bag.cycle();
        }
    }

    /// 쥬시 지렁이로 일어난다: 체력 일부, 잠깐 무적, 주변을 밀어낸다.
    fn revive(&mut self, sfx: &Sfx) {
        self.player.revive();
        let pos = self.player.pos;
        for e in &mut self.enemies {
            let d = e.pos - pos;
            if d.length() < t::REVIVE_PUSH_RADIUS {
                e.hurt(0, d.normalize_or_zero(), t::REVIVE_PUSH_KNOCKBACK, 0.5);
            }
        }
        for p in &mut self.projectiles {
            if (p.pos - pos).length() < t::REVIVE_PUSH_RADIUS {
                p.alive = false;
            }
        }
        sfx.play(sfx::Id::Heal);
        self.fx
            .push(ring(pos, 4.0, t::REVIVE_PUSH_RADIUS, 0.4, BERRY));
        burst(&mut self.fx, pos, 20, BERRY, 120.0);
        self.feel.add_trauma(0.4);
        self.toast = Some((|s| s.toast_revive, 2.5));
    }

    /// 쓰러진 적이 떨어뜨린다: 반딧불 몇 마리 + 확률로 산딸기(loot::berry_chance).
    fn drop_loot(&mut self, pos: Vec2, kind: Kind) {
        let (lo, hi) = loot::glowbugs_for(kind);
        if hi > 0 {
            for _ in 0..rand::gen_range(lo, hi + 1) {
                self.loot.push(Loot::spawn(LootKind::Glowbug, pos));
            }
        }
        if kind != Kind::Dummy && rand::gen_range(0.0, 1.0) < t::WORM_CHANCE {
            self.loot.push(Loot::spawn(LootKind::Worm, pos));
        }
        let max = self.player.stats.max_hp;
        let missing = self.player.hp < max;
        let p = loot::berry_chance(self.player.hp, max, self.kills_since_berry, kind);
        if p > 0.0 && rand::gen_range(0.0, 1.0) < p {
            self.loot.push(Loot::spawn(LootKind::Berry, pos));
            self.kills_since_berry = 0;
        } else if missing && kind != Kind::Dummy {
            self.kills_since_berry += 1;
        }
    }

    /// 보간이 멈춘 화면(일시정지·히트스톱) 위에서 미끄러지지 않게 prev를 현재로 맞춘다.
    pub fn freeze(&mut self) {
        self.freeze_interp();
    }

    fn freeze_interp(&mut self) {
        self.player.prev_pos = self.player.pos;
        for e in &mut self.enemies {
            e.prev_pos = e.pos;
        }
        for p in &mut self.projectiles {
            p.prev_pos = p.pos;
        }
        for l in &mut self.loot {
            l.prev_pos = l.pos;
        }
    }

    pub fn next_wave(&mut self) {
        self.wave += 1;
        if self.biome_by_wave {
            self.biome = Biome::for_wave(self.wave);
        }
        let (chargers, shooters) = match self.wave {
            1 => (1, 1),
            2 => (2, 1),
            3 => (2, 2),
            _ => (3, 2),
        };
        let ppos = self.player.pos;
        let mut points: Vec<Vec2> = self
            .room
            .spawn_points
            .iter()
            .copied()
            .filter(|p| (*p - ppos).length() >= t::SPAWN_MIN_DIST)
            .collect();
        // 섞기 (Fisher–Yates)
        for i in (1..points.len()).rev() {
            let j = rand::gen_range(0, i + 1);
            points.swap(i, j);
        }
        let mut it = points.into_iter().cycle();
        for _ in 0..chargers {
            if let Some(p) = it.next() {
                self.enemies.push(Enemy::new(Kind::Charger, p));
            }
        }
        for _ in 0..shooters {
            if let Some(p) = it.next() {
                self.enemies.push(Enemy::new(Kind::Shooter, p));
            }
        }
    }
}

// ---- 팔레트(렌더와 공유) ---------------------------------------------------------
pub const MOLE: Color = Color::new(0.90, 0.71, 0.47, 1.0);
pub const DIRT: Color = Color::new(0.55, 0.40, 0.28, 1.0);
pub const CHARGER: Color = Color::new(0.85, 0.30, 0.30, 1.0);
pub const SHOOTER: Color = Color::new(0.60, 0.40, 0.85, 1.0);
pub const PROJECTILE: Color = Color::new(1.0, 0.85, 0.30, 1.0);
pub const POISON: Color = Color::new(0.55, 0.90, 0.35, 1.0);
/// 반딧불 빛.
pub const GLOW: Color = Color::new(0.85, 1.0, 0.45, 1.0);
/// 산딸기.
pub const BERRY: Color = Color::new(0.93, 0.30, 0.38, 1.0);

pub fn enemy_color(kind: Kind) -> Color {
    match kind {
        Kind::Charger => CHARGER,
        Kind::Shooter => SHOOTER,
        Kind::Dummy => Color::new(0.55, 0.52, 0.48, 1.0),
    }
}

fn ring(pos: Vec2, r0: f32, r1: f32, dur: f32, color: Color) -> Fx {
    Fx {
        pos,
        vel: Vec2::ZERO,
        time: 0.0,
        dur,
        kind: FxKind::Ring { r0, r1, color },
    }
}

fn burst(fx: &mut Vec<Fx>, pos: Vec2, n: usize, color: Color, speed: f32) {
    for _ in 0..n {
        let ang = rand::gen_range(0.0, std::f32::consts::TAU);
        let sp = rand::gen_range(0.3, 1.0) * speed;
        fx.push(Fx {
            pos,
            vel: vec2(ang.cos(), ang.sin()) * sp,
            time: 0.0,
            dur: rand::gen_range(0.15, 0.35),
            kind: FxKind::Spark { color },
        });
    }
}
