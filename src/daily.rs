//! 오늘의 굴(docs/decisions/0044): 날짜(UTC)가 시드라 같은 날엔 모두 같은 첫 길·같은 굴을 만난다(같은 길을 고르면).
//! 첫 쉼터에서 시작하고, 기록은 그날 닿은 가장 먼 쉼터와 완주. 서버 없이 혼자 보는 기록부터(0040).

/// 1970-01-01부터 센 UTC 날짜.
pub fn today() -> u32 {
    (macroquad::miniquad::date::now() / 86_400.0)
        .floor()
        .max(0.0) as u32
}

/// 날짜 → 런 시드(splitmix64 한 번). 같은 날 = 같은 시드.
pub fn seed(day: u32) -> u64 {
    let mut z = (day as u64).wrapping_add(0x9E37_79B9_7F4A_7C15) ^ 0xDA11_7000;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// 날짜 → (년, 월, 일). Howard Hinnant의 civil_from_days.
pub fn civil(day: u32) -> (i32, u32, u32) {
    let z = day as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = (yoe + era * 400 + if m <= 2 { 1 } else { 0 }) as i32;
    (y, m, d)
}

/// 기록 키: 그날 닿은 가장 먼 쉼터, 완주.
pub fn key_best(day: u32) -> String {
    format!("daily.{day}.best")
}
pub fn key_clear(day: u32) -> String {
    format!("daily.{day}.clear")
}

/// 오래된 날의 기록인가(기록이 끝없이 늘지 않게 지운다).
pub fn is_stale_key(k: &str, today: u32) -> bool {
    k.strip_prefix("daily.")
        .and_then(|r| r.split('.').next())
        .and_then(|d| d.parse::<u32>().ok())
        .is_some_and(|d| d + 30 < today)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates_are_right() {
        assert_eq!(civil(0), (1970, 1, 1));
        // 2026-10-25 = 20751일째
        assert_eq!(civil(20_751), (2026, 10, 25));
        assert_eq!(civil(11_016), (2000, 2, 29));
    }

    #[test]
    fn same_day_same_seed_and_days_differ() {
        assert_eq!(seed(20_751), seed(20_751));
        assert_ne!(seed(20_751), seed(20_752));
    }

    #[test]
    fn stale_keys_are_older_than_a_month() {
        assert!(is_stale_key("daily.100.best", 200));
        assert!(!is_stale_key("daily.190.best", 200));
        assert!(!is_stale_key("tutorial", 200));
    }
}
