//! Livepine CLI（P0：仅 solve 子命令）。
//!
//! 退出标准（docs 14-P0内核实现清单）：`livepine solve <path.lp>`
//! → 加载、update_world、蒙皮，打印每根骨骼 world + 每个顶点坐标。
//!
//! 完整 CLI（inspect/diff/render 等）是 P6 的事。

use lp_core::skin::skin_region;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "solve" {
        eprintln!("用法: livepine solve <path.lp>");
        eprintln!("P0 仅支持 solve 子命令");
        return ExitCode::FAILURE;
    }
    let path = Path::new(&args[2]);
    match run_solve(path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("错误: {e}");
            ExitCode::FAILURE
        }
    }
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
        let pts = skin_region(region, &skeleton);
        for (i, p) in pts.iter().enumerate() {
            println!("vertex[{i}]  ({:.4}, {:.4})", p.x, p.y);
        }
    }
    Ok(())
}
