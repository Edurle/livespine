//! lp-render：wgpu 离屏渲染后端。
//!
//! P1 范围（见 docs 15-P1渲染最小闭环）：离屏渲染 → PNG，无窗口/事件循环。
//! 贴图程序生成，不加载外部 PNG。

pub mod renderer;

pub use renderer::{RegionDraw, Renderer};
