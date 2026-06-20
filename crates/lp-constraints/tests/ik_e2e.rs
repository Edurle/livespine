//! IK 端到端验证（docs 17-P3约束 §模块6）。
//!
//! 纯 CPU（不依赖 wgpu 渲染），稳定验证 IK 数学。
//! 手构造两骨"腿"，跑 solve_pipeline，断言腿弯曲、末端接近 target。
//!
//! 注：不从 .lp 加载（会引入 lp-io→lp-constraints 循环依赖），直接构造数据。

use lp_constraints::{solve_pipeline, Constraint, IkConstraint};
use lp_core::math::BoneLocal;
use lp_core::skeleton::{BoneData, Skeleton};

/// 大腿(thigh)在 (128,200) 沿 -y 长 60，小腿(shin)沿局部 +x 长 50。
fn build_leg() -> Skeleton {
    let sk = Skeleton::from_data(&[
        BoneData {
            name: "thigh".into(), parent: None, length: 60.0,
            setup: BoneLocal { x: 128.0, y: 200.0, rotation: -std::f32::consts::FRAC_PI_2,
                ..BoneLocal::DEFAULT },
        },
        BoneData {
            name: "shin".into(), parent: Some(0), length: 50.0,
            setup: BoneLocal { x: 60.0, y: 0.0, ..BoneLocal::DEFAULT },
        },
    ]);
    sk
}

#[test]
fn ik_bends_the_leg() {
    let mut sk = build_leg();
    sk.update_world();
    let rot_before_thigh = sk.bones[0].local.rotation;
    let rot_before_shin = sk.bones[1].local.rotation;

    // target 在大腿斜下方（不在正下方），两骨都应弯曲
    let constraints = vec![Constraint::Ik(IkConstraint {
        bones: vec![0, 1], target: [100.0, 100.0],
        mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
    })];
    let mut states = lp_constraints::PhysicsStateMap::new();
    solve_pipeline(&mut sk, &constraints, &mut states, 0.0);

    assert!(
        (sk.bones[0].local.rotation - rot_before_thigh).abs() > 0.01,
        "thigh rotation 应被 IK 改变（target 在斜下方）"
    );
    assert!(
        (sk.bones[1].local.rotation - rot_before_shin).abs() > 0.01,
        "shin rotation 应被 IK 改变"
    );
}

#[test]
fn ik_end_reaches_target() {
    let mut sk = build_leg();
    sk.update_world();
    let constraints = vec![Constraint::Ik(IkConstraint {
        bones: vec![0, 1], target: [128.0, 90.0],
        mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
    })];
    let mut states = lp_constraints::PhysicsStateMap::new();
    solve_pipeline(&mut sk, &constraints, &mut states, 0.0);

    // shin 末端 = shin 根部 + 50 沿 shin 世界方向
    let shin = &sk.bones[1];
    let norm = (shin.world.a * shin.world.a + shin.world.b * shin.world.b).sqrt().max(1e-6);
    let end_x = shin.world.wx + 50.0 * shin.world.a / norm;
    let end_y = shin.world.wy + 50.0 * shin.world.b / norm;

    let dist = ((end_x - 128.0).powi(2) + (end_y - 90.0).powi(2)).sqrt();
    assert!(dist < 8.0, "shin 末端 {end_x:.1},{end_y:.1} 应接近 target 128,90（距离 {dist:.2}）");
}

#[test]
fn ik_no_nan() {
    let mut sk = build_leg();
    sk.update_world();
    let constraints = vec![Constraint::Ik(IkConstraint {
        bones: vec![0, 1], target: [128.0, 90.0],
        mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
    })];
    let mut states = lp_constraints::PhysicsStateMap::new();
    solve_pipeline(&mut sk, &constraints, &mut states, 0.0);
    for b in &sk.bones {
        assert!(b.local.rotation.is_finite(), "rotation 不应 NaN");
        assert!(b.world.wx.is_finite() && b.world.wy.is_finite(), "world 不应 NaN");
    }
}

#[test]
fn different_targets_different_poses() {
    // 不同 target → 不同弯曲（验证 IK 真的在响应 target）
    let mut sk1 = build_leg();
    sk1.update_world();
    let mut st1 = lp_constraints::PhysicsStateMap::new();
    solve_pipeline(&mut sk1, &[Constraint::Ik(IkConstraint {
        bones: vec![0, 1], target: [128.0, 90.0], mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
    })], &mut st1, 0.0);

    let mut sk2 = build_leg();
    sk2.update_world();
    let mut st2 = lp_constraints::PhysicsStateMap::new();
    solve_pipeline(&mut sk2, &[Constraint::Ik(IkConstraint {
        bones: vec![0, 1], target: [160.0, 90.0], mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
    })], &mut st2, 0.0);

    assert!(
        (sk1.bones[0].local.rotation - sk2.bones[0].local.rotation).abs() > 0.05,
        "不同 target 应产生不同 thigh rotation"
    );
}

#[test]
fn bend_direction_mirrors_pose() {
    // 同样 target，bendDirection +1 vs -1 应产生镜像姿态（关节角符号相反）
    let target = [100.0, 100.0];

    let mut sk_pos = build_leg();
    sk_pos.update_world();
    let mut st_pos = lp_constraints::PhysicsStateMap::new();
    solve_pipeline(&mut sk_pos, &[Constraint::Ik(IkConstraint {
        bones: vec![0, 1], target, mix: 1.0, bend_direction: 1, softness: 0.0, stretch: 0.0,
    })], &mut st_pos, 0.0);

    let mut sk_neg = build_leg();
    sk_neg.update_world();
    let mut st_neg = lp_constraints::PhysicsStateMap::new();
    solve_pipeline(&mut sk_neg, &[Constraint::Ik(IkConstraint {
        bones: vec![0, 1], target, mix: 1.0, bend_direction: -1, softness: 0.0, stretch: 0.0,
    })], &mut st_neg, 0.0);

    // shin rotation 应符号相反（bendDirection 控制弯曲方向）。
    // 注：因初始姿态非对称，不要求严格镜像（和=0），只要求异号且差异显著。
    let shin_pos = sk_pos.bones[1].local.rotation;
    let shin_neg = sk_neg.bones[1].local.rotation;
    assert!(
        shin_pos * shin_neg < 0.0,
        "bendDirection ±1 应让 shin 异号：+1 得 {shin_pos:.3}, -1 得 {shin_neg:.3}"
    );
    assert!(
        (shin_pos - shin_neg).abs() > 0.5,
        "两种 bendDirection 应产生明显不同的 shin 角度"
    );
}
