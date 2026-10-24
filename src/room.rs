//! 방 하나. 경계 벽 + 기둥 몇 개, 원 vs 사각형 충돌.

use macroquad::prelude::*;

use crate::tuning;

pub struct Room {
    /// 플레이 가능한 안쪽 영역(벽 안쪽).
    pub inner: Rect,
    pub pillars: Vec<Rect>,
    pub spawn_points: Vec<Vec2>,
}

impl Room {
    pub fn new() -> Self {
        let w = tuning::WALL;
        Self {
            inner: Rect::new(w, w, tuning::VIEW_W - 2.0 * w, tuning::VIEW_H - 2.0 * w),
            pillars: vec![
                Rect::new(104.0, 56.0, 16.0, 16.0),
                Rect::new(200.0, 108.0, 16.0, 16.0),
            ],
            spawn_points: vec![
                vec2(40.0, 40.0),
                vec2(280.0, 40.0),
                vec2(40.0, 140.0),
                vec2(280.0, 140.0),
                vec2(160.0, 28.0),
                vec2(160.0, 152.0),
            ],
        }
    }

    pub fn center(&self) -> Vec2 {
        self.inner.center()
    }

    /// 원을 벽·기둥 밖으로 밀어낸다. 무언가에 닿았으면 true.
    pub fn resolve_circle(&self, pos: &mut Vec2, r: f32) -> bool {
        let mut hit = false;
        let clamped = vec2(
            pos.x.clamp(self.inner.x + r, self.inner.right() - r),
            pos.y.clamp(self.inner.y + r, self.inner.bottom() - r),
        );
        if clamped != *pos {
            *pos = clamped;
            hit = true;
        }
        for p in &self.pillars {
            hit |= push_out_rect(pos, r, *p);
        }
        hit
    }

    /// 점이 벽/기둥 안에 있는가(투사체용).
    pub fn is_solid(&self, p: Vec2, r: f32) -> bool {
        if p.x - r < self.inner.x
            || p.x + r > self.inner.right()
            || p.y - r < self.inner.y
            || p.y + r > self.inner.bottom()
        {
            return true;
        }
        self.pillars.iter().any(|rect| {
            let c = vec2(
                p.x.clamp(rect.x, rect.right()),
                p.y.clamp(rect.y, rect.bottom()),
            );
            (p - c).length_squared() < r * r
        })
    }
}

/// 원을 사각형 밖으로 민다. 겹쳤으면 true.
fn push_out_rect(pos: &mut Vec2, r: f32, rect: Rect) -> bool {
    let closest = vec2(
        pos.x.clamp(rect.x, rect.right()),
        pos.y.clamp(rect.y, rect.bottom()),
    );
    let d = *pos - closest;
    let dist2 = d.length_squared();
    if dist2 >= r * r {
        return false;
    }
    if dist2 > 1e-6 {
        let dist = dist2.sqrt();
        *pos += d / dist * (r - dist);
    } else {
        // 중심이 사각형 안: 가장 얕은 축으로 밀어낸다.
        let left = pos.x - rect.x;
        let right = rect.right() - pos.x;
        let top = pos.y - rect.y;
        let bottom = rect.bottom() - pos.y;
        let m = left.min(right).min(top).min(bottom);
        if m == left {
            pos.x = rect.x - r;
        } else if m == right {
            pos.x = rect.right() + r;
        } else if m == top {
            pos.y = rect.y - r;
        } else {
            pos.y = rect.bottom() + r;
        }
    }
    true
}
