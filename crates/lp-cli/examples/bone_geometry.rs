//! 打印骨骼几何（根、末端、长度、世界方向），验证编辑器渲染。
//! 用法：cargo run -p lp-cli --example bone_geometry

use lp_core::math::world_rotation;

fn main() {
    let json = include_str!("../../../tests/fixtures/stickman.lp");
    let file = lp_io::LpFile::from_json(json).unwrap();
    let sk = file.build_skeleton();

    println!("=== 火柴人 setup pose 骨骼几何 ===");
    for (i, b) in sk.bones.iter().enumerate() {
        let name = &file.skeleton.bones[i].name;
        let (x1, y1) = (b.world.wx, b.world.wy);
        let dir = world_rotation(&b.world);
        let (x2, y2) = (x1 + b.length * dir.cos(), y1 + b.length * dir.sin());
        let actual_len = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        println!(
            "  bone[{i}] {name:6} 根({x1:>6.1},{y1:>6.1}) 末端({x2:>6.1},{y2:>6.1}) 长{:.0} 方向{:>6.1}°",
            actual_len, dir.to_degrees()
        );
    }
}
