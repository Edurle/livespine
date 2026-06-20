//! Livepine 约束系统。
//!
//! P3 范围（见 docs 17-P3约束）：IK 约束 + 求解流水线骨架。
//! - [`ik`]        IK 约束求解（单骨/双骨解析解）
//! - [`pipeline`]  求解流水线（按声明顺序执行约束，每约束后重算 world）

pub mod ik;
pub mod pipeline;

pub use ik::{solve_ik, IkConstraint};
pub use pipeline::{solve_pipeline, Constraint};
