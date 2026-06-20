//! Livepine CLI。
//!
//! P0: `solve` 打印坐标。
//! P1: `render` 离屏渲染到 PNG。

use lp_core::skin::transform_region;
use lp_render::{RegionDraw, Renderer};
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        return usage();
    }
    let result: Result<(), Box<dyn std::error::Error>> = match args[1].as_str() {
        "solve" => run_solve(Path::new(&args[2])),
        "render" => {
            let out = args.iter().position(|a| a == "-o")
                .and_then(|i| args.get(i + 1))
                .map(String::as_str)
                .unwrap_or("out.png");
            let width = parse_opt(&args, "--width", 512);
            let height = parse_opt(&args, "--height", 512);
            let anim = args.iter().position(|a| a == "--anim")
                .and_then(|i| args.get(i + 1)).cloned();
            let time = args.iter().position(|a| a == "--time")
                .and_then(|i| args.get(i + 1))
                .and_then(|v| v.parse::<f32>().ok());
            run_render(Path::new(&args[2]), Path::new(out), width, height, anim.as_deref(), time)
        }
        _ => return usage(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("错误: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> ExitCode {
    eprintln!("用法:");
    eprintln!("  livepine solve <path.lp>");
    eprintln!("  livepine render <path.lp> [-o out.png] [--width N] [--height N]");
    ExitCode::FAILURE
}

fn parse_opt(args: &[String], flag: &str, default: u32) -> u32 {
    args.iter().position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn run_solve(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = lp_io::LpFile::load(path)?;
    let skeleton = file.build_skeleton();

    println!("=== 骨骼 world 矩阵 ===");
    for (i, b) in skeleton.bones.iter().enumerate() {
        let w = &b.world;
        println!(
            "bone[{i}]  a={:.4} b={:.4} c={:.4} d={:.4} wx={:.4} wy={:.4}",
            w.a, w.b, w.c, w.d, w.wx, w.wy
        );
    }

    println!("\n=== 顶点坐标 ===");
    for region in &file.regions {
        let pts = transform_region(region, &skeleton);
        for (i, p) in pts.iter().enumerate() {
            println!("vertex[{i}]  ({:.4}, {:.4})", p.x, p.y);
        }
    }
    Ok(())
}

fn run_render(path: &Path, out: &Path, width: u32, height: u32, anim: Option<&str>, time: Option<f32>) -> Result<(), Box<dyn std::error::Error>> {
    let file = lp_io::LpFile::load(path)?;
    let mut skeleton = file.build_skeleton();

    // 若指定动画，采样第 time 秒并 apply（在 setup 之上）
    if let (Some(anim_name), Some(t)) = (anim, time) {
        let animation = file.find_anim(anim_name)
            .ok_or_else(|| format!("动画 '{anim_name}' 不存在"))?;
        let mut state = lp_anim::AnimationState::new(animation);
        state.seek(t);
        state.apply(&mut skeleton);
        skeleton.update_world();
    }

    // 约束求解（IK 等，按声明顺序）。每个约束后流水线内部会重算 world。
    if !file.constraints.is_empty() {
        lp_constraints::solve_pipeline(&mut skeleton, &file.constraints);
    }

    // 变换每个 region → RegionDraw（4 顶点 position + uv）
    // region 用 Unity 式父子变换（骨骼动顶点动），非 LBS
    let mut draws = Vec::new();
    for region in &file.regions {
        let pts = transform_region(region, &skeleton);
        if pts.len() != 4 {
            return Err(format!("region '{}' 顶点数 {} ≠ 4（P1 仅支持矩形 region）", region.name, pts.len()).into());
        }
        // UV：左下(0,0) 右下(1,0) 右上(1,1) 左上(0,1)
        // 注意 pts 顺序与 RegionAttachment::centered 一致：左下、右下、右上、左上
        let vertices = [
            [pts[0].x, pts[0].y, 0.0, 0.0],
            [pts[1].x, pts[1].y, 1.0, 0.0],
            [pts[2].x, pts[2].y, 1.0, 1.0],
            [pts[3].x, pts[3].y, 0.0, 1.0],
        ];
        draws.push(RegionDraw {
            vertices,
            color: [1.0, 0.8, 0.4, 1.0], // 暖橙色（P1 程序上色）
        });
    }

    let renderer = Renderer::new(width, height);
    renderer.render_to_png(&draws, out);
    println!("已渲染 {} 个 region → {}", draws.len(), out.display());
    Ok(())
}
