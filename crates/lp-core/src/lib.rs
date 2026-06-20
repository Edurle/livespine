//! Livepine 核心内核：纯数学 + 数据模型，无 I/O 依赖。
//!
//! 设计纪律（见 docs 03-分层架构 原则 1）：
//! - 本 crate 禁止依赖任何渲染/窗口/网络/文件系统库
//! - 只接受数值输入，产出数值输出
//!
//! 模块（见 docs 14-P0内核实现清单）：
//! - [`math`]    基础数值类型 + 仿射变换运算（模块 1-2）
//! - [`skeleton`] 骨骼树 + 世界矩阵复合（模块 3）
//! - [`attach`]  附件与顶点（模块 4）
//! - [`skin`]    LBS 蒙皮（模块 5）

pub mod math;
pub mod skeleton;
pub mod attach;
pub mod skin;

pub use math::{Affine, BoneLocal, Vec2};
pub use attach::{RegionAttachment, Vertex};
pub use skeleton::{Bone, BoneData, Skeleton};
