//! Livepine 动画系统。
//!
//! P2 范围（见 docs 16-P2动画）：单动画播放。
//! - [`curve`]     cubic bezier 插值曲线（含 linear/stepped）
//! - [`timeline`]  关键帧时间线
//! - [`state`]     Animation / AnimationState

pub mod curve;
pub mod timeline;
pub mod state;

pub use curve::{Curve, CurveKind};
pub use state::{Animation, AnimationState};
pub use timeline::{Keyframe, Property, Timeline};
