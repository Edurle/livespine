//! 模块 4：附件与顶点。
//!
//! P0 只实现 region 附件（矩形贴图引用）。mesh/path/clipping 等类型留待 P1+。
//! 纪律（docs 11-图片与纹理处理）：只存几何 + region 名，**不存像素**。

use crate::math::Vec2;
use serde::{Deserialize, Serialize};

/// 每顶点最大骨骼权重数（P0 默认 4，预留 8 能力）。
pub const MAX_WEIGHTS: usize = 4;

/// 蒙皮顶点。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vertex {
    /// 蒙皮前局部坐标。
    pub local: Vec2,
    /// (骨骼索引, 权重)。长度 ≤ MAX_WEIGHTS，权重和应 ≈ 1。
    pub weights: Vec<(usize, f32)>,
}

impl Vertex {
    /// 单骨全权重快捷构造。
    pub fn single(local: Vec2, bone: usize) -> Self {
        Self { local, weights: vec![(bone, 1.0)] }
    }
}

/// P0 region 附件：用 4 个矩形角顶点表示一张矩形贴图的引用。
///
/// 后续 mesh 化时改为任意三角化顶点，接口不变。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionAttachment {
    pub name: String,
    /// 绑定到哪根骨骼（影响 setup 时顶点坐标的解释）。
    pub bone: usize,
    pub width: f32,
    pub height: f32,
    /// 4 个矩形角顶点（左下、右下、右上、左上），坐标相对 attachment 局部空间。
    pub vertices: Vec<Vertex>,
    /// 是否用 LBS 多骨骼蒙皮（true 时顶点按 weights 加权混合，用于 mesh/布料）。
    /// false（默认）：Unity 式父子变换（单骨骼 region）。
    #[serde(default)]
    pub use_skin: bool,
}

impl RegionAttachment {
    /// 以 bone 为唯一权重，构造居中于原点的矩形 region。
    pub fn centered(name: impl Into<String>, bone: usize, width: f32, height: f32) -> Self {
        let hw = width * 0.5;
        let hh = height * 0.5;
        let corners = [
            Vec2::new(-hw, -hh),
            Vec2::new(hw, -hh),
            Vec2::new(hw, hh),
            Vec2::new(-hw, hh),
        ];
        let vertices = corners.iter().map(|&p| Vertex::single(p, bone)).collect();
        Self { name: name.into(), bone, width, height, vertices, use_skin: false }
    }

    /// 校验：权重和 ≈ 1。
    pub fn validate_weights(&self) -> Result<(), String> {
        const EPS: f32 = 1e-3;
        for (i, v) in self.vertices.iter().enumerate() {
            if v.weights.len() > MAX_WEIGHTS {
                return Err(format!("vertex[{}] 权重数 {} > MAX_WEIGHTS {}", i, v.weights.len(), MAX_WEIGHTS));
            }
            let sum: f32 = v.weights.iter().map(|(_, w)| *w).sum();
            if (sum - 1.0).abs() > EPS {
                return Err(format!("vertex[{}] 权重和 {} ≠ 1", i, sum));
            }
        }
        Ok(())
    }
}
