//! 模块 1-2：基础数值类型 + 仿射变换运算。
//!
//! 公式来源：docs 04-数学模型-变换与蒙皮-实现 §1~4。
//! 设计：纯函数，无全局状态；f32 内部。

use serde::{Deserialize, Serialize};

// ============================================================================
// 模块 1：基础数值类型
// ============================================================================

/// 2D 向量。
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }

    #[inline]
    pub fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }

    #[inline]
    pub fn scale(self, s: f32) -> Self {
        Self { x: self.x * s, y: self.y * s }
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// 归一化；零向量返回零向量（不产生 NaN）。
    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 { self.scale(1.0 / len) } else { Self::ZERO }
    }
}

/// 6 分量仿射变换（2×3 省略末行 [0,0,1]）。
///
/// 变换一个点：`x' = a·x + c·y + wx`，`y' = b·x + d·y + wy`。
///
/// 为什么 6 分量而非 3×3：见 docs 04-数学模型-变换与蒙皮-原理 §4.1（末行恒为 [0,0,1]，存了浪费、算了浪费）。
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Affine {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub wx: f32,
    pub wy: f32,
}

impl Affine {
    /// 单位阵。
    pub const IDENTITY: Self = Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, wx: 0.0, wy: 0.0 };

    #[inline]
    pub fn transform_point(self, p: Vec2) -> Vec2 {
        Vec2::new(
            self.a * p.x + self.c * p.y + self.wx,
            self.b * p.x + self.d * p.y + self.wy,
        )
    }
}

/// 骨骼 local 自由度（7 标量）。
///
/// rotation 内部用**弧度**（见 docs 04-原理 §4.3）。
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct BoneLocal {
    pub x: f32,
    pub y: f32,
    /// 旋转（弧度）。
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub shear_x: f32,
    /// 2D 下通常恒为 0，保留以对称。
    pub shear_y: f32,
}

impl BoneLocal {
    /// setup pose 常用默认值（scale=1）。
    pub const DEFAULT: Self = Self {
        x: 0.0, y: 0.0, rotation: 0.0,
        scale_x: 1.0, scale_y: 1.0, shear_x: 0.0, shear_y: 0.0,
    };
}

// ============================================================================
// 模块 2：仿射变换运算（纯函数）
// ============================================================================

/// BoneLocal → Affine（含 shearX 推导）。
///
/// `M_local = Translate · Rotate · Scale · Shear`。
/// 公式见 docs 04-实现 §1。
pub fn local_to_affine(l: &BoneLocal) -> Affine {
    let cos = l.rotation.cos();
    let sin = l.rotation.sin();

    let la = cos * l.scale_x;
    let lb = sin * l.scale_x;
    let lc = -sin * l.scale_y;
    let ld = cos * l.scale_y;

    // 叠加 shearX：把局部 Y 轴额外转 shearX（仅需 sin 项 sb 扰动 a/b）。
    let sb = l.shear_x.sin();

    Affine {
        a: la + lc * sb,
        b: lb + ld * sb,
        c: lc, // 注意：shearX 不改 c/d 本身，只通过 sb 扰动 a/b
        d: ld,
        wx: l.x,
        wy: l.y,
    }
}

/// 仿射乘法 `P · L`（父变换作用于子局部）。
///
/// 公式见 docs 04-实现 §2。
pub fn multiply(p: &Affine, l: &Affine) -> Affine {
    Affine {
        a: p.a * l.a + p.c * l.b,
        b: p.b * l.a + p.d * l.b,
        c: p.a * l.c + p.c * l.d,
        d: p.b * l.c + p.d * l.d,
        wx: p.a * l.wx + p.c * l.wy + p.wx,
        wy: p.b * l.wx + p.d * l.wy + p.wy,
    }
}

/// 仿射矩阵求逆（6 分量）。
///
/// 公式见 docs 04-实现 §4。
///
/// # Panic
/// 行列式为 0 时 panic（退化的骨骼配置，数据错误）。
pub fn invert(m: &Affine) -> Affine {
    let det = m.a * m.d - m.b * m.c;
    assert!(det != 0.0, "invert: 行列式为 0（退化骨骼配置）");
    let inv_det = 1.0 / det;
    Affine {
        a: m.d * inv_det,
        b: -m.b * inv_det,
        c: -m.c * inv_det,
        d: m.a * inv_det,
        wx: (m.c * m.wy - m.d * m.wx) * inv_det,
        wy: (m.b * m.wx - m.a * m.wy) * inv_det,
    }
}

/// 取世界旋转角（弧度）。
pub fn world_rotation(w: &Affine) -> f32 {
    w.b.atan2(w.a)
}

/// 取世界缩放（X、Y 分离）。注意负缩放会反映在符号上。
pub fn world_scale(w: &Affine) -> (f32, f32) {
    let sx = (w.a * w.a + w.b * w.b).sqrt();
    let sy = (w.c * w.c + w.d * w.d).sqrt();
    (sx, sy)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn vec2_ops() {
        let a = Vec2::new(3.0, 4.0);
        assert!(approx(a.length(), 5.0));
        assert_eq!(a.dot(Vec2::new(1.0, 0.0)), 3.0);
        assert!(approx(a.normalize().length(), 1.0));
        assert_eq!(Vec2::ZERO.normalize(), Vec2::ZERO); // 零向量不 NaN
    }

    #[test]
    fn identity_invariants() {
        assert_eq!(multiply(&Affine::IDENTITY, &Affine::IDENTITY), Affine::IDENTITY);
        assert_eq!(invert(&Affine::IDENTITY), Affine::IDENTITY);
        // multiply(I, x) == x
        let x = Affine { a: 2.0, b: 0.5, c: -1.0, d: 3.0, wx: 4.0, wy: -2.0 };
        assert_eq!(multiply(&Affine::IDENTITY, &x), x);
        assert_eq!(multiply(&x, &Affine::IDENTITY), x);
    }

    #[test]
    fn invert_inverse_of_multiply() {
        // invert(multiply(a,b)) ≈ multiply(invert(b), invert(a))
        let a = Affine { a: 2.0, b: 0.3, c: -0.5, d: 1.5, wx: 4.0, wy: -2.0 };
        let b = Affine { a: 1.2, b: -0.4, c: 0.6, d: 0.8, wx: 1.0, wy: 2.0 };
        let ab = multiply(&a, &b);
        let inv_ab = invert(&ab);
        let expected = multiply(&invert(&b), &invert(&a));
        assert!(approx(inv_ab.a, expected.a));
        assert!(approx(inv_ab.b, expected.b));
        assert!(approx(inv_ab.c, expected.c));
        assert!(approx(inv_ab.d, expected.d));
        assert!(approx(inv_ab.wx, expected.wx));
        assert!(approx(inv_ab.wy, expected.wy));
    }

    #[test]
    fn local_to_affine_identity() {
        // DEFAULT 应得单位阵
        let m = local_to_affine(&BoneLocal::DEFAULT);
        assert!(approx(m.a, 1.0) && approx(m.d, 1.0));
        assert!(approx(m.b, 0.0) && approx(m.c, 0.0));
        assert!(approx(m.wx, 0.0) && approx(m.wy, 0.0));
    }

    #[test]
    fn local_to_affine_pure_translate() {
        let m = local_to_affine(&BoneLocal { x: 5.0, y: -3.0, ..BoneLocal::DEFAULT });
        assert!(approx(m.wx, 5.0) && approx(m.wy, -3.0));
    }

    #[test]
    fn local_to_affine_rotation_90() {
        // 旋转 90° 应把 (1,0) 变成 (0,1)
        let l = BoneLocal { rotation: std::f32::consts::FRAC_PI_2, ..BoneLocal::DEFAULT };
        let m = local_to_affine(&l);
        let p = m.transform_point(Vec2::new(1.0, 0.0));
        assert!(approx(p.x.abs(), 0.0)); // cos90≈0
        assert!(approx(p.y, 1.0));
    }

    #[test]
    fn local_to_affine_shear() {
        // shearX 应扰动 b（使局部 Y 轴倾斜）
        let l = BoneLocal { shear_x: std::f32::consts::FRAC_PI_4, ..BoneLocal::DEFAULT };
        let m = local_to_affine(&l);
        // shear 不为 0 时 b 应非 0
        assert!(m.b.abs() > 0.01, "shearX 应使 b 非零, got b={}", m.b);
    }

    #[test]
    fn world_rotation_and_scale() {
        let l = BoneLocal { rotation: std::f32::consts::FRAC_PI_4, scale_x: 2.0, scale_y: 3.0, ..BoneLocal::DEFAULT };
        let w = local_to_affine(&l);
        assert!(approx(world_rotation(&w), std::f32::consts::FRAC_PI_4));
        let (sx, sy) = world_scale(&w);
        assert!(approx(sx, 2.0));
        assert!(approx(sy, 3.0));
    }
}
