//! 约束黄金值测试（docs 12b-黄金值测试-规范，B6）。
//!
//! 支持 IK / Transform（单次求解，dt=0）+ Physics（多帧序列）。
//! 用例位于 tests/golden/constraints/<name>/{input.json, expected.json}。

use lp_constraints::{solve_pipeline, Constraint, PhysicsStateMap};
use lp_core::math::world_rotation;
use lp_core::skeleton::{BoneData, Skeleton};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct GoldenCase {
    input: CaseInput,
}

#[derive(Deserialize)]
struct CaseInput {
    skeleton: SkeletonInput,
    constraint: ConstraintJson,
    #[serde(default)]
    frames: u32, // Physics 多帧序列：推进帧数（0=单次，IK/Transform 用 0）
}

#[derive(Deserialize)]
struct SkeletonInput {
    bones: Vec<BoneData>,
}

/// 约束 JSON（内联，避免 serde tag 歧义）。
#[derive(Deserialize)]
struct ConstraintJson {
    #[serde(rename = "type")]
    ctype: String,
    bones: Vec<usize>,
    target: Option<[f32; 2]>,
    source: Option<usize>,
    #[serde(default)]
    mix: f32,
    #[serde(default = "default_bend")]
    bend_direction: i8,
    #[serde(default)]
    softness: f32,
    #[serde(default)]
    stretch: f32,
    // physics
    #[serde(default)]
    bone_inertia: f32,
    #[serde(default)]
    strength: f32,
    #[serde(default)]
    damping: f32,
    #[serde(default)]
    gravity: Option<[f32; 2]>,
    #[serde(default = "one")]
    mass: f32,
    #[serde(default)]
    translate_mix: f32,
}
fn default_bend() -> i8 { 1 }
fn one() -> f32 { 1.0 }

impl ConstraintJson {
    fn to_constraint(&self) -> Result<Constraint, String> {
        match self.ctype.as_str() {
            "ik" => Ok(Constraint::Ik(lp_constraints::IkConstraint {
                bones: self.bones.clone(),
                target: self.target.ok_or("ik 缺 target")?,
                mix: self.mix,
                bend_direction: self.bend_direction,
                softness: self.softness,
                stretch: self.stretch,
            })),
            "transform" => Ok(Constraint::Transform(lp_constraints::TransformConstraint {
                source: self.source.ok_or("transform 缺 source")?,
                bones: self.bones.clone(),
                offset_rotate: 0.0, offset_x: 0.0, offset_y: 0.0,
                offset_scale_x: 1.0, offset_scale_y: 1.0, offset_shear_x: 0.0,
                rotate_mix: self.mix, translate_mix: 0.0, scale_mix: 0.0, shear_mix: 0.0,
            })),
            "physics" => Ok(Constraint::Physics(lp_constraints::PhysicsConstraint {
                bone: self.bones[0],
                bone_inertia: self.bone_inertia,
                strength: self.strength,
                damping: self.damping,
                gravity: self.gravity.unwrap_or([0.0, -50.0]),
                mass: self.mass,
                angle_min: -10.0, angle_max: 10.0,
                rotate_mix: 1.0, translate_mix: self.translate_mix, translate_limit: 50.0,
            })),
            other => Err(format!("未知约束类型: {other}")),
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
    /// 单次（IK/Transform）：各骨骼 worldRotation。
    #[serde(default)]
    bones: Vec<BoneOut>,
    /// Physics 多帧：每帧的 bone.rotation 序列。
    #[serde(default)]
    frames: Vec<FrameOut>,
}

#[derive(Deserialize)]
struct BoneOut {
    rotation: f32,
}

#[derive(Deserialize)]
struct FrameOut {
    rotations: Vec<f32>,
}

#[derive(Deserialize)]
struct Meta {
    source: String,
    /// Physics 采样帧序号（1-based）。expected.frames 按此顺序对应。
    #[serde(default)]
    sampled_frames: Vec<u32>,
}

fn approx(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

fn collect_cases() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("tests/golden/constraints");
    let mut cases = vec![];
    if let Ok(entries) = fs::read_dir(&root) {
        for c in entries.flatten() {
            if c.path().join("input.json").exists() {
                cases.push(c.path());
            }
        }
    }
    cases
}

#[test]
fn constraint_golden_cases_pass() {
    let cases = collect_cases();
    let mut failures = Vec::new();
    for case_dir in &cases {
        let name = case_dir.file_name().unwrap().to_string_lossy();
        match run_case(case_dir) {
            Ok(()) => println!("  ✓ {name}"),
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }
    if !failures.is_empty() {
        panic!("约束黄金值失败 ({}):\n{}", failures.len(), failures.join("\n"));
    }
}

fn run_case(dir: &std::path::Path) -> Result<(), String> {
    use std::fs;
    let input_text = fs::read_to_string(dir.join("input.json"))
        .map_err(|e| format!("读 input.json: {e}"))?;
    let expected_text = fs::read_to_string(dir.join("expected.json"))
        .map_err(|e| format!("读 expected.json: {e}"))?;
    let case: GoldenCase = serde_json::from_str(&input_text)
        .map_err(|e| format!("解析 input.json: {e}"))?;
    let expected: Expected = serde_json::from_str(&expected_text)
        .map_err(|e| format!("解析 expected.json: {e}"))?;
    if expected.meta.source.is_empty() {
        return Err("meta.source 不可为空".into());
    }

    let constraint = case.input.constraint.to_constraint()?;
    let mut sk = Skeleton::try_from_data(&case.input.skeleton.bones)
        .map_err(|e| format!("骨骼校验: {e}"))?;
    sk.update_world();

    let dt = 1.0_f32 / 60.0;
    let frames = case.input.frames.max(1);
    let mut states = PhysicsStateMap::new();

    if case.input.frames == 0 {
        // 单次（IK/Transform）：dt=0
        solve_pipeline(&mut sk, std::slice::from_ref(&constraint), &mut states, 0.0);
        for (i, bo) in expected.outputs.bones.iter().enumerate() {
            let actual = world_rotation(&sk.bones[i].world);
            if !approx(actual, bo.rotation, 1e-3) {
                return Err(format!("bone[{i}] rotation 实际 {actual:.4} ≠ 期望 {:.4}", bo.rotation));
            }
        }
    } else {
        // Physics 多帧：推进到 sampled_frames 的每个采样帧，比对 expected.frames
        let sampled = &expected.meta.sampled_frames;
        if sampled.len() != expected.outputs.frames.len() {
            return Err(format!(
                "sampled_frames({}) 与 frames({}) 数量不一致",
                sampled.len(), expected.outputs.frames.len()
            ));
        }
        let mut sample_idx = 0;
        for frame in 1..=frames {
            solve_pipeline(&mut sk, std::slice::from_ref(&constraint), &mut states, dt);
            sk.update_world();
            if sample_idx < sampled.len() && frame == sampled[sample_idx] {
                let frame_exp = &expected.outputs.frames[sample_idx];
                for (i, &exp_rot) in frame_exp.rotations.iter().enumerate() {
                    let actual = world_rotation(&sk.bones[i].world);
                    if !approx(actual, exp_rot, 1e-2) {
                        return Err(format!(
                            "frame[{frame}] bone[{i}] rotation 实际 {actual:.4} ≠ 期望 {exp_rot:.4}"
                        ));
                    }
                }
                sample_idx += 1;
            }
        }
        if sample_idx != sampled.len() {
            return Err(format!("采样帧未全部命中（命中 {sample_idx}/{}）", sampled.len()));
        }
    }
    Ok(())
}

use std::fs;
