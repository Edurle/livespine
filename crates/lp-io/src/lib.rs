//! Livepine 序列化层：最小 `.lp`（JSON）读写。
//!
//! P0 只需支撑 CLI 加载：骨架数据（BoneData 列表）+ region 附件。
//! 完整格式（约束/动画/皮肤等）留待对应阶段。
//!
//! 格式见 docs 06-数据模型与序列化。

use lp_core::attach::RegionAttachment;
use lp_core::skeleton::{BoneData, Skeleton};
use serde::{Deserialize, Serialize};

/// `.lp` 文件顶层结构（P0 最小集）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LpFile {
    pub format: String,
    pub version: String,
    pub skeleton: SkeletonDef,
    #[serde(default)]
    pub regions: Vec<RegionAttachment>,
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

    /// 基本校验（见 docs 06 §6.4）。前序排列由 Skeleton::from_data 再校验。
    pub fn validate(&self) -> Result<(), LpError> {
        if self.format != "lp" {
            return Err(LpError::Invalid("format 字段必须为 'lp'".into()));
        }
        for r in &self.regions {
            r.validate_weights().map_err(LpError::Invalid)?;
        }
        Ok(())
    }

    /// 构建运行时骨架实例（已 update_world + precompute_bind_inverse）。
    pub fn build_skeleton(&self) -> Skeleton {
        let mut sk = Skeleton::from_data(&self.skeleton.bones);
        sk.update_world();
        sk.precompute_bind_inverse();
        sk
    }
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
        };
        let json = f.to_json_pretty().unwrap();
        let back = LpFile::from_json(&json).unwrap();
        assert_eq!(back.skeleton.bones.len(), 1);
        assert_eq!(back.regions.len(), 1);
    }
}
