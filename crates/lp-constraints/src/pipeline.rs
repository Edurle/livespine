//! 求解流水线（docs 19-P5高级 §模块3，原理见 05d-求解流水线）。
//!
//! 约束按声明顺序求解；建议数据顺序 IK → Transform → Physics（见 05d）。
//! Physics 需跨帧状态 + dt，用 PhysicsStateMap 按 bone 索引关联。
//! 每约束后全量 update_world。

use crate::ik::{solve_ik, IkConstraint};
use crate::physics::{solve_physics, PhysicsConstraint, PhysicsRuntimeState};
use crate::transform::{solve_transform, TransformConstraint};
use lp_core::skeleton::Skeleton;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 约束枚举。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Constraint {
    #[serde(rename = "ik")]
    Ik(IkConstraint),
    #[serde(rename = "transform")]
    Transform(TransformConstraint),
    #[serde(rename = "physics")]
    Physics(PhysicsConstraint),
}

impl Constraint {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Constraint::Ik(c) => c.validate(),
            Constraint::Transform(c) => c.validate(),
            Constraint::Physics(c) => c.validate(),
        }
    }
}

/// Physics 运行时状态集合：bone 索引 → 状态。
pub type PhysicsStateMap = HashMap<usize, PhysicsRuntimeState>;

/// 按声明顺序求解所有约束。
///
/// - `physics_states`:Physics 跨帧状态（Play 模式传入；Seek 模式传空或冻结）
/// - `dt`:时间步（Physics 用；非 Physics 约束忽略）。0 表示 Seek/冻结。
pub fn solve_pipeline(
    skeleton: &mut Skeleton,
    constraints: &[Constraint],
    physics_states: &mut PhysicsStateMap,
    dt: f32,
) {
    for c in constraints {
        match c {
            Constraint::Ik(ik) => solve_ik(skeleton, ik),
            Constraint::Transform(t) => solve_transform(skeleton, t),
            Constraint::Physics(p) => {
                let state = physics_states
                    .entry(p.bone)
                    .or_insert_with(PhysicsRuntimeState::default);
                solve_physics(skeleton, p, state, dt);
            }
        }
        skeleton.update_world();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lp_core::math::BoneLocal;
    use lp_core::skeleton::BoneData;

    #[test]
    fn pipeline_runs_ik() {
        let mut sk = Skeleton::from_data(&[
            BoneData { name: "a".into(), parent: None, length: 50.0, setup: BoneLocal::DEFAULT },
            BoneData { name: "b".into(), parent: Some(0), length: 40.0,
                setup: BoneLocal { x: 50.0, y: 0.0, ..BoneLocal::DEFAULT } },
        ]);
        sk.update_world();
        let before = sk.bones[1].local.rotation;
        let constraints = vec![Constraint::Ik(IkConstraint {
            bones: vec![0, 1], target: [50.0, 30.0],
            mix: 1.0, bend_direction: 1, softness: 0.0,
        })];
        let mut states = PhysicsStateMap::new();
        solve_pipeline(&mut sk, &constraints, &mut states, 0.0);
        assert!((sk.bones[1].local.rotation - before).abs() > 0.1, "流水线应执行 IK");
    }

    #[test]
    fn pipeline_runs_transform() {
        let mut sk = Skeleton::from_data(&[
            BoneData { name: "src".into(), parent: None, length: 10.0,
                setup: BoneLocal { rotation: 1.0, ..BoneLocal::DEFAULT } },
            BoneData { name: "tgt".into(), parent: None, length: 10.0,
                setup: BoneLocal::DEFAULT },
        ]);
        sk.update_world();
        let mut states = PhysicsStateMap::new();
        let constraints = vec![Constraint::Transform(TransformConstraint {
            source: 0, bones: vec![1],
            offset_rotate: 0.0, offset_x: 0.0, offset_y: 0.0,
            offset_scale_x: 1.0, offset_scale_y: 1.0,
            rotate_mix: 1.0, translate_mix: 0.0, scale_mix: 0.0,
        })];
        solve_pipeline(&mut sk, &constraints, &mut states, 0.0);
        assert!((sk.bones[1].local.rotation - 1.0).abs() < 1e-3, "Transform 应执行");
    }

    #[test]
    fn pipeline_runs_physics() {
        let mut sk = Skeleton::from_data(&[BoneData {
            name: "b".into(), parent: None, length: 10.0,
            setup: BoneLocal { rotation: 0.5, ..BoneLocal::DEFAULT },
        }]);
        sk.update_world();
        let mut states = PhysicsStateMap::new();
        let constraints = vec![Constraint::Physics(PhysicsConstraint {
            bone: 0, bone_inertia: 0.0, strength: 0.0, damping: 0.5,
            gravity: [0.0, -50.0], mass: 1.0,
            angle_min: -10.0, angle_max: 10.0, rotate_mix: 1.0,
        })];
        for _ in 0..10 {
            solve_pipeline(&mut sk, &constraints, &mut states, 1.0 / 60.0);
        }
        assert!(states.get(&0).unwrap().angle.abs() > 0.001, "Physics 应推进 angle");
    }

    #[test]
    fn empty_constraints_noop() {
        let mut sk = Skeleton::from_data(&[BoneData {
            name: "a".into(), parent: None, length: 10.0, setup: BoneLocal::DEFAULT,
        }]);
        sk.update_world();
        let before = sk.bones[0].local.rotation;
        let mut states = PhysicsStateMap::new();
        solve_pipeline(&mut sk, &[], &mut states, 1.0 / 60.0);
        assert_eq!(sk.bones[0].local.rotation, before);
    }
}
