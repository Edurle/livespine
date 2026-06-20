//! 关键帧时间线（docs 16-P2动画 §模块3）。
//!
//! 每条时间线管一根骨骼的一个属性（rotate/x/y/scaleX/scaleY）。
//! sample(time) 在关键帧间用 Curve 插值求值。

use crate::curve::Curve;
use serde::{Deserialize, Serialize};

/// 时间线驱动的骨骼属性。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Property {
    Rotate,
    X,
    Y,
    ScaleX,
    ScaleY,
}

/// 单个关键帧。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Keyframe {
    /// 时间（秒）。
    pub time: f32,
    /// 该属性的值。
    pub value: f32,
    /// 到下一关键帧的插值曲线（最后一帧的 curve 字段被忽略）。
    #[serde(default = "default_curve")]
    pub curve: Curve,
}

fn default_curve() -> Curve {
    Curve::LINEAR
}

/// 一条时间线：某骨骼某属性的关键帧序列。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timeline {
    /// 目标骨骼索引。
    pub bone: usize,
    /// 驱动的属性。
    pub property: Property,
    /// 关键帧（按 time 升序）。
    pub keyframes: Vec<Keyframe>,
}

impl Timeline {
    /// 校验：关键帧时间单调递增、至少 1 帧。
    pub fn validate(&self) -> Result<(), String> {
        if self.keyframes.is_empty() {
            return Err(format!("timeline(bone={}, {:?}) 无关键帧", self.bone, self.property));
        }
        for w in self.keyframes.windows(2) {
            if w[0].time >= w[1].time {
                return Err(format!(
                    "timeline(bone={}, {:?}) 关键帧时间非单调: {} >= {}",
                    self.bone, self.property, w[0].time, w[1].time
                ));
            }
        }
        Ok(())
    }

    /// 给定时间，在关键帧间插值求值。
    ///
    /// - time 早于第一帧 → 第一帧值
    /// - time 晚于最后一帧 → 最后一帧值
    /// - 两帧之间 → 用 curve 插值
    pub fn sample(&self, time: f32) -> f32 {
        let kf = &self.keyframes;
        if time <= kf[0].time {
            return kf[0].value;
        }
        if time >= kf[kf.len() - 1].time {
            return kf[kf.len() - 1].value;
        }
        // 找到 time 落在哪两帧之间
        for w in kf.windows(2) {
            if time >= w[0].time && time < w[1].time {
                let span = w[1].time - w[0].time;
                let u = if span > 0.0 { (time - w[0].time) / span } else { 0.0 };
                return w[0].curve.sample(u, w[0].value, w[1].value);
            }
        }
        // 不应到达
        kf[kf.len() - 1].value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    fn kf(time: f32, value: f32) -> Keyframe {
        Keyframe { time, value, curve: Curve::LINEAR }
    }

    #[test]
    fn sample_before_first_holds_first() {
        let tl = Timeline { bone: 0, property: Property::Rotate, keyframes: vec![kf(1.0, 10.0), kf(2.0, 20.0)] };
        assert!(approx(tl.sample(0.5), 10.0));
        assert!(approx(tl.sample(1.0), 10.0));
    }

    #[test]
    fn sample_after_last_holds_last() {
        let tl = Timeline { bone: 0, property: Property::Rotate, keyframes: vec![kf(1.0, 10.0), kf(2.0, 20.0)] };
        assert!(approx(tl.sample(2.0), 20.0));
        assert!(approx(tl.sample(3.0), 20.0));
    }

    #[test]
    fn sample_between_interpolates() {
        let tl = Timeline { bone: 0, property: Property::Rotate, keyframes: vec![kf(1.0, 10.0), kf(2.0, 20.0)] };
        assert!(approx(tl.sample(1.5), 15.0)); // 中点
        assert!(approx(tl.sample(1.25), 12.5));
    }

    #[test]
    fn sample_stepped_holds() {
        let tl = Timeline {
            bone: 0, property: Property::Rotate,
            keyframes: vec![
                Keyframe { time: 0.0, value: 10.0, curve: Curve::STEPPED },
                Keyframe { time: 1.0, value: 20.0, curve: Curve::LINEAR },
            ],
        };
        // 第一帧 stepped：在 0~1 之间保持 10
        assert!(approx(tl.sample(0.5), 10.0));
        assert!(approx(tl.sample(0.99), 10.0));
    }

    #[test]
    fn validate_rejects_nonmonotonic() {
        let tl = Timeline {
            bone: 0, property: Property::Rotate,
            keyframes: vec![kf(2.0, 10.0), kf(1.0, 20.0)], // 时间倒序
        };
        assert!(tl.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty() {
        let tl = Timeline { bone: 0, property: Property::Rotate, keyframes: vec![] };
        assert!(tl.validate().is_err());
    }
}
