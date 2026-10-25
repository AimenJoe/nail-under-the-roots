//! 길 지도(docs/decisions/0031): 쉼터에서 다음 쉼터까지 고르는 뿌리 길. 실제로는 한 줄로 나아가지만
//! 층마다 2~3갈래 중 하나를 고르고, 갈래는 갈라졌다 다시 만난다(Slay the Spire 지도의 축소판).
//!
//! - 층(layer) `CHOICE_LAYERS`개 + 마지막 보스 한 칸. 층마다 1~3칸(1칸 = 좁은 목, 가끔).
//! - 이웃 층 사이 길은 서로 엇갈리지 않는다(위 칸이 위로, 아래 칸이 아래로). 칸마다 다음 층 1~3칸으로.
//! - 칸 종류가 굴(던전)의 모양을 바꾼다: 굴(보통) · 둥지(셈, 보물) · 저장고(작음, 보물) · 샘(쉼, 싸움 없음) · 보스.
//!   규칙: 첫 층엔 둥지·샘이 없다. 샘 다음은 샘이 아니다. 보스 바로 앞 층엔 샘이 하나는 있다. 한 층이 다 같은 종류는 아니다.
//! - 보이는 것(Hades 문처럼 고르기 전에 보상을 안다): 지금 고를 수 있는 다음 층은 종류가 보이고, 그 너머는 "?",
//!   보스는 처음부터 보인다(갈 곳이 보여야 길이 길로 느껴진다).
//!
//! 같은 시드 = 같은 지도(테스트). 그리기는 app.rs, 굴 만들기는 dungeon::Params.

use crate::biome::Biome;
use crate::dungeon::{Goal, Params, Rng};

/// 고르는 층 수(보스 칸은 빼고).
pub const CHOICE_LAYERS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// 보통 굴: 싸움방 몇 개 + 끝방(무리). 씨앗.
    Burrow,
    /// 둥지: 더 센 무리, 끝에 장신구 상자.
    Nest,
    /// 저장고: 작은 굴, 싸움 하나 + 상자.
    Cache,
    /// 샘: 싸움 없음. 체력을 채우고 산딸기.
    Spring,
    /// 그 길의 보스.
    Boss,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    /// 다음 층에서 이어지는 칸(번호, 오름차순).
    pub next: Vec<usize>,
    pub biome: Biome,
    /// 그릴 때 흔들림(-1..1). 줄 맞춘 격자처럼 안 보이게.
    pub jx: f32,
    pub jy: f32,
}

pub struct Route {
    pub layers: Vec<Vec<Node>>,
    /// 지금 있는(마지막으로 고른) 칸. None = 아직 쉼터.
    pub at: Option<(usize, usize)>,
    /// 몇 번째 길인가(1부터). 깊이·환경을 정한다.
    pub stage: u32,
}

impl Route {
    pub fn generate(seed: u64, stage: u32) -> Self {
        let mut rng = Rng::new(seed ^ 0x5157_0A7E);
        let mut counts: Vec<usize> = Vec::new();
        for l in 0..CHOICE_LAYERS {
            // 좁은 목(1칸)은 첫 층이 아니고, 연달아 오지 않을 때만.
            let narrow = l > 0 && counts.last() != Some(&1) && rng.chance(0.2);
            counts.push(if narrow { 1 } else { 2 + rng.below(2) as usize });
        }
        counts.push(1); // 보스
        let mut layers: Vec<Vec<Node>> = counts
            .iter()
            .map(|&n| {
                (0..n)
                    .map(|_| Node {
                        kind: NodeKind::Burrow,
                        next: Vec::new(),
                        biome: Biome::Roots,
                        jx: rng.unit() * 2.0 - 1.0,
                        jy: rng.unit() * 2.0 - 1.0,
                    })
                    .collect()
            })
            .collect();
        // 길: 위 칸부터 다음 층의 이어진 구간 [lo, hi]. lo·hi가 줄지 않으니 엇갈리지 않는다.
        for l in 0..layers.len() - 1 {
            let (na, nb) = (layers[l].len(), layers[l + 1].len());
            let mut lo = 0;
            for (i, node) in layers[l].iter_mut().enumerate() {
                let last = i == na - 1;
                let mut hi = (lo + rng.below(2) as usize).min(nb - 1);
                // 남은 칸으로 아래쪽을 다 덮을 수 있어야 한다(칸마다 최대 2개 새로 덮는다).
                let left = na - 1 - i;
                if nb - 1 > hi + left * 2 {
                    hi = (nb - 1 - left * 2).min(nb - 1);
                }
                if last {
                    hi = nb - 1;
                }
                node.next = (lo..=hi).collect();
                // 다음 칸은 여기 끝과 겹치거나(합류) 바로 아래에서(갈래) 시작한다.
                lo = if rng.chance(0.4) {
                    hi
                } else {
                    (hi + 1).min(nb - 1)
                };
            }
        }
        // 종류
        let last_choice = CHOICE_LAYERS - 1;
        for l in 0..CHOICE_LAYERS {
            for i in 0..layers[l].len() {
                let parents_spring = l > 0
                    && layers[l - 1]
                        .iter()
                        .any(|p| p.kind == NodeKind::Spring && p.next.contains(&i));
                let r = rng.unit();
                let kind = if l == 0 {
                    if r < 0.7 {
                        NodeKind::Burrow
                    } else {
                        NodeKind::Cache
                    }
                } else if r < 0.42 {
                    NodeKind::Burrow
                } else if r < 0.64 {
                    NodeKind::Nest
                } else if r < 0.8 {
                    NodeKind::Cache
                } else if parents_spring {
                    NodeKind::Burrow
                } else {
                    NodeKind::Spring
                };
                layers[l][i].kind = kind;
            }
        }
        // 보스 앞 층엔 샘이 하나는 있다. 그 부모가 샘이면 부모를 굴로.
        if !layers[last_choice]
            .iter()
            .any(|x| x.kind == NodeKind::Spring)
        {
            let j = rng.below(layers[last_choice].len() as u64) as usize;
            layers[last_choice][j].kind = NodeKind::Spring;
            if last_choice > 0 {
                for p in layers[last_choice - 1].iter_mut() {
                    if p.kind == NodeKind::Spring && p.next.contains(&j) {
                        p.kind = NodeKind::Burrow;
                    }
                }
            }
        }
        // 한 층이 다 같으면 하나를 바꾼다(고르는 의미가 있게). 샘은 만들지 않으니 샘 규칙은 그대로다.
        for (l, layer) in layers.iter_mut().enumerate().take(CHOICE_LAYERS) {
            let n = layer.len();
            if n > 1 && layer.iter().all(|x| x.kind == layer[0].kind) {
                let j = rng.below(n as u64) as usize;
                layer[j].kind = match layer[0].kind {
                    NodeKind::Burrow if l == 0 => NodeKind::Cache,
                    NodeKind::Burrow => NodeKind::Nest,
                    _ => NodeKind::Burrow,
                };
            }
        }
        let boss = layers.len() - 1;
        layers[boss][0].kind = NodeKind::Boss;
        // 환경: 첫 길은 뿌리 지대, 둥지·깊은 길은 어두운 심층이 섞인다.
        for layer in layers.iter_mut() {
            for n in layer.iter_mut() {
                let deep = match n.kind {
                    NodeKind::Nest | NodeKind::Boss => stage >= 1,
                    _ => stage >= 2 && rng.chance(0.5),
                };
                n.biome = if deep { Biome::Deep } else { Biome::Roots };
            }
        }
        Self {
            layers,
            at: None,
            stage,
        }
    }

    /// 마지막 길: 주인을 이기면 쉼터가 아니라 가장 깊은 곳(0042).
    pub fn is_final(&self) -> bool {
        self.stage >= crate::tuning::FINAL_STAGE
    }

    /// 지금 고를 수 있는 칸(층, 번호).
    pub fn choices(&self) -> Vec<(usize, usize)> {
        match self.at {
            None => (0..self.layers[0].len()).map(|i| (0, i)).collect(),
            Some((l, i)) if l + 1 < self.layers.len() => {
                self.layers[l][i].next.iter().map(|&j| (l + 1, j)).collect()
            }
            Some(_) => Vec::new(),
        }
    }

    pub fn pick(&mut self, at: (usize, usize)) -> bool {
        if self.choices().contains(&at) {
            self.at = Some(at);
            true
        } else {
            false
        }
    }

    pub fn node(&self, at: (usize, usize)) -> &Node {
        &self.layers[at.0][at.1]
    }

    /// 종류가 보이는 칸: 지나온 층, 지금 고를 층, 보스.
    pub fn revealed(&self, layer: usize) -> bool {
        let next = self.at.map(|(l, _)| l + 1).unwrap_or(0);
        layer <= next || layer == self.layers.len() - 1
    }

    /// 굴의 깊이(적 섞기): 길이 깊을수록, 같은 길에서도 뒤 층일수록.
    pub fn depth_at(&self, layer: usize) -> u32 {
        (self.stage.saturating_sub(1)) * 2 + layer as u32 / 2
    }

    /// 이 칸의 굴 모양.
    pub fn params(&self, at: (usize, usize)) -> Params {
        let n = self.node(at);
        let depth = self.depth_at(at.0);
        match n.kind {
            NodeKind::Burrow => Params {
                rooms: 4,
                goal: Goal::Pack,
                ..Params::new(depth)
            },
            NodeKind::Nest => Params {
                rooms: 4,
                goal: Goal::Pack,
                tough: 2,
                treasure: true,
                ..Params::new(depth)
            },
            NodeKind::Cache => Params {
                rooms: 3,
                goal: Goal::Treasure,
                ..Params::new(depth)
            },
            NodeKind::Spring => Params {
                rooms: 2,
                goal: Goal::Spring,
                ..Params::new(depth)
            },
            NodeKind::Boss => Params {
                rooms: 4,
                goal: Goal::Boss,
                ..Params::new(depth + 1)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(r: &Route, l: usize) -> Vec<NodeKind> {
        r.layers[l].iter().map(|n| n.kind).collect()
    }

    #[test]
    fn every_node_reachable_and_edges_never_cross() {
        for seed in 1..300 {
            let r = Route::generate(seed, 1);
            assert_eq!(r.layers.len(), CHOICE_LAYERS + 1);
            for l in 0..r.layers.len() - 1 {
                let nb = r.layers[l + 1].len();
                let mut hit = vec![false; nb];
                let mut prev_hi = 0;
                let mut prev_lo = 0;
                for n in &r.layers[l] {
                    assert!(!n.next.is_empty(), "seed {seed}: 막다른 칸");
                    assert!(n.next.len() <= 3, "seed {seed}: 길이 너무 많다");
                    let (lo, hi) = (n.next[0], *n.next.last().unwrap());
                    assert!(lo >= prev_lo && hi >= prev_hi, "seed {seed}: 엇갈림");
                    (prev_lo, prev_hi) = (lo, hi);
                    for &j in &n.next {
                        hit[j] = true;
                    }
                }
                assert!(hit.iter().all(|h| *h), "seed {seed}: 갈 수 없는 칸");
            }
        }
    }

    #[test]
    fn kind_rules_hold() {
        for seed in 1..300 {
            let r = Route::generate(seed, 1);
            assert!(!kinds(&r, 0).contains(&NodeKind::Nest));
            assert!(!kinds(&r, 0).contains(&NodeKind::Spring));
            assert!(kinds(&r, CHOICE_LAYERS - 1).contains(&NodeKind::Spring));
            assert_eq!(kinds(&r, CHOICE_LAYERS), vec![NodeKind::Boss]);
            for l in 0..CHOICE_LAYERS {
                let k = kinds(&r, l);
                if k.len() > 1 {
                    assert!(
                        k.iter().any(|x| *x != k[0]),
                        "seed {seed}: 층 {l}이 다 같다"
                    );
                }
                if l > 0 {
                    for p in &r.layers[l - 1] {
                        for &j in &p.next {
                            assert!(
                                !(p.kind == NodeKind::Spring
                                    && r.layers[l][j].kind == NodeKind::Spring),
                                "seed {seed}: 샘 다음 샘"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn walking_the_route_reaches_the_boss_and_hides_far_layers() {
        let mut r = Route::generate(42, 1);
        assert!(r.revealed(0) && !r.revealed(1) && r.revealed(CHOICE_LAYERS));
        assert!(!r.pick((1, 0)), "건너뛸 수 없다");
        while let Some(&c) = r.choices().first() {
            assert!(r.pick(c));
        }
        assert_eq!(r.node(r.at.unwrap()).kind, NodeKind::Boss);
        let a = Route::generate(9, 2);
        let b = Route::generate(9, 2);
        assert_eq!(
            (0..a.layers.len())
                .map(|l| kinds(&a, l))
                .collect::<Vec<_>>(),
            (0..b.layers.len())
                .map(|l| kinds(&b, l))
                .collect::<Vec<_>>()
        );
    }
}
