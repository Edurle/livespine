//! 黄金值集成测试（docs 12b-黄金值测试-规范）。
//!
//! 遍历 tests/golden/ 下每个用例目录，加载 input.json + expected.json，
//! 跑内核 solve，比对容差。
//!
//! P0 用例：transforms/（world 矩阵）、skinning/（顶点坐标）。

use lp_core::attach::Vertex;
use lp_core::math::{BoneLocal, Vec2};
use lp_core::skeleton::{BoneData, Skeleton};
use lp_core::skin::skin_vertex;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

// ---------- 用例 JSON 结构 ----------

#[derive(Deserialize)]
struct GoldenCase {
    input: Input,
}

#[derive(Deserialize)]
struct Input {
    skeleton: SkeletonInput,
    #[serde(default)]
    pose: Vec<BoneLocal>,
    #[serde(default)]
    vertices: Vec<VertexInput>,
}

#[derive(Deserialize)]
struct SkeletonInput {
    bones: Vec<BoneData>,
}

#[derive(Deserialize)]
struct VertexInput {
    local: [f32; 2],
    weights: Vec<[serde_json::Value; 2]>,
}

impl VertexInput {
    fn to_vertex(&self) -> Vertex {
        let weights: Vec<(usize, f32)> = self.weights.iter().map(|w| {
            let idx = w[0].as_u64().unwrap() as usize;
            let wt = w[1].as_f64().unwrap() as f32;
            (idx, wt)
        }).collect();
        Vertex {
            local: Vec2::new(self.local[0], self.local[1]),
            weights,
        }
    }
}

#[derive(Deserialize)]
struct Expected {
    outputs: Outputs,
    meta: Meta,
}

#[derive(Deserialize)]
struct Outputs {
    #[serde(default)]
    bones: Vec<BoneOut>,
    #[serde(default)]
    vertices: Vec<VertexOut>,
}

#[derive(Deserialize)]
struct BoneOut {
    name: String,
    world: [f32; 6], // a,b,c,d,wx,wy
}

#[derive(Deserialize)]
struct VertexOut {
    x: f32,
    y: f32,
}

#[derive(Deserialize)]
struct Meta {
    source: String,
    #[allow(dead_code)]
    tolerance: Option<f32>,
}

// ---------- 比对 ----------

fn approx(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

const TRANSFORM_TOL: f32 = 1e-4;
const VERTEX_TOL: f32 = 1e-3;

// ---------- 发现用例 ----------

fn collect_cases() -> Vec<PathBuf> {
    // CARGO_MANIFEST_DIR = crates/lp-core，workspace 根 = 上两级
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()   // crates/
        .parent().unwrap()   // workspace 根
        .join("tests/golden");
    let mut cases = vec![];
    if let Ok(entries) = fs::read_dir(&root) {
        for cat in entries.flatten() {
            let cat_path = cat.path();
            if cat_path.is_dir() {
                // constraints/ 目录的用例由 lp-constraints 的黄金值加载器处理，跳过。
                if cat_path.file_name() == Some(std::ffi::OsStr::new("constraints")) {
                    continue;
                }
                if let Ok(subs) = fs::read_dir(&cat_path) {
                    for c in subs.flatten() {
                        if c.path().join("input.json").exists() {
                            cases.push(c.path());
                        }
                    }
                }
            }
        }
    }
    cases
}

// ---------- 主测试 ----------

#[test]
fn golden_cases_pass() {
    let cases = collect_cases();
    assert!(!cases.is_empty(), "未发现黄金值用例，检查 tests/golden 目录");

    let mut failures = Vec::new();
    for case_dir in &cases {
        let name = case_dir.file_name().unwrap().to_string_lossy();
        match run_case(case_dir) {
            Ok(()) => println!("  ✓ {name}"),
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }

    if !failures.is_empty() {
        panic!("黄金值测试失败 ({} 项):\n{}", failures.len(), failures.join("\n"));
    }
}

fn run_case(dir: &std::path::Path) -> Result<(), String> {
    let input_text = fs::read_to_string(dir.join("input.json"))
        .map_err(|e| format!("读 input.json 失败: {e}"))?;
    let expected_text = fs::read_to_string(dir.join("expected.json"))
        .map_err(|e| format!("读 expected.json 失败: {e}"))?;

    let case: GoldenCase = serde_json::from_str(&input_text)
        .map_err(|e| format!("解析 input.json 失败: {e}"))?;
    let expected: Expected = serde_json::from_str(&expected_text)
        .map_err(|e| format!("解析 expected.json 失败: {e}"))?;

    if expected.meta.source.is_empty() {
        return Err("meta.source 不可为空（黄金值纪律）".into());
    }

    // 构建骨架
    let mut sk = Skeleton::from_data(&case.input.skeleton.bones);
    sk.update_world();
    sk.precompute_bind_inverse();

    // 应用 pose override（动画模拟）
    if !case.input.pose.is_empty() {
        assert_eq!(
            case.input.pose.len(), sk.bones.len(),
            "pose 长度 != 骨骼数"
        );
        for (i, p) in case.input.pose.iter().enumerate() {
            sk.bones[i].local = *p;
        }
        sk.update_world();
    }

    // 比对 bones
    for (i, bo) in expected.outputs.bones.iter().enumerate() {
        let w = &sk.bones[i].world;
        let actual = [w.a, w.b, w.c, w.d, w.wx, w.wy];
        for (j, (&a, &e)) in actual.iter().zip(bo.world.iter()).enumerate() {
            if !approx(a, e, TRANSFORM_TOL) {
                return Err(format!(
                    "bone[{}] '{}' world[{}] 实际 {:.6} ≠ 期望 {:.6} (容差 {})",
                    i, bo.name, j, a, e, TRANSFORM_TOL
                ));
            }
        }
    }

    // 比对 vertices
    for (i, v_in) in case.input.vertices.iter().enumerate() {
        let v = v_in.to_vertex();
        let out = skin_vertex(&v, &sk, Vec2::ZERO);
        let exp = &expected.outputs.vertices[i];
        if !approx(out.x, exp.x, VERTEX_TOL) {
            return Err(format!("vertex[{i}] x 实际 {:.6} ≠ 期望 {:.6}", out.x, exp.x));
        }
        if !approx(out.y, exp.y, VERTEX_TOL) {
            return Err(format!("vertex[{i}] y 实际 {:.6} ≠ 期望 {:.6}", out.y, exp.y));
        }
    }

    Ok(())
}
