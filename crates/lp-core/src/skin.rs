//! 模块 5：LBS（Linear Blend Skinning）蒙皮。
//!
//! 公式来源：docs 04-数学模型-变换与蒙皮-实现 §5。
//! 核心：`V_world = Σ wᵢ · (M_world_i · M_bind_i⁻¹) · V_local`，
//! 其中 `M_world · M_bind⁻¹` 称为形变矩阵，每帧每骨骼算一次，顶点循环复用。
//!
//! P0：FFD delta 恒为 0（`V_final = V_skinned + 0`），接口预留。

use crate::attach::{RegionAttachment, Vertex};
use crate::math::{multiply, Affine, Vec2};
use crate::skeleton::Skeleton;

/// 计算单根骨骼的形变矩阵 = world · bind_world_inverse。
///
/// 每帧每骨骼算一次。
pub fn deform_matrix(world: &Affine, bind_world_inverse: &Affine) -> Affine {
    multiply(world, bind_world_inverse)
}

/// 对单个顶点蒙皮（LBS 加权混合）。
///
/// 注意：bone 的 weights 指向 skeleton.bones 的索引。
/// P0 不加 FFD delta（接口的 `ffd: Vec2` 传入，目前应传 ZERO）。
pub fn skin_vertex(v: &Vertex, skeleton: &Skeleton, ffd: Vec2) -> Vec2 {
    let mut sum = Vec2::ZERO;
    for &(bone_idx, weight) in &v.weights {
        let bone = &skeleton.bones[bone_idx];
        let dm = deform_matrix(&bone.world, &bone.bind_world_inverse);
        sum = sum.add(dm.transform_point(v.local).scale(weight));
    }
    sum.add(ffd)
}

/// 对整个 region 附件蒙皮，返回每个顶点的世界坐标。
///
/// P0 FFD delta 全为 0。
pub fn skin_region(attachment: &RegionAttachment, skeleton: &Skeleton) -> Vec<Vec2> {
    attachment.vertices.iter()
        .map(|v| skin_vertex(v, skeleton, Vec2::ZERO))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::BoneLocal;
    use crate::skeleton::BoneData;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3 // 顶点容差 1e-3
    }

    #[test]
    fn single_bone_weight_identity_at_bind() {
        // 单骨全权重，bind pose（local == setup）→ 形变矩阵为单位阵
        // 顶点 local (3,-1) 是骨骼局部坐标；bind pose 下 D=I，故顶点世界 = (3,-1)
        // （顶点世界位置 = bind 时它在世界中的位置，由 attachment 局部坐标决定）
        let data = vec![BoneData {
            name: "bone".into(), parent: None, length: 10.0,
            setup: BoneLocal { x: 5.0, y: 2.0, ..BoneLocal::DEFAULT },
        }];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();
        sk.precompute_bind_inverse();

        let v = Vertex::single(Vec2::new(3.0, -1.0), 0);
        let out = skin_vertex(&v, &sk, Vec2::ZERO);
        // bind pose：D = world · bind_inv = 单位阵，顶点世界 = local = (3,-1)
        assert!(approx(out.x, 3.0), "x={}, 应 3", out.x);
        assert!(approx(out.y, -1.0), "y={}, 应 -1", out.y);
    }

    #[test]
    fn single_bone_weight_moves_with_bone() {
        // 单骨全权重，骨骼移动后顶点跟着移动
        let data = vec![BoneData {
            name: "bone".into(), parent: None, length: 10.0,
            setup: BoneLocal::DEFAULT, // setup 在原点
        }];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();
        sk.precompute_bind_inverse();
        // 动画/手动把骨骼移到 (10, 0)
        sk.bones[0].local.x = 10.0;
        sk.update_world();

        let v = Vertex::single(Vec2::new(0.0, 0.0), 0);
        let out = skin_vertex(&v, &sk, Vec2::ZERO);
        // 顶点绑在骨骼原点，骨骼移到 (10,0) → 顶点世界 (10,0)
        assert!(approx(out.x, 10.0));
        assert!(approx(out.y, 0.0));
    }

    #[test]
    fn two_bone_weight_blend_midpoint() {
        // 两骨各 0.5 权重：顶点应在两骨位置的中点
        let data = vec![
            BoneData { name: "a".into(), parent: None, length: 5.0,
                setup: BoneLocal::DEFAULT }, // setup 原点
            BoneData { name: "b".into(), parent: None, length: 5.0,
                setup: BoneLocal::DEFAULT },
        ];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();
        sk.precompute_bind_inverse();
        // 把 a 移到 (0,0)，b 移到 (10,0)
        sk.bones[0].local = BoneLocal::DEFAULT;
        sk.bones[1].local = BoneLocal { x: 10.0, y: 0.0, ..BoneLocal::DEFAULT };
        sk.update_world();

        let v = Vertex { local: Vec2::ZERO, weights: vec![(0, 0.5), (1, 0.5)] };
        let out = skin_vertex(&v, &sk, Vec2::ZERO);
        // 中点 (5, 0)
        assert!(approx(out.x, 5.0), "blend x={}, 应 5", out.x);
        assert!(approx(out.y, 0.0));
    }

    #[test]
    fn region_skinning() {
        let data = vec![BoneData {
            name: "bone".into(), parent: None, length: 10.0,
            setup: BoneLocal::DEFAULT,
        }];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();
        sk.precompute_bind_inverse();

        let region = RegionAttachment::centered("rect", 0, 4.0, 2.0); // 4×2 矩形
        assert!(region.validate_weights().is_ok());
        let pts = skin_region(&region, &sk);
        assert_eq!(pts.len(), 4);
        // 居中于原点，4 角应分别是 (-2,-1)(2,-1)(2,1)(-2,1)
        assert!(approx(pts[0].x, -2.0) && approx(pts[0].y, -1.0));
        assert!(approx(pts[2].x, 2.0) && approx(pts[2].y, 1.0));
    }
}
