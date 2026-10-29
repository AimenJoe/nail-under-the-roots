//! 쉼터부터 이어하기(docs/decisions/0080): 진행 중인 런을 쉼터에서 적고(`suspend.rs` 형식), 타이틀의 이어하기로 되돌린다.

use super::{App, Inventory, Item, Scene, Tally};
use crate::adventure::Place;
use crate::equipment::Character;
use crate::items::Consumable;
use crate::profile;
use crate::scenario::Scenario;
use crate::suspend::{RunSave, Slot};
use crate::world::World;

/// 이번 런 통계(상태 탭·저널 셈의 바탕). 이름은 저장 글의 키다 — 바꾸지 않는다.
fn stat_fields(w: &World) -> [(&'static str, u32); 11] {
    [
        ("kills", w.stats.kills),
        ("emerge_kills", w.stats.emerge_kills),
        ("parries", w.stats.parries),
        ("items_used", w.stats.items_used),
        ("drills", w.stats.drills),
        ("emerge_hits", w.stats.emerge_hits),
        ("chests", w.stats.chests),
        ("springs", w.stats.springs),
        ("slams", w.stats.slams),
        ("finisher_hits", w.stats.finisher_hits),
        ("burrows", w.stats.burrows),
    ]
}

fn set_stat(w: &mut World, name: &str, n: u32) {
    let f = match name {
        "kills" => &mut w.stats.kills,
        "emerge_kills" => &mut w.stats.emerge_kills,
        "parries" => &mut w.stats.parries,
        "items_used" => &mut w.stats.items_used,
        "drills" => &mut w.stats.drills,
        "emerge_hits" => &mut w.stats.emerge_hits,
        "chests" => &mut w.stats.chests,
        "springs" => &mut w.stats.springs,
        "slams" => &mut w.stats.slams,
        "finisher_hits" => &mut w.stats.finisher_hits,
        "burrows" => &mut w.stats.burrows,
        _ => return,
    };
    *f = n;
}

impl App {
    /// 지금 런을 적을 값.
    fn capture_run(&self) -> RunSave {
        let w = &self.world;
        RunSave {
            stage: w.adv.stage,
            seed: w.adv_seed(),
            depth: w.adv.depth,
            character: w.character.def().code,
            knots: w.knots.level,
            seeds: w.inv.seeds,
            glowbugs: w.inv.glowbugs,
            berries: w.inv.bag.berries,
            worms: w.inv.bag.worms,
            selected: u8::from(w.inv.bag.selected == Consumable::Worm),
            root_hearts: w.inv.root_hearts.max(0) as u32,
            heart_sold: w.adv.heart_sold,
            sp_milli: (w.player.sp.max(0.0) * 1000.0) as u32,
            momo_fight: w.story.fight(crate::enemy::Kind::Momo),
            doto_fight: w.story.fight(crate::enemy::Kind::Doto),
            kongi_fight: w.story.fight(crate::enemy::Kind::Kongi),
            no_damage: w.no_damage,
            stats: stat_fields(w)
                .iter()
                .map(|(k, n)| (k.to_string(), *n))
                .collect(),
            run_secs: self.run_time.max(0.0) as u32,
            falls: w.adv.falls,
            slots: self
                .inventory
                .slots
                .iter()
                .map(|s| match s {
                    Some(Item::Weapon(x)) => Some(Slot::Weapon(*x)),
                    Some(Item::Charm(x)) => Some(Slot::Charm(*x)),
                    None => None,
                })
                .collect(),
            equipped: self.inventory.equipped,
            charms: w.inv.charms.clone(),
            charms_found: w.inv.charms_found.clone(),
        }
    }

    /// 쉼터에 있으면 지금 런을 적는다(첫 시작 런만, 0080). 굴 안이면 마지막 쉼터 것을 그대로 둔다.
    pub(super) fn checkpoint_run(&mut self) {
        let w = &self.world;
        if !self.recording() || w.adv.place != Place::Camp || !w.player.is_alive() {
            return;
        }
        let r = self.capture_run();
        profile::save_run(profile::slot(), Some(&r.to_profile()));
    }

    /// 런이 끝났다(새 런·마을로 돌아감·가장 깊은 곳): 지금 슬롯의 이어하기를 지운다.
    pub(super) fn clear_run(&mut self) {
        profile::save_run(profile::slot(), None);
    }

    /// 슬롯 `i`의 이어할 런. 튜토리얼 전 기록에는 없다(첫 쉼터 전에 튜토리얼이 끝난다).
    pub(super) fn slot_run(&self, i: usize) -> Option<RunSave> {
        profile::load_run(i)
            .and_then(|p| RunSave::from_profile(&p))
            .filter(|_| {
                let p = if i == profile::slot() {
                    &self.profile
                } else {
                    &self.slot_cache[i]
                };
                p.flag(profile::key::TUTORIAL)
            })
    }

    /// 적어 둔 쉼터로 돌아간다. 새 런처럼 기록에서 정하는 것(재주·실뜨기·장신구 풀·털빛)은 `start`가 하고,
    /// 그 위에 이번 런의 값(쉼터·시드·재화·가방·장비·통계)을 덮는다. 쉼터 도착 장면은 다시 틀지 않는다.
    pub(super) fn resume_run(&mut self, r: RunSave) {
        let ch = Character::from_code(r.character);
        self.profile
            .set_count(profile::key::CHAR_PICK, r.character as u32);
        self.knot_level = r.knots;
        self.resuming = true;
        self.start(Scenario::Normal);
        self.resuming = false;
        let w = &mut self.world;
        w.character = ch;
        w.player.kit = ch;
        w.resume_camp(r.stage, r.seed, r.depth);
        w.adv.events.clear();
        w.inv.seeds = r.seeds;
        w.inv.glowbugs = r.glowbugs;
        w.inv.bag.berries = r.berries;
        w.inv.bag.worms = r.worms;
        w.inv.bag.selected = if r.selected == 1 {
            Consumable::Worm
        } else {
            Consumable::Berry
        };
        w.inv.root_hearts = r.root_hearts as i32;
        w.adv.heart_sold = r.heart_sold;
        w.inv.charms = r.charms.clone();
        for c in &r.charms_found {
            if !w.inv.charms_found.contains(c) {
                w.inv.charms_found.push(*c);
            }
        }
        w.story.set_fight(crate::enemy::Kind::Momo, r.momo_fight);
        w.story.set_fight(crate::enemy::Kind::Doto, r.doto_fight);
        w.story.set_fight(crate::enemy::Kind::Kongi, r.kongi_fight);
        w.no_damage = r.no_damage;
        for (k, n) in &r.stats {
            set_stat(w, k, *n);
        }
        w.refresh_stats();
        w.player.hp = w.player.stats.max_hp;
        w.player.sp = r.sp_milli as f32 / 1000.0;
        let mut inv = Inventory::new(w.character, &[]);
        for (slot, s) in inv.slots.iter_mut().zip(&r.slots) {
            *slot = s.map(|s| match s {
                Slot::Weapon(x) => Item::Weapon(x),
                Slot::Charm(x) => Item::Charm(x),
            });
        }
        inv.equipped = r.equipped.filter(|&e| inv.has(e));
        self.inventory = inv;
        self.apply_equipped();
        self.run_time = r.run_secs as f32;
        self.world.adv.falls = r.falls;
        self.tally = Tally::of(&self.world);
        self.cutscene = None;
        self.tutorial = None;
        self.scene = Scene::Game;
    }
}
