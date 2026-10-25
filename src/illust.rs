//! 컷씬 일러스트(docs/decisions/0039). 화면 전체를 덮는 한 장 그림 + 느린 패닝 + 떠다니는 입자.
//!
//! - 그림은 640×360(게임 픽셀 320×180의 2배 밀도, 초상화와 같은 방식). 패닝하는 그림은 그보다 크다.
//! - 지금 그림은 `tools/art/illust.py`가 만든 임시 그림이다. 사람이 그린 그림으로 바꿀 때는
//!   같은 이름의 PNG를 `assets/art/illust/`에 넣으면 된다(사양: docs/art/cutscene-illustrations.md).
//! - 대화창이 아래 1/3을 덮으므로 중요한 것은 위 2/3에 둔다.

use macroquad::prelude::*;

/// 화면에 보이는 원본 크기(px). 게임 화면 320×180의 2배.
pub const VIEW_W: f32 = 640.0;
pub const VIEW_H: f32 = 360.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Illust {
    /// 오프닝: 잎이 거의 없는 큰 나무 → 땅속 뿌리와 보라 실(아래로 패닝).
    GreatTree,
    /// 오프닝: 어미가 Naïl을 굴 밖으로 민다.
    MotherBurrow,
    /// 보리에게서 보라 실이 풀린다.
    BoriFreed,
    /// 모모의 귀 밑동 꽃이 핀다.
    MomoBloom,
    /// 반쪽 엔딩: 가장 깊은 곳의 누룩 할매.
    HalmaeDeep,
    /// 참 엔딩: 큰 나무의 마지막 꽃.
    LastBloom,
    /// 참 엔딩: 흰 실이 새싹을 감싼다.
    Sprout,
    /// 에필로그: 새싹 아래 마을과 새 두더지 둔덕.
    Molehill,
    /// 도토가 기억해 낸다: 아주 오래전, 흰 실 곁에 씨앗을 묻던 날(세피아, 0053).
    DotoMemory,
}

/// 그림 위에 떠다니는 것.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fx {
    /// 마른 잎이 떨어진다.
    Leaves,
    /// 굴 안 먼지(빛 속에서 천천히).
    Dust,
    /// 풀려난 실이 반짝이며 올라간다.
    Sparkles,
    /// 꽃잎이 떨어진다.
    Petals,
    /// 포자가 빛나며 떠오른다.
    Spores,
}

pub struct Spec {
    /// `assets/art/illust/<file>.png`
    pub file: &'static str,
    /// 패닝: 원본 px 기준 보이는 창의 왼쪽 위(시작, 끝)와 걸리는 시간.
    pub pan: (Vec2, Vec2),
    pub pan_secs: f32,
    pub fx: Fx,
}

impl Illust {
    pub const ALL: [Illust; 9] = [
        Illust::GreatTree,
        Illust::MotherBurrow,
        Illust::BoriFreed,
        Illust::MomoBloom,
        Illust::HalmaeDeep,
        Illust::LastBloom,
        Illust::Sprout,
        Illust::Molehill,
        Illust::DotoMemory,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|i| *i == self).unwrap_or(0)
    }

    pub fn spec(self) -> Spec {
        let still = (Vec2::ZERO, Vec2::ZERO);
        let (file, pan, pan_secs, fx) = match self {
            // 640×560: 하늘·나무에서 시작해 땅속 뿌리로 내려간다.
            Illust::GreatTree => (
                "great_tree",
                (Vec2::ZERO, vec2(0.0, 200.0)),
                9.0,
                Fx::Leaves,
            ),
            Illust::MotherBurrow => ("mother_burrow", still, 1.0, Fx::Dust),
            Illust::BoriFreed => ("bori_freed", still, 1.0, Fx::Sparkles),
            Illust::MomoBloom => ("momo_bloom", still, 1.0, Fx::Petals),
            // 704×360: 옆으로 천천히, 실이 모인 한가운데로.
            Illust::HalmaeDeep => (
                "halmae_deep",
                (Vec2::ZERO, vec2(64.0, 0.0)),
                12.0,
                Fx::Spores,
            ),
            // 640×440: 꽃 핀 가지에서 밑동으로.
            Illust::LastBloom => (
                "last_bloom",
                (Vec2::ZERO, vec2(0.0, 80.0)),
                10.0,
                Fx::Petals,
            ),
            Illust::Sprout => ("sprout", still, 1.0, Fx::Spores),
            Illust::Molehill => ("molehill", still, 1.0, Fx::Petals),
            Illust::DotoMemory => ("doto_memory", still, 1.0, Fx::Dust),
        };
        Spec {
            file,
            pan,
            pan_secs,
            fx,
        }
    }
}

/// 보이는 창의 왼쪽 위(원본 px, 정수). 부드럽게 출발해 부드럽게 멈춘다.
pub fn pan_at(spec: &Spec, t: f32) -> Vec2 {
    let k = (t / spec.pan_secs.max(0.01)).clamp(0.0, 1.0);
    let k = k * k * (3.0 - 2.0 * k);
    (spec.pan.0 + (spec.pan.1 - spec.pan.0) * k).round()
}

/// 입자 하나: 원본 px 위치(보이는 창 기준), 크기(원본 px), 색.
pub struct Particle {
    pub pos: Vec2,
    pub size: f32,
    pub color: Color,
}

fn hash(i: u32, salt: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

/// 시간 `t`의 입자들. 상태 없이 시간만으로 정한다(건너뛰기·되감기에도 같은 그림).
pub fn particles(fx: Fx, t: f32) -> Vec<Particle> {
    let (n, fall, sway, size, color) = match fx {
        Fx::Leaves => (14, 26.0, 18.0, 4.0, Color::new(0.62, 0.42, 0.24, 1.0)),
        Fx::Dust => (24, -4.0, 6.0, 2.0, Color::new(1.0, 0.92, 0.70, 0.55)),
        Fx::Sparkles => (22, -22.0, 8.0, 2.0, Color::new(0.86, 0.72, 1.0, 0.9)),
        Fx::Petals => (20, 22.0, 22.0, 4.0, Color::new(1.0, 0.80, 0.86, 1.0)),
        Fx::Spores => (26, -10.0, 10.0, 2.0, Color::new(0.88, 1.0, 0.86, 0.8)),
    };
    (0..n)
        .map(|i| {
            let speed = 0.6 + hash(i, 1) * 0.8;
            let x0 = hash(i, 2) * VIEW_W;
            let y0 = hash(i, 3) * VIEW_H;
            let ph = hash(i, 4) * std::f32::consts::TAU;
            let y = (y0 + fall * speed * t).rem_euclid(VIEW_H + 8.0) - 4.0;
            let x = (x0 + (t * 0.8 + ph).sin() * sway).rem_euclid(VIEW_W);
            // 깜빡임: 반짝이·포자·먼지는 숨 쉬듯.
            let a = match fx {
                Fx::Sparkles | Fx::Spores | Fx::Dust => {
                    color.a * (0.55 + 0.45 * (t * 2.0 + ph).sin())
                }
                _ => color.a,
            };
            Particle {
                pos: vec2(x.round(), y.round()),
                size: if hash(i, 5) > 0.7 { size } else { size / 2.0 },
                color: Color { a, ..color },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pan_starts_and_ends_on_whole_pixels() {
        let s = Illust::GreatTree.spec();
        assert_eq!(pan_at(&s, 0.0), s.pan.0);
        assert_eq!(pan_at(&s, s.pan_secs + 5.0), s.pan.1);
        let mid = pan_at(&s, s.pan_secs * 0.37);
        assert_eq!(mid, mid.round());
        assert!(mid.y > s.pan.0.y && mid.y < s.pan.1.y);
    }

    #[test]
    fn every_illustration_has_its_own_file_and_index() {
        for (i, il) in Illust::ALL.iter().enumerate() {
            assert_eq!(il.index(), i);
            let f = il.spec().file;
            assert!(Illust::ALL[..i].iter().all(|o| o.spec().file != f));
        }
    }

    #[test]
    fn particles_stay_on_the_picture() {
        for fx in [Fx::Leaves, Fx::Dust, Fx::Sparkles, Fx::Petals, Fx::Spores] {
            for t in [0.0, 1.3, 47.9] {
                for p in particles(fx, t) {
                    assert!(p.pos.x >= 0.0 && p.pos.x <= VIEW_W);
                    assert!(p.pos.y >= -4.0 && p.pos.y <= VIEW_H + 4.0);
                }
            }
        }
    }
}
