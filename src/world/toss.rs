//! 도토의 도토리(0195): 던진 탄과 묻은 도토리. 플레이어가 `Player::spawns`에 쌓은 것을 월드가 받아 날리고 터뜨린다.
//! 탄이 맞는 순간은 손으로 때린 것과 같은 길(`World::resolve_strike`)을 탄다 — 피해·멈춤·킥·등딱지·게이지 규칙을
//! 두 번 쓰지 않는다. 적의 탄(`Projectile`)과는 따로 둔다(받아치기·플레이어 피격 규칙이 섞이지 않게).

use super::*;
use crate::combat::{Shape, StrikeKind};
use crate::player::Spawn;

/// 날아가는 도토리 하나.
pub struct Shot {
    pub pos: Vec2,
    pub prev: Vec2,
    pub vel: Vec2,
    /// 남은 거리(px). 0이 되면 떨어진다(큰 도토리는 그 자리에서 튄다).
    pub left: f32,
    pub radius: f32,
    pub splash: f32,
    /// 남은 벽 튕김(솔방울), 멈출 때 갈라질 조각 수(호두), 멈춘 자리에 가시(밤송이). 0246.
    pub bounce: u8,
    pub split: u8,
    pub thorns: bool,
    /// 맞을 때 쓸 타격(모양·번호는 맞는 순간 정한다).
    pub strike: Strike,
    /// 날아가는 동안 그물·고치·매듭·부술 것에 쓰는 번호(같은 탄이 같은 것을 두 번 치지 않게).
    pub id: u32,
}

/// 묻은 도토리 하나(적만 맞는다).
pub struct Buried {
    pub pos: Vec2,
    pub time: f32,
    pub fuse: f32,
    pub radius: f32,
    pub damage: i32,
}

/// 가시 덤불(가시덩굴 각반·밤송이, 0246): `left`초 남아 `THORN_PATCH_PERIOD`마다 둘레의 적을 찌른다(적만).
pub struct Patch {
    pub pos: Vec2,
    pub left: f32,
    pub pulse: f32,
}

#[derive(Default)]
pub struct Toss {
    pub shots: Vec<Shot>,
    pub buried: Vec<Buried>,
    pub patches: Vec<Patch>,
    next_id: u32,
}

impl Toss {
    /// 탄·터짐의 타격 번호: 플레이어 스윙 번호(작은 수)와 겹치지 않게 윗자리에서 센다.
    fn id(&mut self) -> u32 {
        self.next_id = self.next_id.wrapping_add(1);
        0x8000_0000 | (self.next_id & 0x7fff_ffff)
    }

    pub fn clear(&mut self) {
        self.shots.clear();
        self.buried.clear();
        self.patches.clear();
    }

    /// 가시 덤불을 놓는다(많으면 오래된 것부터 뺀다).
    fn plant(&mut self, at: Vec2) {
        if self.patches.len() >= t::THORN_PATCH_MAX {
            self.patches.remove(0);
        }
        self.patches.push(Patch {
            pos: at,
            left: t::THORN_PATCH_TIME,
            // 놓자마자 한 번 찌른다.
            pulse: 0.0,
        });
    }
}

/// 도토리 색(그림·이펙트).
pub(crate) const ACORN: Color = Color::new(0.78, 0.52, 0.26, 1.0);
/// 가시 덤불 잎 색(흙 튀김).
pub(crate) const THORN_GREEN: Color = Color::new(0.42, 0.62, 0.30, 1.0);

impl World {
    /// 플레이어가 이번 틱에 내놓은 탄·묻은 도토리를 받는다.
    pub(super) fn take_spawns(&mut self) {
        for sp in std::mem::take(&mut self.player.spawns) {
            match sp {
                Spawn::Shot {
                    from,
                    vel,
                    reach,
                    radius,
                    splash,
                    bounce,
                    split,
                    thorns,
                    strike,
                } => {
                    let id = self.toss.id();
                    self.toss.shots.push(Shot {
                        pos: from,
                        prev: from,
                        vel,
                        left: reach,
                        radius,
                        splash,
                        bounce,
                        split,
                        thorns,
                        strike,
                        id,
                    });
                }
                Spawn::Thorns { at } => {
                    let mut p = at;
                    self.room.resolve_circle(&mut p, 4.0);
                    self.toss.plant(p);
                    clods(&mut self.juice.fx, p, 4, THORN_GREEN, 40.0);
                }
                Spawn::Bury {
                    at,
                    fuse,
                    radius,
                    damage,
                } => {
                    let mut p = at;
                    self.room.resolve_circle(&mut p, 4.0);
                    self.toss.buried.push(Buried {
                        pos: p,
                        time: 0.0,
                        fuse,
                        radius,
                        damage,
                    });
                    clods(&mut self.juice.fx, p, 5, DIRT, 50.0);
                }
            }
        }
    }

    /// 탄을 날리고 묻은 도토리를 터뜨린다. 시동(멈춤) 중에는 그대로.
    pub(super) fn tick_toss(&mut self, dt: f32, frozen: bool, deaths: &mut Vec<(Vec2, Kind)>) {
        if frozen {
            return;
        }
        let mut shots = std::mem::take(&mut self.toss.shots);
        shots.retain_mut(|s| self.fly(s, dt, deaths));
        // 날던 중에 새로 생긴 탄(없지만 혹시)을 잃지 않게 합친다.
        shots.append(&mut self.toss.shots);
        self.toss.shots = shots;

        let mut popped = Vec::new();
        for b in &mut self.toss.buried {
            b.time += dt;
            if b.time >= b.fuse {
                popped.push((b.pos, b.radius, b.damage));
            }
        }
        self.toss.buried.retain(|b| b.time < b.fuse);
        for (at, r, damage) in popped {
            let id = self.toss.id();
            let s = Strike {
                id,
                kind: StrikeKind::Burst,
                shape: Shape::Circle { center: at, r },
                damage,
                knockback: t::BURY_KNOCKBACK,
                stun: t::BURY_STUN,
                hitstop: 0.06,
                shake: 0.3,
                finisher: false,
                poison: 0.0,
                crush: false,
            };
            self.juice.audio.play(sfx::Id::Emerge);
            self.juice.feel.add_trauma(0.15);
            self.juice.fx.push(ring(at, 3.0, r + 2.0, 0.25, ACORN));
            burst(&mut self.juice.fx, at, 12, DIRT, 110.0);
            self.touch_world(&s);
            self.resolve_strike(s, at, Vec2::ZERO, true, deaths);
        }
        self.tick_patches(dt, deaths);
    }

    /// 가시 덤불: 때마다 둘레의 적을 찌른다(작게 밀고 잠깐 멈춘다). 플레이어는 안 다친다.
    fn tick_patches(&mut self, dt: f32, deaths: &mut Vec<(Vec2, Kind)>) {
        let mut pricks = Vec::new();
        for p in &mut self.toss.patches {
            p.left -= dt;
            p.pulse -= dt;
            if p.pulse <= 0.0 && p.left > 0.0 {
                p.pulse = t::THORN_PATCH_PERIOD;
                pricks.push(p.pos);
            }
        }
        self.toss.patches.retain(|p| p.left > 0.0);
        for at in pricks {
            let r = t::THORN_PATCH_RADIUS;
            let near = self
                .enemies
                .iter()
                .any(|e| e.alive && e.hittable() && (e.pos - at).length() <= r + e.radius());
            if !near {
                continue;
            }
            let s = Strike {
                id: self.toss.id(),
                kind: StrikeKind::Prick,
                shape: Shape::Circle { center: at, r },
                damage: t::THORN_PATCH_DAMAGE + self.player.stats.damage,
                knockback: 40.0,
                stun: 0.2,
                hitstop: 0.0,
                shake: 0.05,
                finisher: false,
                poison: 0.0,
                crush: false,
            };
            self.resolve_strike(s, at, Vec2::ZERO, true, deaths);
        }
    }

    /// 탄 한 틱: 움직이고, 벽·적·그물·고치·부술 것·매듭에 닿는지. 계속 날면 true.
    fn fly(&mut self, s: &mut Shot, dt: f32, deaths: &mut Vec<(Vec2, Kind)>) -> bool {
        let step = s.vel * dt;
        s.prev = s.pos;
        s.pos += step;
        s.left -= step.length();
        let dir = s.vel.normalize_or_zero();
        // 날아가는 동안 닿는 것(적 말고): 같은 번호라 한 번씩만.
        let touch = Strike {
            id: s.id,
            shape: Shape::Circle {
                center: s.pos,
                r: s.radius + 1.0,
            },
            ..s.strike
        };
        let props = self.room.props.len();
        let cocoon_was = self.hazards.cocoon_bounce_id;
        self.touch_world(&touch);
        let stopped = self.room.props.len() < props || self.hazards.cocoon_bounce_id != cocoon_was;
        let wall = self.room.is_solid(s.pos, s.radius);
        let enemy = self.enemies.iter().any(|e| {
            e.alive && e.hittable() && {
                let rr = e.radius() + s.radius;
                (e.pos - s.pos).length_squared() <= rr * rr
            }
        });
        if !(enemy || wall || stopped || s.left <= 0.0) {
            return true;
        }
        // 솔방울(0246): 벽이면 한 번 튕겨 계속 난다. 축마다 막혔는지 보고 그 성분만 뒤집는다.
        if wall && !enemy && !stopped && s.bounce > 0 && s.left > 0.0 {
            s.bounce -= 1;
            let back = s.prev;
            if self.room.is_solid(vec2(s.pos.x, back.y), s.radius) {
                s.vel.x = -s.vel.x;
            }
            if self.room.is_solid(vec2(back.x, s.pos.y), s.radius) {
                s.vel.y = -s.vel.y;
            }
            if s.vel.normalize_or_zero() == dir {
                s.vel = -s.vel;
            }
            s.pos = back;
            self.juice.fx.push(ring(s.pos, 1.5, 4.0, 0.1, ACORN));
            return true;
        }
        let stop_at = if wall { s.prev } else { s.pos };
        // 맞은 적 뒤쪽에서 갈라진다(맞은 적을 조각이 또 치지 않게).
        let split_at = self
            .enemies
            .iter()
            .filter(|e| e.alive && e.hittable())
            .find(|e| (e.pos - s.pos).length() <= e.radius() + s.radius)
            .map_or(stop_at, |e| {
                e.pos + dir * (e.radius() + t::SHARD_RADIUS + 2.0)
            });
        // 호두(0246): 멈추는 자리에서 조각으로 갈라진다(뒤로 튀지 않게 앞쪽 부채꼴).
        if s.split > 0 {
            let n = s.split as i32;
            for i in 0..n {
                let id = self.toss.id();
                let off = (i as f32 - (n - 1) as f32 * 0.5) * t::SHARD_SPREAD;
                let d = Vec2::from_angle(off).rotate(if wall { -dir } else { dir });
                self.toss.shots.push(Shot {
                    pos: split_at,
                    prev: split_at,
                    vel: d * t::SHARD_SPEED,
                    left: t::SHARD_REACH,
                    radius: t::SHARD_RADIUS,
                    splash: 0.0,
                    bounce: 0,
                    split: 0,
                    thorns: false,
                    strike: Strike {
                        damage: 1 + self.player.stats.damage,
                        knockback: 60.0,
                        finisher: false,
                        ..s.strike
                    },
                    id,
                });
            }
            clods(&mut self.juice.fx, split_at, 5, ACORN, 80.0);
        }
        // 밤송이(0246): 멈춘 자리에 가시.
        if s.thorns {
            self.toss.plant(stop_at);
        }
        if enemy || s.splash > 0.0 {
            let r = if s.splash > 0.0 {
                s.splash
            } else {
                s.radius + 2.0
            };
            let hit = Strike {
                id: self.toss.id(),
                shape: Shape::Circle { center: s.pos, r },
                ..s.strike
            };
            if s.splash > 0.0 {
                self.juice.audio.play(sfx::Id::Impact);
                self.juice.fx.push(ring(s.pos, 3.0, r + 2.0, 0.22, ACORN));
                clods(&mut self.juice.fx, s.pos, 8, ACORN, 110.0);
                self.touch_world(&hit);
            }
            self.resolve_strike(hit, s.pos - dir * 8.0, dir, true, deaths);
        } else {
            // 벽에 맞거나 다 날아 떨어졌다.
            self.juice.fx.push(ring(s.pos, 1.5, 5.0, 0.12, ACORN));
        }
        false
    }

    /// 적 말고 타격이 닿는 것들(손 타격은 `tick_player`가 부른다): 가장 깊은 곳의 매듭, 흰 실 그물, 고치, 부술 것.
    pub(super) fn touch_world(&mut self, s: &Strike) {
        self.deep_hit(s);
        self.web_cut(s);
        self.cocoon_hit(s);
        self.prop_break(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::equipment::{Character, ACORN, WEAPONS};

    /// 빈 방의 도토(도토리 든). 오른쪽을 본다.
    fn doto() -> World {
        let mut w = World::empty();
        w.character = Character::Doto;
        w.player.kit = Character::Doto;
        w.player.moveset = WEAPONS[ACORN].moveset;
        w.player.facing = vec2(1.0, 0.0);
        w
    }

    fn press() -> Actions {
        Actions {
            attack: true,
            attack_held: true,
            aim: vec2(1.0, 0.0),
            ..Default::default()
        }
    }

    fn run(w: &mut World, a: &Actions, ticks: usize) {
        for _ in 0..ticks {
            w.tick(a);
        }
    }

    /// 달리며 모으기(0215): 회피를 쥐고 달려 멀리뛰기를 뛸 수 있게 되면 땅울림이 켜지고, 떼서 뛰면 꺼진다.
    #[test]
    fn running_rumbles_once_the_jump_is_ready() {
        let mut w = doto();
        w.player.pos = w.room.center() - vec2(60.0, 0.0);
        let hold = Actions {
            move_dir: vec2(1.0, 0.0),
            dodge: true,
            dodge_held: true,
            ..Default::default()
        };
        w.tick(&hold);
        let keep = Actions {
            dodge: false,
            ..hold
        };
        run(&mut w, &keep, 20);
        assert!(matches!(w.player.state, crate::player::State::Run { .. }));
        run(&mut w, &keep, (t::RUN_MIN / t::TICK_DT) as usize + 2);
        assert!(w.run_rumble, "뛸 수 있게 되면 운다");
        w.tick(&Actions::default());
        assert!(!w.run_rumble, "떼서 뛰면 그친다");
    }

    /// 플레이어에서 `off`만큼 떨어진 자리에 허수아비.
    fn dummy(w: &mut World, off: Vec2) -> usize {
        let at = w.player.pos + off;
        w.spawn_mob(Kind::Dummy, at);
        // 맞아도 쓰러지지 않게(목록에서 빠지면 번호가 바뀐다).
        let i = w.enemies.len() - 1;
        w.enemies[i].max_hp = 99;
        w.enemies[i].hp = 99;
        i
    }

    /// 던지기(0195): 1타는 탄 하나가 멀리 있는 적을 맞힌다. 손앞의 짧은 판정은 바로 옆 적도 때리지 않는다.
    #[test]
    fn a_throw_hits_from_afar_and_the_hand_hits_no_one() {
        let mut w = doto();
        let far = dummy(&mut w, vec2(80.0, 0.0));
        let near = dummy(&mut w, vec2(0.0, 9.0));
        let (hp_far, hp_near) = (w.enemies[far].hp, w.enemies[near].hp);
        w.tick(&press());
        run(&mut w, &Actions::default(), 4);
        assert_eq!(w.toss.shots.len(), 1, "1타 = 탄 하나");
        run(&mut w, &Actions::default(), 40);
        assert!(w.enemies[far].hp < hp_far, "멀리 있는 적이 맞았다");
        assert_eq!(w.enemies[near].hp, hp_near, "손은 적을 때리지 않는다");
        assert!(w.toss.shots.is_empty());
    }

    /// 3타는 부채꼴 셋, 다 날면(사거리 끝) 떨어진다.
    #[test]
    fn the_third_throw_is_a_fan_of_three_that_drops_at_its_reach() {
        let mut w = doto();
        let mut most = 0;
        for _ in 0..3 {
            w.tick(&press());
            for _ in 0..30 {
                w.tick(&Actions::default());
                most = most.max(w.toss.shots.len());
            }
        }
        assert_eq!(most, 3, "마지막 타는 셋");
        run(&mut w, &Actions::default(), 60);
        assert!(w.toss.shots.is_empty(), "사거리 끝에서 떨어진다");
    }

    /// 강타 = 큰 도토리(0195): 맞은 자리에서 튀어 둘레의 적도 맞는다.
    #[test]
    fn the_heavy_is_a_big_acorn_that_splashes() {
        let mut w = doto();
        let a = dummy(&mut w, vec2(70.0, 0.0));
        let b = dummy(&mut w, vec2(70.0, 16.0));
        let (ha, hb) = (w.enemies[a].hp, w.enemies[b].hp);
        let hold = Actions {
            attack_held: true,
            aim: vec2(1.0, 0.0),
            ..Default::default()
        };
        w.tick(&press());
        // 1타 탄이 a를 맞히기 전에, 계속 눌러 모았다가 뗀다.
        run(&mut w, &hold, (t::HEAVY_CHARGE / t::TICK_DT) as usize + 12);
        let hit_by_first = ha - w.enemies[a].hp;
        // 뗀 틱에 강타가 시작되고, 다음 틱(판정 첫 틱)에 큰 도토리가 나간다(0189).
        let mut big = false;
        for _ in 0..3 {
            w.tick(&Actions::default());
            big |= w.toss.shots.iter().any(|s| s.splash > 0.0);
        }
        assert!(big, "떼면 큰 도토리가 나간다");
        run(&mut w, &Actions::default(), 60);
        assert!(ha - w.enemies[a].hp > hit_by_first, "큰 도토리가 맞았다");
        assert!(w.enemies[b].hp < hb, "튀어서 옆의 적도");
    }

    /// 뒤로 뛰기(0195): 서 있으면 겨눈 쪽 반대로 물러나고, 공중에서는 안 맞고, 겨눈 쪽은 그대로.
    #[test]
    fn back_leap_retreats_from_the_aim_and_is_untouchable() {
        let mut w = doto();
        let x0 = w.player.pos.x;
        w.tick(&Actions {
            dodge: true,
            ..Default::default()
        });
        assert!(matches!(w.player.state, crate::player::State::Leap { .. }));
        assert!(w.player.is_invulnerable() && w.player.height() >= 0.0);
        run(
            &mut w,
            &Actions::default(),
            (t::BACKLEAP_TIME / t::TICK_DT) as usize + 2,
        );
        assert!(
            w.player.pos.x < x0 - t::BACKLEAP_DIST * 0.8,
            "뒤로 물러났다"
        );
        assert_eq!(w.player.facing, vec2(1.0, 0.0), "겨눈 쪽은 그대로");
        assert!(w.player.burrow_cd > 0.0);
    }

    /// 도토리 묻기(0195): 게이지 가득 + 특기 → 앞에 셋, 잠시 뒤 터져 적만 맞는다. 게이지는 빈다.
    #[test]
    fn buried_acorns_burst_on_enemies_only() {
        let mut w = doto();
        w.player.sp = t::SP_MAX;
        let e = dummy(&mut w, vec2(t::BURY_AHEAD, 0.0));
        let hp = w.enemies[e].hp;
        let php = w.player.hp;
        w.tick(&Actions {
            special: true,
            aim: vec2(1.0, 0.0),
            ..Default::default()
        });
        assert_eq!(w.toss.buried.len(), t::BURY_COUNT);
        assert_eq!(w.player.sp, 0.0);
        // 플레이어가 묻은 자리 위에 서 있어도 안 아프다.
        w.player.pos = w.toss.buried[0].pos;
        run(
            &mut w,
            &Actions::default(),
            (t::BURY_FUSE / t::TICK_DT) as usize + 4,
        );
        assert!(w.toss.buried.is_empty(), "다 터졌다");
        assert!(w.enemies[e].hp < hp, "적이 맞았다");
        assert_eq!(w.player.hp, php, "제 도토리에는 안 다친다");
    }

    /// 받아치기(0195): 던지는 순간 손앞의 판정이 날아오는 탄을 되받아친다.
    #[test]
    fn the_throwing_hand_parries_incoming_shots() {
        let mut w = doto();
        let p = w.player.pos;
        w.projectiles.push(Projectile {
            pos: p + vec2(20.0, 0.0),
            prev_pos: p + vec2(22.0, 0.0),
            vel: vec2(-120.0, 0.0),
            alive: true,
            friendly: false,
        });
        w.tick(&press());
        run(&mut w, &Actions::default(), 4);
        assert!(
            w.projectiles.iter().any(|b| b.alive && b.friendly),
            "되받아쳤다"
        );
    }

    /// 그 캐릭터·무기로 바꾼다(0246).
    fn arm(w: &mut World, ch: Character, weapon: usize) {
        w.character = ch;
        w.player.kit = ch;
        w.player.moveset = WEAPONS[weapon].moveset;
    }

    /// 한 콤보(타 수만큼 누르고 기다리기)를 끝까지.
    fn combo(w: &mut World) {
        for _ in 0..w.player.moveset.hits.len() {
            w.tick(&press());
            run(w, &Actions::default(), 26);
        }
    }

    /// 솔방울(0246): 벽에 닿으면 한 번 튕겨 되돌아 난다.
    #[test]
    fn pinecones_bounce_off_a_wall_once() {
        let mut w = doto();
        arm(&mut w, Character::Doto, crate::equipment::PINECONE);
        // 오른쪽 벽 가까이에서 오른쪽으로.
        let mut x = w.player.pos.x;
        while !w.room.is_solid(vec2(x + 30.0, w.player.pos.y), 3.0) {
            x += 4.0;
        }
        w.player.pos.x = x;
        w.tick(&press());
        let mut bounced = false;
        for _ in 0..40 {
            w.tick(&Actions::default());
            bounced |= w.toss.shots.iter().any(|s| s.vel.x < 0.0 && s.bounce == 0);
        }
        assert!(bounced, "벽에 튕겨 왼쪽으로 난다");
    }

    /// 호두(0246): 맞은 자리에서 조각 셋으로 갈라지고, 조각이 뒤의 적을 맞힌다(맞은 적은 다시 치지 않는다).
    #[test]
    fn walnuts_split_and_the_shards_fly_on() {
        let mut w = doto();
        arm(&mut w, Character::Doto, crate::equipment::WALNUT);
        let front = dummy(&mut w, vec2(50.0, 0.0));
        let back = dummy(&mut w, vec2(72.0, 0.0));
        let (hf, hb) = (w.enemies[front].hp, w.enemies[back].hp);
        w.tick(&press());
        let mut most = 0;
        for _ in 0..50 {
            w.tick(&Actions::default());
            most = most.max(w.toss.shots.len());
        }
        assert!(most >= 3, "조각 셋");
        assert_eq!(hf - w.enemies[front].hp, 2, "앞의 적은 호두에만(피해 2)");
        assert!(w.enemies[back].hp < hb, "뒤의 적은 조각에");
    }

    /// 밤송이(0246): 셋째 던지기가 떨어진 자리에 가시를 남기고, 그 위의 적이 찔린다(플레이어는 안 다친다).
    #[test]
    fn chestnut_burrs_leave_thorns_that_prick_enemies() {
        let mut w = doto();
        arm(&mut w, Character::Doto, crate::equipment::CHESTNUT);
        combo(&mut w);
        run(&mut w, &Actions::default(), 30);
        assert!(!w.toss.patches.is_empty(), "가시가 남았다");
        let at = w.toss.patches[0].pos;
        w.spawn_mob(Kind::Dummy, at);
        let e = w.enemies.len() - 1;
        w.enemies[e].max_hp = 99;
        w.enemies[e].hp = 99;
        let php = w.player.hp;
        w.player.pos = at + vec2(0.0, 2.0);
        run(&mut w, &Actions::default(), 40);
        assert!(w.enemies[e].hp < 99, "찔렸다");
        assert_eq!(w.player.hp, php);
        run(
            &mut w,
            &Actions::default(),
            (t::THORN_PATCH_TIME / t::TICK_DT) as usize,
        );
        assert!(w.toss.patches.is_empty(), "시간이 지나면 사라진다");
    }

    /// 가시덩굴 각반(0246): 마무리 차기가 발 앞에 가시를 남긴다.
    #[test]
    fn briar_finisher_leaves_thorns() {
        let mut w = doto();
        arm(&mut w, Character::Momo, crate::equipment::BRIAR);
        combo(&mut w);
        assert_eq!(w.toss.patches.len(), 1);
        assert!(
            w.toss.patches[0].pos.x > w.player.pos.x - 4.0,
            "앞쪽(오른쪽)에"
        );
    }

    /// 꽃잎 발목끈(0246): 박자 창이 넓고, 박자에 맞춘 타는 기를 두 배로 채운다.
    #[test]
    fn petal_anklet_widens_the_beat_and_doubles_ki() {
        let mut w = doto();
        arm(&mut w, Character::Momo, crate::equipment::PETAL);
        assert!(w.player.beat_window() > t::BEAT_WINDOW * 1.5);
        w.player.sp = 0.0;
        w.player.on_beat = false;
        w.player.gain_sp(false);
        let off = w.player.sp;
        w.player.sp = 0.0;
        w.player.on_beat = true;
        w.player.gain_sp(false);
        assert!((w.player.sp - off * t::PETAL_BEAT_SP).abs() < 1e-4);
    }
}
