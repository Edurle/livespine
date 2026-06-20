//! 求解流水线（docs 17-P3约束 §模块3，原理见 05d-求解流水线）。
//!
//! 按声明顺序执行约束，每个约束改 local 后全量重算 world（P3 简化）。
//! P5 扩展时加 Path/Transform/Physics + dirty 优化。

use crate::ik::{solve_ik, IkConstraint};
use lp_core::skeleton::Skeleton;
use serde::{Deserialize, Serialize};

/// 约束枚举。P3 只有 IK；P5 加 Path/Transform/Physics。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Constraint {
    #[serde(rename = "ik")]
    Ik(IkConstraint),
    // P5: Path, Transform, Physics
}

impl Constraint {
    /// 校验内部数据。
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Constraint::Ik(ik) => ik.validate(),
        }
    }
}

/// 按声明顺序求解所有约束。
///
/// 每个约束后全量 update_world（P3 简化；P5 改 dirty 增量）。
pub fn solve_pipeline(skeleton: &mut Skeleton, constraints: &[Constraint]) {
    for c in constraints {
        match c {
            Constraint::Ik(ik) => solve_ik(skeleton, ik),
        }
        // 约束改了 local，重算 world（全量，P3）
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
        solve_pipeline(&mut sk, &constraints);
        assert!((sk.bones[1].local.rotation - before).abs() > 0.1, "流水线应执行 IK");
    }

    #[test]
    fn empty_constraints_noop() {
        let mut sk = Skeleton::from_data(&[BoneData {
            name: "a".into(), parent: None, length: 10.0, setup: BoneLocal::DEFAULT,
        }]);
        sk.update_world();
        let before = sk.bones[0].local.rotation;
        solve_pipeline(&mut sk, &[]);
        assert_eq!(sk.bones[0].local.rotation, before);
    }
}
