//! Transform 约束（docs 19-P5高级 §模块2，数学见 05c-变换与物理约束-实现 §A）。
//!
//! 把 source 骨骼的变换（旋转/平移/缩放）复制到 target 骨骼，加 offset，按 mix 混合。
//! 增量式（+=/*=），叠加到动画值之上。

use lp_core::math::{world_rotation, world_scale};
use lp_core::skeleton::Skeleton;
use serde::{Deserialize, Serialize};

/// Transform 约束。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformConstraint {
    /// 源骨骼索引。
    pub source: usize,
    /// 目标骨骼索引列表（可一拖多）。
    pub bones: Vec<usize>,
    /// 偏移量（旋转弧度/平移/缩放）。
    #[serde(default)]
    pub offset_rotate: f32,
    #[serde(default)]
    pub offset_x: f32,
    #[serde(default)]
    pub offset_y: f32,
    #[serde(default = "one")]
    pub offset_scale_x: f32,
    #[serde(default = "one")]
    pub offset_scale_y: f32,
    /// 混合强度 0~1。
    #[serde(default)]
    pub rotate_mix: f32,
    #[serde(default)]
    pub translate_mix: f32,
    #[serde(default)]
    pub scale_mix: f32,
}

fn one() -> f32 { 1.0 }

impl TransformConstraint {
    pub fn validate(&self) -> Result<(), String> {
        if self.bones.is_empty() {
            return Err("transform bones 为空".into());
        }
        Ok(())
    }
}

/// 求解 Transform 约束（增量式）。
pub fn solve_transform(skeleton: &mut Skeleton, c: &TransformConstraint) {
    // 取 source 世界状态
    let src_world = skeleton.bones[c.source].world;
    let src_rot = world_rotation(&src_world);
    let (src_sx, src_sy) = world_scale(&src_world);
    let src_wx = src_world.wx;
    let src_wy = src_world.wy;

    for &target_idx in &c.bones {
        // 旋转：source 旋转 + offset，增量叠加
        if c.rotate_mix != 0.0 {
            let cur_rot = world_rotation(&skeleton.bones[target_idx].world);
            let delta = shortest_angle_diff(src_rot + c.offset_rotate, cur_rot);
            skeleton.bones[target_idx].local.rotation += delta * c.rotate_mix;
        }
        // 平移：source 世界位置 + offset - target 世界位置，增量
        if c.translate_mix != 0.0 {
            let tw = &skeleton.bones[target_idx].world;
            let dx = (src_wx + c.offset_x - tw.wx) * c.translate_mix;
            let dy = (src_wy + c.offset_y - tw.wy) * c.translate_mix;
            skeleton.bones[target_idx].local.x += dx;
            skeleton.bones[target_idx].local.y += dy;
        }
        // 缩放：乘法增量
        if c.scale_mix != 0.0 {
            let (cur_sx, cur_sy) = world_scale(&skeleton.bones[target_idx].world);
            let factor_x = 1.0 + (src_sx * c.offset_scale_x - cur_sx) * c.scale_mix;
            let factor_y = 1.0 + (src_sy * c.offset_scale_y - cur_sy) * c.scale_mix;
            skeleton.bones[target_idx].local.scale_x *= factor_x;
            skeleton.bones[target_idx].local.scale_y *= factor_y;
        }
    }
}

fn shortest_angle_diff(to: f32, from: f32) -> f32 {
    let mut d = (to - from) % std::f32::consts::TAU;
    if d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    } else if d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use lp_core::math::BoneLocal;
    use lp_core::skeleton::BoneData;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn transform_rotates_target_toward_source() {
        let mut sk = Skeleton::from_data(&[
            BoneData { name: "src".into(), parent: None, length: 10.0,
                setup: BoneLocal { rotation: 1.0, ..BoneLocal::DEFAULT } },
            BoneData { name: "tgt".into(), parent: None, length: 10.0,
                setup: BoneLocal::DEFAULT },
        ]);
        sk.update_world();
        let c = TransformConstraint {
            source: 0, bones: vec![1],
            offset_rotate: 0.0, offset_x: 0.0, offset_y: 0.0,
            offset_scale_x: 1.0, offset_scale_y: 1.0,
            rotate_mix: 1.0, translate_mix: 0.0, scale_mix: 0.0,
        };
        solve_transform(&mut sk, &c);
        // target rotation 应回到 source 的 1.0
        assert!(approx(sk.bones[1].local.rotation, 1.0),
            "target rot={}, 应 1.0", sk.bones[1].local.rotation);
    }

    #[test]
    fn transform_mix_zero_noop() {
        let mut sk = Skeleton::from_data(&[
            BoneData { name: "src".into(), parent: None, length: 10.0,
                setup: BoneLocal { rotation: 1.0, ..BoneLocal::DEFAULT } },
            BoneData { name: "tgt".into(), parent: None, length: 10.0,
                setup: BoneLocal::DEFAULT },
        ]);
        sk.update_world();
        let before = sk.bones[1].local.rotation;
        let c = TransformConstraint {
            source: 0, bones: vec![1],
            offset_rotate: 0.0, offset_x: 0.0, offset_y: 0.0,
            offset_scale_x: 1.0, offset_scale_y: 1.0,
            rotate_mix: 0.0, translate_mix: 0.0, scale_mix: 0.0,
        };
        solve_transform(&mut sk, &c);
        assert!(approx(sk.bones[1].local.rotation, before), "mix=0 不改");
    }

    #[test]
    fn transform_translates_target() {
        let mut sk = Skeleton::from_data(&[
            BoneData { name: "src".into(), parent: None, length: 10.0,
                setup: BoneLocal { x: 50.0, y: 0.0, ..BoneLocal::DEFAULT } },
            BoneData { name: "tgt".into(), parent: None, length: 10.0,
                setup: BoneLocal::DEFAULT },
        ]);
        sk.update_world();
        let c = TransformConstraint {
            source: 0, bones: vec![1],
            offset_rotate: 0.0, offset_x: 0.0, offset_y: 0.0,
            offset_scale_x: 1.0, offset_scale_y: 1.0,
            rotate_mix: 0.0, translate_mix: 1.0, scale_mix: 0.0,
        };
        solve_transform(&mut sk, &c);
        // target 应移到 source 位置（增量）
        assert!(approx(sk.bones[1].local.x, 50.0), "x={}", sk.bones[1].local.x);
    }
}
