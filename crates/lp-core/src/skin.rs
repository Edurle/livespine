//! 模块 5：附件变换 / 蒙皮。
//!
//! **分层设计**（见 docs 04-数学模型-变换与蒙皮-实现）：
//! - **region 附件（单骨骼）** → [`transform_region`]：Unity 式父子变换。
//!   顶点 local 是骨骼局部坐标，世界坐标 = `bone.world · local`。
//!   骨骼动顶点跟着动，直观，性能好（1 次矩阵×点）。覆盖 80%+ 部件。
//! - **mesh 附件（多骨骼）** → [`skin_vertex`] / [`skin_region`]：LBS 加权混合。
//!   用于关节/软体等需要多骨骼影响的可变形网格。
//!
//! region 用简单变换符合 Unity GameObject 父子级、Spine region attachment 的实际做法。

use crate::attach::{RegionAttachment, Vertex};
use crate::math::{multiply, Affine, Vec2};
use crate::skeleton::Skeleton;

// ============================================================================
// region 附件：Unity 式父子变换（单骨骼）
// ============================================================================

/// 对 region 附件做 Unity 式父子变换，返回每个顶点的世界坐标。
///
/// 顶点 local 是绑定骨骼（`attachment.bone`）的**局部坐标**；
/// 世界坐标 = `bone.world · local`。
///
/// 骨骼移动/旋转，顶点立刻跟着变（无需动画驱动）。
pub fn transform_region(attachment: &RegionAttachment, skeleton: &Skeleton) -> Vec<Vec2> {
    let bone = &skeleton.bones[attachment.bone];
    attachment.vertices.iter()
        .map(|v| bone.world.transform_point(v.local))
        .collect()
}

// ============================================================================
// mesh 附件：LBS 加权混合（多骨骼）
// ============================================================================

/// 计算单根骨骼的形变矩阵 = world · bind_world_inverse。
///
/// 每帧每骨骼算一次。仅 LBS（mesh）使用。
pub fn deform_matrix(world: &Affine, bind_world_inverse: &Affine) -> Affine {
    multiply(world, bind_world_inverse)
}

/// 对单个顶点 LBS 蒙皮（多骨骼加权混合）。
///
/// 用于 mesh 附件。顶点 local 是**绑定世界坐标**（LBS 语义要求）。
/// `ffd` 为 FFD 变形偏移（P0 恒为 ZERO）。
pub fn skin_vertex(v: &Vertex, skeleton: &Skeleton, ffd: Vec2) -> Vec2 {
    let mut sum = Vec2::ZERO;
    for &(bone_idx, weight) in &v.weights {
        let bone = &skeleton.bones[bone_idx];
        let dm = deform_matrix(&bone.world, &bone.bind_world_inverse);
        sum = sum.add(dm.transform_point(v.local).scale(weight));
    }
    sum.add(ffd)
}

/// 对整个附件做 LBS 蒙皮（mesh 用）。P0 FFD delta 全为 0。
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

    // ===== transform_region（Unity 式父子变换）测试 =====

    #[test]
    fn transform_region_follows_bone_translate() {
        // region 绑在骨骼上，骨骼平移 → 顶点跟着平移
        let data = vec![BoneData {
            name: "bone".into(), parent: None, length: 10.0,
            setup: BoneLocal { x: 100.0, y: 50.0, ..BoneLocal::DEFAULT },
        }];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();

        let region = RegionAttachment::centered("rect", 0, 20.0, 20.0); // 局部 ±10
        let pts = transform_region(&region, &sk);
        // 骨骼在 (100,50)，矩形局部 ±10 → 世界 (90,40)..(110,60)
        assert!(approx(pts[0].x, 90.0) && approx(pts[0].y, 40.0), "左下 {:?}", pts[0]);
        assert!(approx(pts[2].x, 110.0) && approx(pts[2].y, 60.0), "右上 {:?}", pts[2]);
    }

    #[test]
    fn transform_region_follows_bone_move() {
        // 改骨骼位置后重新 update_world → 顶点跟着变（核心：骨骼动顶点动）
        let data = vec![BoneData {
            name: "bone".into(), parent: None, length: 10.0,
            setup: BoneLocal::DEFAULT,
        }];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();

        let region = RegionAttachment::centered("rect", 0, 20.0, 20.0);
        let before = transform_region(&region, &sk);
        // 骨骼从原点移到 (30, 40)
        sk.bones[0].local = BoneLocal { x: 30.0, y: 40.0, ..BoneLocal::DEFAULT };
        sk.update_world();
        let after = transform_region(&region, &sk);

        // 顶点整体平移 (30,40)
        assert!(approx(after[0].x - before[0].x, 30.0), "x 应平移 30");
        assert!(approx(after[0].y - before[0].y, 40.0), "y 应平移 40");
    }

    #[test]
    fn transform_region_follows_bone_rotation() {
        // 骨骼旋转 90° → 矩形顶点跟着转
        let data = vec![BoneData {
            name: "bone".into(), parent: None, length: 10.0,
            setup: BoneLocal::DEFAULT,
        }];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();

        // 自定义顶点：局部 (10, 0) 的点
        let region = RegionAttachment {
            name: "r".into(), bone: 0, width: 20.0, height: 1.0,
            vertices: vec![Vertex::single(Vec2::new(10.0, 0.0), 0)],
            use_skin: false,
        };
        let before = transform_region(&region, &sk);
        assert!(approx(before[0].x, 10.0) && approx(before[0].y, 0.0));

        // 旋转 90°
        sk.bones[0].local = BoneLocal { rotation: std::f32::consts::FRAC_PI_2, ..BoneLocal::DEFAULT };
        sk.update_world();
        let after = transform_region(&region, &sk);
        // (10,0) 转 90° → (0,10)
        assert!(approx(after[0].x.abs(), 0.0), "转后 x≈0, got {}", after[0].x);
        assert!(approx(after[0].y, 10.0), "转后 y=10, got {}", after[0].y);
    }

    #[test]
    fn transform_region_parent_chain() {
        // 父骨骼动 → 子骨骼 region 跟着动（父子链传播）
        let data = vec![
            BoneData { name: "parent".into(), parent: None, length: 10.0,
                setup: BoneLocal::DEFAULT },
            BoneData { name: "child".into(), parent: Some(0), length: 10.0,
                setup: BoneLocal { x: 5.0, y: 0.0, ..BoneLocal::DEFAULT } },
        ];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();

        let region = RegionAttachment::centered("rect", 1, 2.0, 2.0); // 绑 child，局部 ±1
        let before = transform_region(&region, &sk);

        // parent 移动 → child 跟着 → region 跟着
        sk.bones[0].local = BoneLocal { x: 20.0, y: 0.0, ..BoneLocal::DEFAULT };
        sk.update_world();
        let after = transform_region(&region, &sk);

        assert!(approx(after[0].x - before[0].x, 20.0), "父子链传播 x 平移 20");
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
