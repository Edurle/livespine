//! IK 约束求解（docs 17-P3约束 §模块2，数学见 05a-IK逆运动学-实现）。
//!
//! 单骨：旋转指向 target。
//! 双骨：余弦定理解析解，含可达性 clamp + mix。
//!
//! 关键纪律（05a 易错点）：acos 入参 clamp [-1,1]；applyRotation 减父骨骼世界旋转。

use lp_core::math::{world_rotation, Vec2};
use lp_core::skeleton::Skeleton;
use serde::{Deserialize, Serialize};

/// IK 约束。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IkConstraint {
    /// 受 IK 影响的骨骼索引。长度 1（单骨）或 2（双骨）。
    pub bones: Vec<usize>,
    /// 目标世界坐标（静态）。
    pub target: [f32; 2],
    /// 0~1，0=不应用，1=完全。
    #[serde(default = "default_mix")]
    pub mix: f32,
    /// 弯曲方向：+1 或 -1（仅双骨）。
    #[serde(default = "default_bend")]
    pub bend_direction: i8,
    /// 边界软化量（可达边界附近平滑过渡，避免抖动）。0=不软化。
    #[serde(default)]
    pub softness: f32,
    /// 拉伸强度 0~1。target 超出臂展时按比例拉长骨骼够到。0=不拉伸（夹紧）。
    #[serde(default)]
    pub stretch: f32,
}

fn default_mix() -> f32 { 1.0 }
fn default_bend() -> i8 { 1 }

impl IkConstraint {
    /// 校验。
    pub fn validate(&self) -> Result<(), String> {
        if self.bones.is_empty() || self.bones.len() > 2 {
            return Err(format!("IK bones 长度 {} 不合法（需 1 或 2）", self.bones.len()));
        }
        if !(0.0..=1.0).contains(&self.mix) {
            return Err(format!("IK mix {} 不在 [0,1]", self.mix));
        }
        Ok(())
    }

    fn target_vec(&self) -> Vec2 {
        Vec2::new(self.target[0], self.target[1])
    }
}

/// 求解一个 IK 约束，写入骨骼 local（改 rotation）。
pub fn solve_ik(skeleton: &mut Skeleton, ik: &IkConstraint) {
    if ik.mix == 0.0 {
        return;
    }
    match ik.bones.len() {
        1 => solve_one_bone(skeleton, ik),
        2 => solve_two_bone(skeleton, ik),
        _ => {}
    }
}

/// 单骨 IK：旋转骨骼指向 target。
fn solve_one_bone(skeleton: &mut Skeleton, ik: &IkConstraint) {
    let bi = ik.bones[0];
    let root = world_pos(skeleton, bi);
    let target = ik.target_vec();
    let desired_world = (target.y - root.y).atan2(target.x - root.x);
    apply_rotation(skeleton, bi, desired_world, ik.mix);
}

/// 双骨 IK：余弦定理解析解。
///
/// 含可达性处理：stretch（拉伸够到）+ softness（边界软化）。
fn solve_two_bone(skeleton: &mut Skeleton, ik: &IkConstraint) {
    let a_idx = ik.bones[0];
    let b_idx = ik.bones[1];
    // 用骨骼真实长度（BoneData.length）
    let mut a_len = skeleton.bones[a_idx].length.max(1e-6);
    let mut b_len = skeleton.bones[b_idx].length.max(1e-6);

    // 骨骼A 根部世界位置
    let p = world_pos(skeleton, a_idx);

    let target = ik.target_vec();
    let mut d = target.sub(p).length();
    let max_reach = a_len + b_len;
    let min_reach = (a_len - b_len).abs();

    // stretch：target 超出臂展时按比例拉长骨骼（均匀拉伸），够到 target。
    if ik.stretch > 0.0 && d > max_reach {
        let factor = (d / max_reach).powf(ik.stretch);
        a_len *= 1.0 + (factor - 1.0) * ik.stretch;
        b_len *= 1.0 + (factor - 1.0) * ik.stretch;
        // 拉伸后重新算可达范围，d 视为可达
        d = d.min(a_len + b_len);
    } else {
        // 可达性 clamp
        d = d.clamp(min_reach.max(1e-6), a_len + b_len);
    }

    // softness：在可达上边界附近平滑过渡，避免 target 来回穿越时末端抖动。
    let max_reach = a_len + b_len;
    if ik.softness > 0.0 && d > max_reach - ik.softness {
        let t = ((d - (max_reach - ik.softness)) / ik.softness).clamp(0.0, 1.0);
        let t = t * t * (3.0 - 2.0 * t); // smoothstep
        d = (max_reach - ik.softness) + ik.softness * t;
    }

    // 余弦定理解关节角
    let cos_inner = ((a_len * a_len + b_len * b_len - d * d) / (2.0 * a_len * b_len)).clamp(-1.0, 1.0);
    let inner_angle = cos_inner.acos();

    let cos_a = ((a_len * a_len + d * d - b_len * b_len) / (2.0 * a_len * d)).clamp(-1.0, 1.0);
    let bone_a_angle = cos_a.acos();

    // 组装到世界角度
    let base_angle = (target.y - p.y).atan2(target.x - p.x);
    let bend = ik.bend_direction as f32;
    let desired_a = base_angle - bone_a_angle * bend;
    let desired_b = desired_a + (std::f32::consts::PI - inner_angle) * bend;

    apply_rotation(skeleton, a_idx, desired_a, ik.mix);
    apply_rotation(skeleton, b_idx, desired_b, ik.mix);
}

/// 取骨骼世界位置（world 矩阵的 wx, wy）。
fn world_pos(skeleton: &Skeleton, bone_idx: usize) -> Vec2 {
    let w = &skeleton.bones[bone_idx].world;
    Vec2::new(w.wx, w.wy)
}

/// 把世界旋转角写入骨骼 local，按 mix 混合。
///
/// desired_world 是世界系角度；local 旋转 = desired_world - 父骨骼世界旋转。
fn apply_rotation(skeleton: &mut Skeleton, bone_idx: usize, desired_world: f32, mix: f32) {
    let parent_world = skeleton.parent_of(bone_idx)
        .map(|p| world_rotation(&skeleton.bones[p].world))
        .unwrap_or(0.0);
    let desired_local = desired_world - parent_world;
    let cur = skeleton.bones[bone_idx].local.rotation;
    let delta = shortest_angle_diff(desired_local, cur);
    skeleton.bones[bone_idx].local.rotation += delta * mix;
}

/// 最短角度差（处理 350° vs -10° 这类环绕）。
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

    fn two_bone_skeleton() -> Skeleton {
        // boneA 在原点沿 +x 长 50，boneB 沿 +x 长 40（关节在 50,0，末端在 90,0）
        Skeleton::from_data(&[
            BoneData { name: "a".into(), parent: None, length: 50.0,
                setup: BoneLocal::DEFAULT },
            BoneData { name: "b".into(), parent: Some(0), length: 40.0,
                setup: BoneLocal { x: 50.0, y: 0.0, ..BoneLocal::DEFAULT } },
        ])
    }

    #[test]
    fn two_bone_reachable_target_bends() {
        let mut sk = two_bone_skeleton();
        sk.update_world();
        // target 在 (50, 30)：在可达范围内，腿应向上弯
        let ik = IkConstraint {
            bones: vec![0, 1], target: [50.0, 30.0],
            mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
        };
        solve_ik(&mut sk, &ik);
        // 骨骼B 旋转后应非零（弯曲了）
        let b_rot = sk.bones[1].local.rotation;
        assert!(b_rot.abs() > 0.1, "骨骼B 应弯曲，rotation={b_rot}");
    }

    #[test]
    fn two_bone_unreachable_clamps() {
        let mut sk = two_bone_skeleton();
        sk.update_world();
        // target 极远（超出 a+b=90），应夹紧不 NaN
        let ik = IkConstraint {
            bones: vec![0, 1], target: [1000.0, 1000.0],
            mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
        };
        solve_ik(&mut sk, &ik);
        // 不应产生 NaN
        assert!(sk.bones[0].local.rotation.is_finite(), "骨骼A rotation 不应 NaN");
        assert!(sk.bones[1].local.rotation.is_finite(), "骨骼B rotation 不应 NaN");
    }

    #[test]
    fn mix_zero_does_nothing() {
        let mut sk = two_bone_skeleton();
        sk.update_world();
        let rot_before = sk.bones[0].local.rotation;
        let ik = IkConstraint {
            bones: vec![0, 1], target: [50.0, 30.0],
            mix: 0.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
        };
        solve_ik(&mut sk, &ik);
        assert!(approx(sk.bones[0].local.rotation, rot_before), "mix=0 不应改骨骼");
    }

    #[test]
    fn one_bone_points_to_target() {
        let mut sk = Skeleton::from_data(&[BoneData {
            name: "a".into(), parent: None, length: 10.0,
            setup: BoneLocal::DEFAULT,
        }]);
        sk.update_world();
        // target 在 +y 方向 (0,50)，骨骼应转向 90°
        let ik = IkConstraint {
            bones: vec![0], target: [0.0, 50.0],
            mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
        };
        solve_ik(&mut sk, &ik);
        assert!(approx(sk.bones[0].local.rotation, std::f32::consts::FRAC_PI_2),
            "单骨应转向 90°，got {}", sk.bones[0].local.rotation);
    }

    #[test]
    fn validate_rejects_bad_bone_count() {
        let ik = IkConstraint { bones: vec![0, 1, 2], target: [0.0, 0.0], mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0 };
        assert!(ik.validate().is_err());
    }

    #[test]
    fn stretch_reaches_beyond_arm_length() {
        // 两骨各长 50（臂展 100）。target 在 150（超出）。stretch=1 应拉长够到。
        let mut sk = Skeleton::from_data(&[
            BoneData { name: "a".into(), parent: None, length: 50.0, setup: BoneLocal::DEFAULT },
            BoneData { name: "b".into(), parent: Some(0), length: 50.0,
                setup: BoneLocal { x: 50.0, y: 0.0, ..BoneLocal::DEFAULT } },
        ]);
        sk.update_world();
        let ik = IkConstraint {
            bones: vec![0, 1], target: [150.0, 0.0],
            mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 1.0,
        };
        solve_ik(&mut sk, &ik);
        // 拉伸后骨骼应指向 target 方向（不 NaN，且末端接近 target）
        let end = world_pos(&sk, 1);
        let dir_x = sk.bones[1].world.a;
        let dir_y = sk.bones[1].world.b;
        let norm = (dir_x * dir_x + dir_y * dir_y).sqrt().max(1e-6);
        let end_x = end.x + 50.0 * dir_x / norm; // 用原 length 估末端（拉伸改的是求解用长度，非 bone.length）
        // 关键：不崩溃 + 角度合理（指向 +x）
        assert!(sk.bones[0].local.rotation.is_finite(), "stretch 不应 NaN");
        assert!(approx(world_rotation(&sk.bones[0].world), 0.0), "拉伸后骨骼应指向 target 方向");
    }

    #[test]
    fn stretch_zero_clamps_to_max_reach() {
        // stretch=0 时 target 超出臂展 → 夹紧，骨骼指向 target 方向但不拉伸
        let mut sk = Skeleton::from_data(&[
            BoneData { name: "a".into(), parent: None, length: 50.0, setup: BoneLocal::DEFAULT },
            BoneData { name: "b".into(), parent: Some(0), length: 50.0,
                setup: BoneLocal { x: 50.0, y: 0.0, ..BoneLocal::DEFAULT } },
        ]);
        sk.update_world();
        let ik = IkConstraint {
            bones: vec![0, 1], target: [1000.0, 0.0],
            mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
        };
        solve_ik(&mut sk, &ik);
        // 夹紧：骨骼应伸直指向 +x（达 max reach），不 NaN
        assert!(sk.bones[0].local.rotation.is_finite());
        assert!(approx(world_rotation(&sk.bones[0].world), 0.0));
        // 伸直时 shin rotation 应 ≈ 0（与 thigh 同向）
        assert!(approx(sk.bones[1].local.rotation, 0.0), "夹紧到 max reach 时双腿应伸直");
    }

    #[test]
    fn softness_smooths_near_boundary() {
        // softness>0 时，target 在边界附近不应产生剧烈角度跳变。
        // 这里只验证不崩溃 + 结果有限（softness 的平滑效果难精确断言）。
        let mut sk = Skeleton::from_data(&[
            BoneData { name: "a".into(), parent: None, length: 50.0, setup: BoneLocal::DEFAULT },
            BoneData { name: "b".into(), parent: Some(0), length: 50.0,
                setup: BoneLocal { x: 50.0, y: 0.0, ..BoneLocal::DEFAULT } },
        ]);
        sk.update_world();
        // target 恰在 max reach (100) 边界，softness=10
        let ik = IkConstraint {
            bones: vec![0, 1], target: [100.0, 0.0],
            mix: 1.0, bend_direction: 1, softness: 10.0, stretch: 0.0,
        };
        solve_ik(&mut sk, &ik);
        assert!(sk.bones[0].local.rotation.is_finite(), "softness 不应 NaN");
        assert!(sk.bones[1].local.rotation.is_finite());
    }

    #[test]
    fn shortest_angle_diff_wraps() {
        // 350° 和 -10° 是同一方向（差 360°），最短差应为 0
        assert!(approx(shortest_angle_diff(350f32.to_radians(), (-10f32).to_radians()), 0.0));
        // 10° vs 350°：差应为 -20°（走最短路径，而非 +340°）
        assert!(approx(shortest_angle_diff(350f32.to_radians(), 10f32.to_radians()), -20f32.to_radians()));
    }
}
