//! cubic bezier 插值曲线（docs 16-P2动画 §模块2，原理见 05d §5.6）。
//!
//! 关键帧之间的过渡曲线。控制点 (cx1,cy1,cx2,cy2) 归一化到 [0,1]。
//! 给定归一化进度 u ∈ [0,1]，求插值结果 y。
//!
//! 难点：bezier 的 x(t) 与 t 非线性，要由"目标 x"反解 t 再求 y。

use serde::{Deserialize, Serialize};

/// 插值曲线类型。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CurveKind {
    /// 线性插值（等价 bezier 控制点 (0,0)(1,1)）。
    Linear,
    /// 阶跃：保持上一帧值，不插值（用于离散切换）。
    Stepped,
    /// cubic bezier，控制点 (cx1,cy1,cx2,cy2) 归一化 [0,1]。
    Bezier(f32, f32, f32, f32),
}

/// 到下一关键帧的插值曲线。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Curve {
    pub kind: CurveKind,
}

impl Curve {
    pub const LINEAR: Self = Self { kind: CurveKind::Linear };
    pub const STEPPED: Self = Self { kind: CurveKind::Stepped };

    /// 给定归一化进度 u ∈ [0,1] 与起止值，求插值结果。
    ///
    /// - Linear: 线性插值
    /// - Stepped: 返回 from（保持上一帧）
    /// - Bezier: 由 u 反解 t，再求 y(t) 插值
    pub fn sample(&self, u: f32, from: f32, to: f32) -> f32 {
        let u = u.clamp(0.0, 1.0);
        match &self.kind {
            CurveKind::Linear => from + (to - from) * u,
            CurveKind::Stepped => from,
            CurveKind::Bezier(cx1, cy1, cx2, cy2) => {
                let t = solve_bezier_t(*cx1, *cx2, u);
                let y = bezier_component(t, *cy1, *cy2);
                from + (to - from) * y
            }
        }
    }
}

/// bezier 单分量求值。
/// 标准三次 bezier（端点固定 0 和 1）：
/// `B(t) = 3(1-t)²t·c1 + 3(1-t)t²·c2`
fn bezier_component(t: f32, c1: f32, c2: f32) -> f32 {
    let one_t = 1.0 - t;
    3.0 * one_t * one_t * t * c1 + 3.0 * one_t * t * t * c2 + t * t * t
}

/// 由目标 x 反解 t：解 bezier 的 x(t) = target。
///
/// x(t) = 3(1-t)²t·cx1 + 3(1-t)t²·cx2（端点 0 和 1）。
/// 用二分法（足够稳定，避免牛顿的导数退化问题）。
fn solve_bezier_t(cx1: f32, cx2: f32, target_x: f32) -> f32 {
    let target_x = target_x.clamp(0.0, 1.0);
    let mut lo = 0.0_f32;
    let mut hi = 1.0_f32;
    // 二分 30 次足够 f32 精度
    for _ in 0..30 {
        let mid = (lo + hi) * 0.5;
        let x = bezier_component(mid, cx1, cx2);
        if x < target_x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn linear_interp() {
        let c = Curve::LINEAR;
        assert!(approx(c.sample(0.0, 10.0, 20.0), 10.0));
        assert!(approx(c.sample(0.5, 10.0, 20.0), 15.0));
        assert!(approx(c.sample(1.0, 10.0, 20.0), 20.0));
    }

    #[test]
    fn stepped_holds_from() {
        let c = Curve::STEPPED;
        // 阶跃：任何进度都返回 from
        assert!(approx(c.sample(0.0, 10.0, 20.0), 10.0));
        assert!(approx(c.sample(0.5, 10.0, 20.0), 10.0));
        assert!(approx(c.sample(0.99, 10.0, 20.0), 10.0));
    }

    #[test]
    fn bezier_endpoints() {
        // bezier 控制点 (0.3,0.5,0.7,0.5)（对称 ease-in-out）
        let c = Curve { kind: CurveKind::Bezier(0.3, 0.5, 0.7, 0.5) };
        // 端点：u=0 → from，u=1 → to
        assert!(approx(c.sample(0.0, 0.0, 100.0), 0.0));
        assert!(approx(c.sample(1.0, 0.0, 100.0), 100.0));
    }

    #[test]
    fn bezier_midpoint_symmetric() {
        // 对称曲线（控制点关于 (0.5,0.5) 对称），中点 y 应≈0.5
        let c = Curve { kind: CurveKind::Bezier(0.25, 0.75, 0.75, 0.25) };
        let y = c.sample(0.5, 0.0, 100.0);
        // 对称曲线中点不一定严格 0.5，但应在合理范围
        assert!(y > 30.0 && y < 70.0, "对称曲线中点 y={y} 应在 30~70");
    }

    #[test]
    fn bezier_linear_equivalent() {
        // bezier 控制点 (1/3, 1/3, 2/3, 2/3) 近似线性
        let c = Curve { kind: CurveKind::Bezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0) };
        let y = c.sample(0.5, 0.0, 100.0);
        assert!(approx(y, 50.0), "近似线性中点应≈50, got {y}");
    }

    #[test]
    fn clamp_out_of_range() {
        let c = Curve::LINEAR;
        // u 超出 [0,1] 应被 clamp
        assert!(approx(c.sample(-0.5, 10.0, 20.0), 10.0));
        assert!(approx(c.sample(1.5, 10.0, 20.0), 20.0));
    }
}
