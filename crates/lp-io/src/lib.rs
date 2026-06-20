//! Livepine 序列化层：最小 `.lp`（JSON）读写。
//!
//! P0 只需支撑 CLI 加载：骨架数据（BoneData 列表）+ region 附件。
//! 完整格式（约束/动画/皮肤等）留待对应阶段。
//!
//! 格式见 docs 06-数据模型与序列化。

use lp_anim::Animation;
use lp_constraints::Constraint;
use lp_core::attach::RegionAttachment;
use lp_core::skeleton::{BoneData, Skeleton};
use serde::{Deserialize, Serialize};

/// `.lp` 文件顶层结构。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LpFile {
    pub format: String,
    pub version: String,
    pub skeleton: SkeletonDef,
    #[serde(default)]
    pub regions: Vec<RegionAttachment>,
    #[serde(default)]
    pub animations: Vec<Animation>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
}

/// 骨架定义（P0：仅骨骼数组）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkeletonDef {
    pub bones: Vec<BoneData>,
}

impl LpFile {
    /// 解析 `.lp` JSON 文本。
    pub fn from_json(text: &str) -> Result<Self, LpError> {
        let f: Self = serde_json::from_str(text)?;
        f.validate()?;
        Ok(f)
    }

    /// 序列化为 JSON 文本（美化）。
    pub fn to_json_pretty(&self) -> Result<String, LpError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// 从文件加载。
    pub fn load(path: &std::path::Path) -> Result<Self, LpError> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json(&text)
    }

    /// 基本校验（见 docs 06 §6.4）。前序排列/无环由 Skeleton::try_from_data 校验。
    pub fn validate(&self) -> Result<(), LpError> {
        if self.format != "lp" {
            return Err(LpError::Invalid("format 字段必须为 'lp'".into()));
        }
        // 名字唯一性（bone / animation）—— AI 按名操作，重名会取第一个导致行为不可预期
        check_unique_names(
            self.skeleton.bones.iter().map(|b| b.name.as_str()),
            "骨骼",
        )?;
        check_unique_names(
            self.animations.iter().map(|a| a.name.as_str()),
            "动画",
        )?;
        for r in &self.regions {
            r.validate_weights().map_err(LpError::Invalid)?;
        }
        for a in &self.animations {
            a.validate().map_err(LpError::Invalid)?;
        }
        for c in &self.constraints {
            c.validate().map_err(LpError::Invalid)?;
        }
        Ok(())
    }

    /// 按名查找动画。
    pub fn find_anim(&self, name: &str) -> Option<&Animation> {
        self.animations.iter().find(|a| a.name == name)
    }

    /// 构建运行时骨架实例（已 update_world + precompute_bind_inverse）。
    ///
    /// 校验失败（parent 越界/成环）返回错误，不 panic。
    pub fn build_skeleton(&self) -> Result<Skeleton, LpError> {
        let mut sk = Skeleton::try_from_data(&self.skeleton.bones)
            .map_err(LpError::Invalid)?;
        sk.update_world();
        sk.precompute_bind_inverse();
        Ok(sk)
    }
}

/// 校验名字唯一性。重复返回错误（指出重复的名字）。
fn check_unique_names<'a, I: Iterator<Item = &'a str>>(
    names: I,
    kind: &str,
) -> Result<(), LpError> {
    let mut seen = std::collections::HashSet::new();
    for name in names {
        if !seen.insert(name) {
            return Err(LpError::Invalid(format!("{kind}名字重复: '{name}'")));
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum LpError {
    #[error("JSON 解析失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("数据无效: {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use lp_core::math::BoneLocal;

    #[test]
    fn roundtrip_minimal() {
        let f = LpFile {
            format: "lp".into(),
            version: "0.1.0".into(),
            skeleton: SkeletonDef {
                bones: vec![BoneData {
                    name: "root".into(), parent: None, length: 10.0,
                    setup: BoneLocal::DEFAULT,
                }],
            },
            regions: vec![RegionAttachment::centered("rect", 0, 4.0, 2.0)],
            animations: vec![],
            constraints: vec![],
        };
        let json = f.to_json_pretty().unwrap();
        let back = LpFile::from_json(&json).unwrap();
        assert_eq!(back.skeleton.bones.len(), 1);
        assert_eq!(back.regions.len(), 1);
    }

    #[test]
    fn find_anim_works() {
        use lp_anim::{Animation, Keyframe, Property, Timeline};
        use lp_core::math::BoneLocal;
        let f = LpFile {
            format: "lp".into(),
            version: "0.1.0".into(),
            skeleton: SkeletonDef {
                bones: vec![BoneData {
                    name: "root".into(), parent: None, length: 10.0,
                    setup: BoneLocal::DEFAULT,
                }],
            },
            regions: vec![],
            animations: vec![Animation {
                name: "wave".into(),
                duration: 1.0,
                timelines: vec![Timeline {
                    bone: 0, property: Property::Rotate,
                    keyframes: vec![
                        Keyframe { time: 0.0, value: 0.0, curve: lp_anim::Curve::LINEAR },
                        Keyframe { time: 1.0, value: 1.0, curve: lp_anim::Curve::LINEAR },
                    ],
                }],
            }],
            constraints: vec![],
        };
        assert!(f.find_anim("wave").is_some());
        assert!(f.find_anim("nope").is_none());
    }

    #[test]
    fn duplicate_bone_name_rejected() {
        let f = LpFile {
            format: "lp".into(),
            version: "0.1.0".into(),
            skeleton: SkeletonDef {
                bones: vec![
                    BoneData { name: "root".into(), parent: None, length: 1.0, setup: BoneLocal::DEFAULT },
                    BoneData { name: "root".into(), parent: Some(0), length: 1.0, setup: BoneLocal::DEFAULT },
                ],
            },
            regions: vec![],
            animations: vec![],
            constraints: vec![],
        };
        let err = f.validate();
        assert!(err.is_err(), "重复骨骼名应被拒绝");
        assert!(err.unwrap_err().to_string().contains("骨骼名字重复"));
    }

    #[test]
    fn duplicate_anim_name_rejected() {
        let f = LpFile {
            format: "lp".into(),
            version: "0.1.0".into(),
            skeleton: SkeletonDef {
                bones: vec![BoneData { name: "a".into(), parent: None, length: 1.0, setup: BoneLocal::DEFAULT }],
            },
            regions: vec![],
            animations: vec![
                Animation { name: "wave".into(), duration: 1.0, timelines: vec![] },
                Animation { name: "wave".into(), duration: 1.0, timelines: vec![] },
            ],
            constraints: vec![],
        };
        let err = f.validate();
        assert!(err.is_err(), "重复动画名应被拒绝");
        assert!(err.unwrap_err().to_string().contains("动画名字重复"));
    }
}
