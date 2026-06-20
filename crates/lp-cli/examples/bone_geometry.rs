//! 打印骨骼几何 + 披风 LBS 多帧诊断（验证 mesh+Physics 是否让披风变形）。
//! 用法：cargo run -p lp-cli --example bone_geometry

use lp_core::math::world_rotation;
use lp_core::skin::skin_region;
use std::collections::HashMap;

fn main() {
    let json = include_str!("../../../tests/fixtures/stickman.lp");
    let file = lp_io::LpFile::from_json(json).unwrap();
    let sk = file.build_skeleton().unwrap();

    println!("=== setup 骨骼几何 ===");
    for (i, b) in sk.bones.iter().enumerate() {
        let name = &file.skeleton.bones[i].name;
        let (x1, y1) = (b.world.wx, b.world.wy);
        let dir = world_rotation(&b.world);
        let (x2, y2) = (x1 + b.length * dir.cos(), y1 + b.length * dir.sin());
        println!("  bone[{i}] {name:9} 根({x1:>6.1},{y1:>6.1}) 末({x2:>6.1},{y2:>6.1}) 方向{:>6.1}°", dir.to_degrees());
    }

    // 找披风 region（use_skin=true）
    println!("\n=== region use_skin 检查 ===");
    for r in &file.regions {
        println!("  {} use_skin={}", r.name, r.use_skin);
    }
    let cape_idx = file.regions.iter().position(|r| r.use_skin).unwrap();
    let cape = &file.regions[cape_idx];

    println!("\n=== 披风 LBS 多帧诊断（模拟 Play 推进）===");
    println!("setup 披风顶点:");
    let pts0 = skin_region(cape, &sk);
    for (i, p) in pts0.iter().enumerate() {
        println!("  v[{i}] ({:.1}, {:.1})", p.x, p.y);
    }

    // 模拟 wave 动画 + Physics 推进多帧，看披风是否变形
    if let Some(anim) = file.find_anim("wave") {
        let mut sk = file.build_skeleton().unwrap();
        sk.update_world();
        let mut states: HashMap<usize, lp_constraints::PhysicsRuntimeState> = HashMap::new();
        let dt = 1.0_f32 / 60.0;

        println!("\n推进 60 帧（1 秒）后：");
        for frame in 1..=60 {
            // 动画 apply（每帧推进时间）
            let t = (frame as f32 * dt) % anim.duration;
            let mut st = lp_anim::AnimationState::new(anim);
            st.seek(t);
            st.apply(&mut sk);
            sk.update_world();
            // Physics 推进
            lp_constraints::solve_pipeline(&mut sk, &file.constraints, &mut states, dt);

            if frame == 60 {
                println!("  cape_root 方向 {:.1}°", world_rotation(&sk.bones[7].world).to_degrees());
                println!("  cape_mid  方向 {:.1}°", world_rotation(&sk.bones[8].world).to_degrees());
                println!("  cape_tip  方向 {:.1}°", world_rotation(&sk.bones[9].world).to_degrees());
                println!("  Physics cape_root.angle = {:.3}", states.get(&7).map(|s| s.angle).unwrap_or(0.0));
                let pts = skin_region(cape, &sk);
                println!("  披风顶点:");
                for (i, p) in pts.iter().enumerate() {
                    println!("    v[{i}] ({:.1}, {:.1})  变化=({:+.1}, {:+.1})",
                        p.x, p.y, p.x - pts0[i].x, p.y - pts0[i].y);
                }
            }
        }
    }
}
