//! 가방의 퀵 아이템(docs/decisions/0017). 바닥에서 주우면 여기로 들어오고, 퀵 버튼으로 먹는다.
//!
//! - 산딸기: 체력 1. 최대 BERRY_MAX개.
//! - 쥬시 지렁이: 먹으면 체력 가득. 들고 있다가 쓰러지면 저절로 먹고 일어난다(목숨). 최대 WORM_MAX개 → 최대 목숨 1 + WORM_MAX.
//!
//! 고르기(퀵 슬롯 넘기기)와 먹기는 버튼 두 개. 하나뿐이면 넘기기는 아무 일도 안 한다.
//! 먹기는 바로 낫지 않는다: 먹는 시간(맞으면 끊김) + 다음 먹기까지 대기(docs/decisions/0038).

use crate::tuning as t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consumable {
    Berry,
    Worm,
}

impl Consumable {
    pub const ALL: [Consumable; 2] = [Consumable::Berry, Consumable::Worm];

    /// 먹는 데 걸리는 시간(docs/decisions/0038).
    pub fn eat_time(self) -> f32 {
        match self {
            Consumable::Berry => t::BERRY_EAT_TIME,
            Consumable::Worm => t::WORM_EAT_TIME,
        }
    }

    pub fn max(self) -> u8 {
        match self {
            Consumable::Berry => t::BERRY_MAX,
            Consumable::Worm => t::WORM_MAX,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bag {
    pub berries: u8,
    pub worms: u8,
    /// 퀵 슬롯에 올라와 있는 것.
    pub selected: Consumable,
}

impl Default for Bag {
    fn default() -> Self {
        Self {
            berries: 0,
            worms: 0,
            selected: Consumable::Berry,
        }
    }
}

impl Bag {
    pub fn count(&self, c: Consumable) -> u8 {
        match c {
            Consumable::Berry => self.berries,
            Consumable::Worm => self.worms,
        }
    }

    fn count_mut(&mut self, c: Consumable) -> &mut u8 {
        match c {
            Consumable::Berry => &mut self.berries,
            Consumable::Worm => &mut self.worms,
        }
    }

    /// 넣을 자리가 있나.
    pub fn has_room(&self, c: Consumable) -> bool {
        self.count(c) < c.max()
    }

    /// 하나 넣는다. 가득이면 false. 지금 슬롯이 비어 있으면 넣은 것으로 바꾼다.
    pub fn add(&mut self, c: Consumable) -> bool {
        if !self.has_room(c) {
            return false;
        }
        *self.count_mut(c) += 1;
        if self.count(self.selected) == 0 {
            self.selected = c;
        }
        true
    }

    /// 하나 꺼낸다. 없으면 false.
    pub fn take(&mut self, c: Consumable) -> bool {
        let n = self.count_mut(c);
        if *n == 0 {
            return false;
        }
        *n -= 1;
        true
    }

    /// 퀵 슬롯을 다음 "가진 것"으로. 아무것도 없으면 그대로.
    /// 다음(`forward`) / 앞 칸으로. 비어 있는 칸은 건너뛴다.
    pub fn cycle(&mut self, forward: bool) {
        let all = Consumable::ALL;
        let n = all.len();
        let i = all.iter().position(|c| *c == self.selected).unwrap_or(0);
        for k in 1..=n {
            let c = if forward {
                all[(i + k) % n]
            } else {
                all[(i + n * 2 - k) % n]
            };
            if self.count(c) > 0 {
                self.selected = c;
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacks_are_capped_and_slot_follows_what_you_have() {
        let mut b = Bag::default();
        assert!(b.add(Consumable::Worm));
        assert_eq!(b.selected, Consumable::Worm);
        for _ in 0..10 {
            b.add(Consumable::Berry);
        }
        assert_eq!(b.berries, t::BERRY_MAX);
        assert!(!b.add(Consumable::Berry));
        b.cycle(true);
        assert_eq!(b.selected, Consumable::Berry);
        assert!(b.take(Consumable::Worm));
        assert!(!b.take(Consumable::Worm));
        b.cycle(true);
        assert_eq!(b.selected, Consumable::Berry);
    }
}
