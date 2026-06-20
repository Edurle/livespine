//! Physics 约束（docs 19-P5高级 §模块1，数学见 05c-变换与物理约束-实现 §B）。
//!
//! 阻尼弹簧 + 钟摆，跨帧状态，半隐式 Euler。
//! 模拟布料/头发/披风的惯性滞后。
//!
//! 状态:PhysicsRuntimeState 跨帧保持（角速度/角度偏移）。

use lp_core::math::world_rotation;
use lp_core::skeleton::Skeleton;
use serde::{Deserialize, Serialize};

/// Physics 约束（序列化数据）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsConstraint {
    pub bone: usize,
    /// 惯性 0~1（父级运动产生的滞后量）。
    #[serde(default = "default_inertia")]
    pub bone_inertia: f32,
    /// 弹簧回复强度。
    #[serde(default = "default_strength")]
    pub strength: f32,
    /// 阻尼 0~1（越大停得越快）。
    #[serde(default = "default_damping")]
    pub damping: f32,
    /// 重力 [x, y]（世界系）。
    #[serde(default = "default_gravity")]
    pub gravity: [f32; 2],
    /// 质量。
    #[serde(default = "one")]
    pub mass: f32,
    /// 角度限制（local 空间，弧度）。
    #[serde(default = "default_min")]
    pub angle_min: f32,
    #[serde(default = "default_max")]
    pub angle_max: f32,
    /// 旋转混合 0~1。
    #[serde(default = "one")]
    pub rotate_mix: f32,
}

fn default_inertia() -> f32 { 0.5 }
fn default_strength() -> f32 { 0.3 }
fn default_damping() -> f32 { 0.85 }
fn default_gravity() -> [f32; 2] { [0.0, -50.0] }
fn one() -> f32 { 1.0 }
fn default_min() -> f32 { -1.0 }
fn default_max() -> f32 { 1.0 }

impl PhysicsConstraint {
    pub fn validate(&self) -> Result<(), String> {
        if self.mass <= 0.0 {
            return Err(format!("physics mass {} 必须 > 0", self.mass));
        }
        Ok(())
    }
}

/// Physics 跨帧运行时状态（非序列化）。
#[derive(Clone, Debug, Default)]
pub struct PhysicsRuntimeState {
    pub angle: f32,
    pub velocity: f32,
    pub last_world_x: f32,
    pub last_world_y: f32,
    pub initialized: bool,
}

/// 求解单个 Physics 约束，推进状态 dt 秒。
///
/// 半隐式 Euler（见 05c §B）。
pub fn solve_physics(
    skeleton: &mut Skeleton,
    c: &PhysicsConstraint,
    state: &mut PhysicsRuntimeState,
    dt: f32,
) {
    // dt clamp 防卡顿穿透
    let dt = dt.min(1.0 / 30.0).max(0.0);
    if dt == 0.0 {
        return;
    }

    let bone = &skeleton.bones[c.bone];
    let world_angle = world_rotation(&bone.world);
    let cur_wx = bone.world.wx;
    let cur_wy = bone.world.wy;

    // 1. 父级运动速度（世界差分）
    let (vel_x, vel_y) = if state.initialized {
        (
            (cur_wx - state.last_world_x) / dt,
            (cur_wy - state.last_world_y) / dt,
        )
    } else {
        (0.0, 0.0)
    };
    state.last_world_x = cur_wx;
    state.last_world_y = cur_wy;
    state.initialized = true;

    // 2. 各力矩分量
    let inertia_torque =
        (-vel_x * world_angle.sin() + vel_y * world_angle.cos()) * c.bone_inertia;
    let gravity_torque = c.gravity[0] * world_angle.sin() + c.gravity[1] * world_angle.cos();
    let spring_torque = -c.strength * state.angle;
    let damping_torque = -c.damping * state.velocity;
    let torque =
        (gravity_torque + spring_torque + damping_torque + inertia_torque) / c.mass;

    // 3. 半隐式 Euler：先速度后角度
    state.velocity += torque * dt;
    state.angle += state.velocity * dt;

    // 4. 角度限制 + 速度归零（防穿透）
    if state.angle < c.angle_min {
        state.angle = c.angle_min;
        state.velocity = 0.0;
    } else if state.angle > c.angle_max {
        state.angle = c.angle_max;
        state.velocity = 0.0;
    }

    // 5. apply 到 bone.local.rotation（mix）
    skeleton.bones[c.bone].local.rotation += state.angle * c.rotate_mix;
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
    fn gravity_pulls_bone_down() {
        // 重力应让 angle 增大（下垂方向，取决于重力符号）
        let mut sk = Skeleton::from_data(&[BoneData {
            name: "b".into(), parent: None, length: 10.0,
            setup: BoneLocal { rotation: 0.5, ..BoneLocal::DEFAULT },
        }]);
        sk.update_world();
        let c = PhysicsConstraint {
            bone: 0, bone_inertia: 0.0, strength: 0.0, damping: 0.5,
            gravity: [0.0, -50.0], mass: 1.0,
            angle_min: -10.0, angle_max: 10.0, rotate_mix: 1.0,
        };
        let mut state = PhysicsRuntimeState::default();
        let angle_before = state.angle;
        for _ in 0..10 {
            solve_physics(&mut sk, &c, &mut state, 1.0 / 60.0);
            sk.update_world();
        }
        // 多帧后 angle 应有变化（重力起作用）
        assert!((state.angle - angle_before).abs() > 0.001, "重力应改变 angle");
    }

    #[test]
    fn no_nan_under_many_frames() {
        let mut sk = Skeleton::from_data(&[BoneData {
            name: "b".into(), parent: None, length: 10.0,
            setup: BoneLocal::DEFAULT,
        }]);
        sk.update_world();
        let c = PhysicsConstraint {
            bone: 0, bone_inertia: 0.5, strength: 0.3, damping: 0.85,
            gravity: [0.0, -50.0], mass: 1.0,
            angle_min: -1.0, angle_max: 1.0, rotate_mix: 1.0,
        };
        let mut state = PhysicsRuntimeState::default();
        for _ in 0..600 {
            // 10 秒
            solve_physics(&mut sk, &c, &mut state, 1.0 / 60.0);
            sk.update_world();
        }
        assert!(state.angle.is_finite(), "angle 不应 NaN");
        assert!(state.velocity.is_finite(), "velocity 不应 NaN");
        // 阻尼应让系统稳定（不发散）
        assert!(state.angle.abs() < 100.0, "angle 不应发散: {}", state.angle);
    }

    #[test]
    fn angle_limit_clamps() {
        let mut sk = Skeleton::from_data(&[BoneData {
            name: "b".into(), parent: None, length: 10.0,
            setup: BoneLocal::DEFAULT,
        }]);
        sk.update_world();
        // 极强重力，小角度限制
        let c = PhysicsConstraint {
            bone: 0, bone_inertia: 0.0, strength: 0.0, damping: 0.0,
            gravity: [0.0, -10000.0], mass: 1.0,
            angle_min: -0.5, angle_max: 0.5, rotate_mix: 1.0,
        };
        let mut state = PhysicsRuntimeState::default();
        for _ in 0..300 {
            solve_physics(&mut sk, &c, &mut state, 1.0 / 60.0);
            sk.update_world();
        }
        assert!(state.angle <= 0.5 + 1e-3, "angle 应被限制 ≤0.5, got {}", state.angle);
        assert!(state.angle >= -0.5 - 1e-3, "angle 应被限制 ≥-0.5, got {}", state.angle);
    }

    #[test]
    fn zero_dt_noop() {
        let mut sk = Skeleton::from_data(&[BoneData {
            name: "b".into(), parent: None, length: 10.0, setup: BoneLocal::DEFAULT,
        }]);
        sk.update_world();
        let c = PhysicsConstraint {
            bone: 0, bone_inertia: 0.5, strength: 0.3, damping: 0.85,
            gravity: [0.0, -50.0], mass: 1.0,
            angle_min: -1.0, angle_max: 1.0, rotate_mix: 1.0,
        };
        let mut state = PhysicsRuntimeState::default();
        let before = sk.bones[0].local.rotation;
        solve_physics(&mut sk, &c, &mut state, 0.0);
        assert!(approx(sk.bones[0].local.rotation, before), "dt=0 不应改");
    }
}
