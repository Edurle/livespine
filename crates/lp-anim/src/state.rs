//! Animation 与 AnimationState（docs 16-P2动画 §模块4）。
//!
//! Animation = 多条时间线的集合；AnimationState = 播放控制（时间推进、循环）。
//! apply() 把动画采样结果写入 Skeleton 的骨骼 local（在 setup 之上）。

use crate::timeline::{Property, Timeline};
use lp_core::skeleton::Skeleton;
use serde::{Deserialize, Serialize};

/// 一个动画：多条时间线 + 总时长。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Animation {
    pub name: String,
    /// 动画总时长（秒）。
    pub duration: f32,
    pub timelines: Vec<Timeline>,
}

impl Animation {
    /// 校验所有时间线。
    pub fn validate(&self) -> Result<(), String> {
        for tl in &self.timelines {
            tl.validate()?;
        }
        Ok(())
    }
}

/// 动画播放状态。
pub struct AnimationState<'a> {
    pub animation: &'a Animation,
    /// 当前播放时间（秒）。
    pub current_time: f32,
    /// 是否循环。
    pub looping: bool,
}

impl<'a> AnimationState<'a> {
    pub fn new(animation: &'a Animation) -> Self {
        Self { animation, current_time: 0.0, looping: true }
    }

    /// 推进时间。loop 时对 duration 取模。
    pub fn update(&mut self, dt: f32) {
        self.current_time += dt;
        if self.looping && self.animation.duration > 0.0 {
            self.current_time %= self.animation.duration;
        }
    }

    /// 设置到指定时间（用于按帧渲染）。loop 时取模。
    pub fn seek(&mut self, time: f32) {
        self.current_time = if self.looping && self.animation.duration > 0.0 {
            ((time % self.animation.duration) + self.animation.duration) % self.animation.duration
        } else {
            time.clamp(0.0, self.animation.duration)
        };
    }

    /// 把动画采样应用到骨架：骨骼 local = setup + 动画值（仅覆盖有时间线的属性）。
    ///
    /// 前置：skeleton 的 bones[i].local 应已是 setup pose（调用方负责先重置）。
    pub fn apply(&self, skeleton: &mut Skeleton) {
        for tl in &self.animation.timelines {
            let value = tl.sample(self.current_time);
            let bone = &mut skeleton.bones[tl.bone].local;
            match tl.property {
                Property::Rotate => bone.rotation = value,
                Property::X => bone.x = value,
                Property::Y => bone.y = value,
                Property::ScaleX => bone.scale_x = value,
                Property::ScaleY => bone.scale_y = value,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::Curve;
    use crate::timeline::Keyframe;
    use lp_core::math::BoneLocal;
    use lp_core::skeleton::BoneData;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    fn make_anim() -> Animation {
        Animation {
            name: "test".into(),
            duration: 1.0,
            timelines: vec![Timeline {
                bone: 0,
                property: Property::Rotate,
                keyframes: vec![
                    Keyframe { time: 0.0, value: 0.0, curve: Curve::LINEAR },
                    Keyframe { time: 1.0, value: 1.0, curve: Curve::LINEAR },
                ],
            }],
        }
    }

    fn make_skeleton() -> Skeleton {
        Skeleton::from_data(&[BoneData {
            name: "b".into(), parent: None, length: 1.0,
            setup: BoneLocal { x: 5.0, y: 5.0, rotation: 0.5, ..BoneLocal::DEFAULT },
        }])
    }

    #[test]
    fn loop_wraps_time() {
        let anim = make_anim();
        let mut st = AnimationState::new(&anim);
        st.update(1.5); // duration=1.0
        assert!(approx(st.current_time, 0.5), "loop 后 time 应 0.5, got {}", st.current_time);
    }

    #[test]
    fn seek_takes_modulo() {
        let anim = make_anim();
        let mut st = AnimationState::new(&anim);
        st.seek(1.7);
        assert!(approx(st.current_time, 0.7));
        st.seek(-0.3); // 负数也应正确取模
        assert!(approx(st.current_time, 0.7));
    }

    #[test]
    fn apply_overrides_only_timelined_property() {
        let anim = make_anim();
        let mut sk = make_skeleton();
        let mut st = AnimationState::new(&anim);
        st.seek(0.5); // rotate 中点 → 0.5
        st.apply(&mut sk);

        // rotate 被动画覆盖（0.5）
        assert!(approx(sk.bones[0].local.rotation, 0.5));
        // x/y 未被动画涉及，保持 setup（5.0）
        assert!(approx(sk.bones[0].local.x, 5.0));
        assert!(approx(sk.bones[0].local.y, 5.0));
    }

    #[test]
    fn apply_at_end_uses_last_value() {
        let anim = make_anim();
        let mut sk = make_skeleton();
        let mut st = AnimationState::new(&anim);
        st.looping = false; // 非 loop：seek 到 duration 不取模
        st.seek(1.0);
        st.apply(&mut sk);
        assert!(approx(sk.bones[0].local.rotation, 1.0));
    }
}
