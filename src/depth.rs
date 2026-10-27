//! 장소마다 "지하 몇 m"(0156). 게임 수치가 아니라 연출이다: 내려가는 전환의 지층 단면과 지대 이름 카드가 쓴다.
//! 땅 위(필드·마을) 0m → 쉼터 1·2·3 → 가장 깊은 곳. 길 칸은 그 길의 앞뒤 쉼터 사이를 층 비율로 나눈다.

use crate::tuning as t;

/// 쉼터 깊이(인덱스 = 몇 번째 쉼터, 0 = 땅 위).
const CAMP_M: [u32; 4] = [0, 12, 48, 90];
/// 마지막 길의 끝(가장 깊은 곳).
pub const DEEPEST_M: u32 = 130;

/// 몇 번째 쉼터의 깊이.
pub fn camp(stage: u32) -> u32 {
    CAMP_M.get(stage as usize).copied().unwrap_or(DEEPEST_M)
}

/// 이 길이 끝나는 깊이(다음 쉼터, 마지막 길이면 가장 깊은 곳).
pub fn stage_end(stage: u32) -> u32 {
    if stage >= t::FINAL_STAGE {
        DEEPEST_M
    } else {
        camp(stage + 1)
    }
}

/// 길(stage)의 층 `layer`(0부터, 전체 `layers`층, 마지막 = 보스) 굴의 깊이. 앞 쉼터와 끝 사이를 고르게 나눈다.
pub fn node(stage: u32, layer: usize, layers: usize) -> u32 {
    let (a, b) = (camp(stage), stage_end(stage));
    a + (b - a) * (layer as u32 + 1) / (layers as u32 + 1)
}

/// 길 지도 없이 필드에서 들어간 예전 굴(깊이 단계마다).
pub fn classic(depth: u32) -> u32 {
    6 + 20 * depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deeper_along_the_run() {
        // 쉼터 → 길의 층들 → 다음 쉼터 → … → 가장 깊은 곳이 늘 깊어진다.
        let layers = crate::route::CHOICE_LAYERS + 1;
        let mut last = 0;
        for stage in 1..=t::FINAL_STAGE {
            assert!(camp(stage) > last, "쉼터 {stage}");
            last = camp(stage);
            for l in 0..layers {
                let d = node(stage, l, layers);
                assert!(d > last, "길 {stage} 층 {l}");
                last = d;
            }
            assert!(stage_end(stage) > last);
        }
        assert_eq!(stage_end(t::FINAL_STAGE), DEEPEST_M);
    }
}
