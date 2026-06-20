//! Livepine 约束系统。
//!
//! - [`ik`]        IK 约束（单骨/双骨解析解）
//! - [`transform`] Transform 约束（变换复制）
//! - [`physics`]   Physics 约束（阻尼弹簧，跨帧状态）
//! - [`pipeline`]  求解流水线（按声明顺序执行约束）

pub mod ik;
pub mod physics;
pub mod pipeline;
pub mod transform;

pub use ik::{solve_ik, IkConstraint};
pub use physics::{solve_physics, PhysicsConstraint, PhysicsRuntimeState};
pub use pipeline::{solve_pipeline, Constraint, PhysicsStateMap};
pub use transform::{solve_transform, TransformConstraint};
