//! 실의 꿈(docs/decisions/0055): 누룩 할매의 실이 기억하는 물든 모습들과 꿈속에서 다시 싸운다(Hollow Knight Godhome 자리).
//!
//! 마을 할매 곁 실 고리(`X`)에서 고른다. 이긴 길의 주인만 열린다. 넷을 다 이기면 넷을 이어서 꾸는 줄이 열린다.
//! 기록은 가장 빠른 시간(0.1초)과 안 맞고 이김(★)만 남는다. 이야기·통계·반딧불은 남기지 않는다(꿈이니까).

use crate::enemy::Kind;
use crate::profile::{key, Profile};

/// 꿈의 길의 주인(나오는 순서 = 이어서 꾸기 순서).
pub const BOSSES: [Kind; 4] = [Kind::Chief, Kind::Momo, Kind::Doto, Kind::Kongi];
/// 목록 줄: 보스 넷 + 이어서 꾸기.
pub const ROWS: usize = BOSSES.len() + 1;
/// 꿈의 모모는 세 번째 모습(한 수씩 다 배운).
pub const MOMO_LEVEL: u8 = 3;
/// 이어서 꾸기에서 보스 사이에 돌아오는 체력.
pub const RUSH_HEAL: i32 = 2;
/// 꿈에서 이긴 서로 다른 길의 주인 수(저널).
pub const KEY_WINS: &str = "dream.wins";

/// 이 줄을 꿀 수 있나(그 보스를 한 번이라도 이겼나, 이어서 꾸기는 넷 다).
pub fn unlocked(p: &Profile, row: usize) -> bool {
    match BOSSES.get(row) {
        Some(Kind::Chief) => p.flag(key::BORI_BACK),
        Some(Kind::Momo) => p.count(key::MOMO_WINS) >= 1,
        Some(Kind::Doto) => p.flag(key::DOTO_MEMORY),
        Some(Kind::Kongi) => p.flag(key::KONGI_BACK),
        Some(_) => false,
        None => (0..BOSSES.len()).all(|r| unlocked(p, r)),
    }
}

/// 이 줄에서 싸울 보스들.
pub fn queue(row: usize) -> Vec<Kind> {
    match BOSSES.get(row) {
        Some(k) => vec![*k],
        None => BOSSES.to_vec(),
    }
}

/// 기록 키(바꾸지 않는다, 0041): 가장 빠른 시간(0.1초, 0 = 아직), 안 맞고 이김.
pub fn key_best(row: usize) -> String {
    match BOSSES.get(row) {
        Some(k) => format!("dream.{k:?}"),
        None => "dream.all".to_string(),
    }
}
pub fn key_clean(row: usize) -> String {
    format!("{}.nohit", key_best(row))
}

/// 이겼다: 더 빠르면 시간을 바꾸고, 안 맞았으면 ★. 더 빨랐으면 true.
pub fn record(p: &mut Profile, row: usize, secs: f32, clean: bool) -> bool {
    let tenths = ((secs * 10.0).round() as u32).max(1);
    let k = key_best(row);
    let old = p.count(&k);
    let better = old == 0 || tenths < old;
    if better {
        p.set_count(&k, tenths);
    }
    if clean {
        p.set(&key_clean(row));
    }
    let wins = (0..BOSSES.len())
        .filter(|r| p.count(&key_best(*r)) > 0)
        .count() as u32;
    p.set_count(KEY_WINS, wins);
    better
}

/// 0.1초 단위 → "1:23.4".
pub fn fmt_time(tenths: u32) -> String {
    format!("{}:{:02}.{}", tenths / 600, (tenths / 10) % 60, tenths % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_open_with_story_and_rush_needs_all() {
        let mut p = Profile::default();
        assert!(!(0..ROWS).any(|r| unlocked(&p, r)));
        p.set(key::BORI_BACK);
        p.add(key::MOMO_WINS, 1);
        assert!(unlocked(&p, 0) && unlocked(&p, 1) && !unlocked(&p, 2));
        p.set(key::DOTO_MEMORY);
        p.set(key::KONGI_BACK);
        assert!(unlocked(&p, BOSSES.len()), "넷 다 이기면 이어서 꾸기");
        assert_eq!(queue(BOSSES.len()).len(), 4);
        assert_eq!(queue(2), vec![Kind::Doto]);
    }

    #[test]
    fn record_keeps_the_fastest_and_counts_bosses() {
        let mut p = Profile::default();
        assert!(record(&mut p, 1, 42.34, false));
        assert_eq!(p.count(&key_best(1)), 423);
        assert!(!record(&mut p, 1, 50.0, true), "느리면 시간은 그대로");
        assert_eq!(p.count(&key_best(1)), 423);
        assert!(p.flag(&key_clean(1)), "안 맞고 이긴 건 남는다");
        assert!(record(&mut p, 1, 30.0, false));
        assert_eq!(p.count(KEY_WINS), 1);
        record(&mut p, 3, 61.0, false);
        assert_eq!(p.count(KEY_WINS), 2);
        record(&mut p, BOSSES.len(), 200.0, false);
        assert_eq!(p.count(KEY_WINS), 2, "이어서 꾸기는 보스 수에 안 센다");
        assert_eq!(fmt_time(845), "1:24.5");
    }
}
