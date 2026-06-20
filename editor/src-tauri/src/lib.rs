//! Livepine 编辑器后端（Tauri 2）。
//!
//! 通过 Tauri command 把内核能力暴露给前端。
//! P4 只读：load_skeleton + sample_pose。

use lp_io::LpFile;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// 前端绘制用的姿态数据。
#[derive(Serialize, Deserialize)]
pub struct Pose {
    /// 骨骼（供调试绘制）：根位置 + 末端位置。
    pub bones: Vec<BoneDraw>,
    /// region 多边形顶点（世界坐标）。
    pub regions: Vec<RegionDraw>,
}

#[derive(Serialize, Deserialize)]
pub struct BoneDraw {
    pub name: String,
    pub x1: f32, pub y1: f32, // 根部
    pub x2: f32, pub y2: f32, // 末端
}

#[derive(Serialize, Deserialize)]
pub struct RegionDraw {
    pub name: String,
    /// 多边形顶点 [x, y]（世界坐标，y 向上）。
    pub vertices: Vec<[f32; 2]>,
    pub color: [f32; 4],
}

/// 加载 .lp 文件后返回的元信息。
#[derive(Serialize, Deserialize)]
pub struct SkeletonInfo {
    pub bone_names: Vec<String>,
    pub animation_names: Vec<String>,
    pub duration: f32,
}

/// 全局状态：已加载的 LpFile + Physics 跨帧状态（Mutex 保护）。
pub struct AppState {
    pub file: Mutex<Option<LpFile>>,
    /// Physics 跨帧状态：Play 模式累积，Seek 模式重置。
    pub physics_states: Mutex<lp_constraints::PhysicsStateMap>,
}

/// 加载 .lp 文件（指定路径）。
#[tauri::command]
fn load_skeleton(path: String, state: State<'_, AppState>) -> Result<SkeletonInfo, String> {
    load_from(&path, state)
}

/// 加载默认 stickman.lp（P4 简化：前端启动即加载）。
///
/// 路径解析顺序（避免硬编码绝对路径，提升可移植性）：
/// 1. 环境变量 `LIVEPINE_FIXTURE`（优先）
/// 2. 相对路径（开发时 cargo run 在 src-tauri 目录）
#[tauri::command]
fn load_default(state: State<'_, AppState>) -> Result<SkeletonInfo, String> {
    let path = std::env::var("LIVEPINE_FIXTURE")
        .unwrap_or_else(|_| "../../tests/fixtures/stickman.lp".to_string());
    load_from(&path, state)
}

fn load_from(path: &str, state: State<'_, AppState>) -> Result<SkeletonInfo, String> {
    let file = LpFile::load(std::path::Path::new(path)).map_err(|e| e.to_string())?;
    let info = SkeletonInfo {
        bone_names: file.skeleton.bones.iter().map(|b| b.name.clone()).collect(),
        animation_names: file.animations.iter().map(|a| a.name.clone()).collect(),
        duration: file.animations.first().map(|a| a.duration).unwrap_or(0.0),
    };
    *state.file.lock().map_err(|e| e.to_string())? = Some(file);
    Ok(info)
}

/// 采样某动画某时刻的姿态。
///
/// - mode="play":Physics 用固定 dt 推进跨帧状态（实时播放，真实惯性）
/// - mode="seek":Physics 冻结（dt=0，清状态），用于拖进度条精确定位
#[tauri::command]
fn sample_pose(
    anim: Option<String>,
    time: f32,
    mode: Option<String>,
    state: State<'_, AppState>,
) -> Result<Pose, String> {
    let guard = state.file.lock().map_err(|e| e.to_string())?;
    let file = guard.as_ref().ok_or("未加载骨架")?;

    let mut skeleton = file.build_skeleton().map_err(|e| e.to_string())?;

    // 动画 apply
    if let Some(name) = &anim {
        if let Some(animation) = file.find_anim(name) {
            let mut st = lp_anim::AnimationState::new(animation);
            st.seek(time);
            st.apply(&mut skeleton);
            skeleton.update_world();
        }
    }

    // 约束求解：Play 推进 Physics（固定 60Hz dt），Seek 冻结 + 清状态
    let is_play = mode.as_deref() == Some("play");
    let dt = if is_play { 1.0 / 60.0 } else { 0.0 };
    if !is_play {
        // Seek 模式：重置 Physics 状态，避免拖动时残留累积
        state.physics_states.lock().map_err(|e| e.to_string())?.clear();
    }
    if !file.constraints.is_empty() {
        let mut pstates = state.physics_states.lock().map_err(|e| e.to_string())?;
        lp_constraints::solve_pipeline(&mut skeleton, &file.constraints, &mut pstates, dt);
    }

    // 骨骼绘制数据：根 + 末端
    let bones = skeleton.bones.iter().enumerate().map(|(i, b)| {
        let x1 = b.world.wx;
        let y1 = b.world.wy;
        // 末端 = 根 + length 沿世界 X 方向
        let dir_x = b.world.a;
        let dir_y = b.world.b;
        let norm = (dir_x * dir_x + dir_y * dir_y).sqrt().max(1e-6);
        let x2 = x1 + b.length * dir_x / norm;
        let y2 = y1 + b.length * dir_y / norm;
        BoneDraw {
            name: file.skeleton.bones[i].name.clone(),
            x1, y1, x2, y2,
        }
    }).collect();

    // region 绘制数据：蒙皮后的世界顶点
    // use_skin=true 用 LBS（多骨骼布料），否则 Unity 式父子变换（单骨骼 region）
    let regions = file.regions.iter().map(|r| {
        let pts = if r.use_skin {
            lp_core::skin::skin_region(r, &skeleton)
        } else {
            lp_core::skin::transform_region(r, &skeleton)
        };
        RegionDraw {
            name: r.name.clone(),
            vertices: pts.iter().map(|p| [p.x, p.y]).collect(),
            // use_skin 的 region（披风）用红色，便于区分；其余橙色
            color: if r.use_skin { [1.0, 0.2, 0.2, 1.0] } else { [1.0, 0.6, 0.3, 1.0] },
        }
    }).collect();

    Ok(Pose { bones, regions })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            file: Mutex::new(None),
            physics_states: Mutex::new(std::collections::HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![load_skeleton, load_default, sample_pose])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
