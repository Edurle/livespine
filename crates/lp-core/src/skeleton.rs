//! 模块 3：骨骼树 + 世界矩阵复合。
//!
//! 公式来源：docs 04-数学模型-变换与蒙皮-实现 §3, §5。
//! 纪律：骨骼数组必须**前序排列（父在子前）**，加载/构建时校验。
//! P0：每帧全量 `update_world`，不做 dirty 增量（那是 P3 的事）。

use crate::math::{local_to_affine, multiply, invert, Affine, BoneLocal};
use serde::{Deserialize, Serialize};

/// 骨骼模板数据（不可变，来自 SkeletonData）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoneData {
    pub name: String,
    /// 父骨骼索引。根骨骼为 None。
    pub parent: Option<usize>,
    pub length: f32,
    /// setup pose 的局部变换。
    pub setup: BoneLocal,
}

/// 运行时骨骼实例（可变状态）。
#[derive(Clone, Debug)]
pub struct Bone {
    pub local: BoneLocal,
    pub world: Affine,
    /// 绑定姿势世界矩阵的逆（蒙皮用，setup 后固定）。
    pub bind_world_inverse: Affine,
    /// 骨骙长度（来自 BoneData，IK 等约束用）。
    pub length: f32,
}

/// 骨架实例：骨骼数组 + parent 索引（与 BoneData 一一对应）。
#[derive(Clone, Debug)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    /// 与 bones 同长；bones[i] 的 parent 索引，根为 None。
    parents: Vec<Option<usize>>,
}

impl Skeleton {
    /// 从 BoneData 列表构建实例（失败则 panic）。
    ///
    /// 供测试/可信数据用。加载不可信数据请用 [`try_from_data`]（返回 Result）。
    /// 初始化时 local = setup，随后应调用 `update_world` + `precompute_bind_inverse`。
    pub fn from_data(data: &[BoneData]) -> Self {
        Self::try_from_data(data).expect("Skeleton::from_data 校验失败")
    }

    /// 从 BoneData 列表构建实例，校验失败返回错误（不 panic）。
    ///
    /// 校验内容（docs 06 §6.4）：
    /// - parent 索引不越界
    /// - 前序排列（父在子前）—— 等价于无环
    ///
    /// 加载不可信数据（如用户/AI 提供的 `.lp`）应使用此方法。
    pub fn try_from_data(data: &[BoneData]) -> Result<Self, String> {
        Self::validate(data)?;
        let bones = data.iter().map(|b| Bone {
            local: b.setup,
            world: Affine::IDENTITY,
            bind_world_inverse: Affine::IDENTITY,
            length: b.length,
        }).collect();
        let parents = data.iter().map(|b| b.parent).collect();
        Ok(Self { bones, parents })
    }

    /// 校验骨骼数据：parent 越界 + 前序排列（无环）。
    fn validate(data: &[BoneData]) -> Result<(), String> {
        for (i, b) in data.iter().enumerate() {
            if let Some(p) = b.parent {
                // 越界
                if p >= data.len() {
                    return Err(format!(
                        "骨骼 '{}' (idx {}) 的 parent idx {} 越界（共 {} 根骨骼）",
                        b.name, i, p, data.len()
                    ));
                }
                // 前序：父必须在子之前（p < i）。满足此条件即无环。
                if p >= i {
                    return Err(format!(
                        "骨骼 '{}' (idx {}) 的 parent idx {} 不在前序位置；骨骼数组必须父在子前（违反则成环）",
                        b.name, i, p
                    ));
                }
            }
        }
        Ok(())
    }

    /// 自顶向下算 world 矩阵（前序遍历）。
    ///
    /// 公式见 docs 04-实现 §3：`world[i] = world[parent] · local[i]`。
    /// 前序遍历保证算子骨骼时其 parent.world 已是最新。
    pub fn update_world(&mut self) {
        for i in 0..self.bones.len() {
            let local = self.bones[i].local;
            let new_world = match self.parents[i] {
                Some(p) => multiply(&self.bones[p].world, &local_to_affine(&local)),
                None => local_to_affine(&local),
            };
            self.bones[i].world = new_world;
        }
    }

    /// setup pose 后调用：缓存每根骨骼绑定姿势世界逆矩阵（蒙皮用）。
    ///
    /// 必须在 `update_world` 之后调用。
    pub fn precompute_bind_inverse(&mut self) {
        for b in &mut self.bones {
            b.bind_world_inverse = invert(&b.world);
        }
    }

    /// 取某骨骼的 parent 索引（调试/测试用）。
    pub fn parent_of(&self, i: usize) -> Option<usize> {
        self.parents[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn single_bone_world_equals_local() {
        let data = vec![BoneData {
            name: "root".into(), parent: None, length: 10.0,
            setup: BoneLocal { x: 5.0, y: 3.0, ..BoneLocal::DEFAULT },
        }];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();
        let expected = local_to_affine(&data[0].setup);
        let w = &sk.bones[0].world;
        assert!(approx(w.wx, expected.wx) && approx(w.wy, expected.wy));
    }

    #[test]
    fn parent_chain_composees() {
        // root 在 (10,0)，child 相对 root 平移 (5,0) → child 世界 (15,0)
        let data = vec![
            BoneData { name: "root".into(), parent: None, length: 10.0,
                setup: BoneLocal { x: 10.0, y: 0.0, ..BoneLocal::DEFAULT } },
            BoneData { name: "child".into(), parent: Some(0), length: 5.0,
                setup: BoneLocal { x: 5.0, y: 0.0, ..BoneLocal::DEFAULT } },
        ];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();
        let child_w = &sk.bones[1].world;
        assert!(approx(child_w.wx, 15.0), "child world x 应为 15, got {}", child_w.wx);
        assert!(approx(child_w.wy, 0.0));
    }

    #[test]
    fn parent_chain_with_rotation() {
        // root 旋转 90°，child 沿局部 +x 平移 5 → child 世界应在 root 上方 5
        let data = vec![
            BoneData { name: "root".into(), parent: None, length: 5.0,
                setup: BoneLocal { rotation: std::f32::consts::FRAC_PI_2, ..BoneLocal::DEFAULT } },
            BoneData { name: "child".into(), parent: Some(0), length: 5.0,
                setup: BoneLocal { x: 5.0, y: 0.0, ..BoneLocal::DEFAULT } },
        ];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();
        let child_w = &sk.bones[1].world;
        // root 转 90°：child 局部 (5,0) → 世界 (0,5)
        assert!(approx(child_w.wx.abs(), 0.0), "child world x≈0, got {}", child_w.wx);
        assert!(approx(child_w.wy, 5.0), "child world y=5, got {}", child_w.wy);
    }

    #[test]
    #[should_panic(expected = "前序")]
    fn preorder_violation_panics() {
        let data = vec![
            BoneData { name: "child".into(), parent: Some(1), length: 1.0, setup: BoneLocal::DEFAULT },
            BoneData { name: "root".into(), parent: None, length: 1.0, setup: BoneLocal::DEFAULT },
        ];
        let _ = Skeleton::from_data(&data);
    }

    #[test]
    fn try_from_data_cycle_returns_error() {
        // 成环（parent 指向自己或形成环）→ try_from_data 返回错误，不 panic
        let data = vec![
            BoneData { name: "child".into(), parent: Some(1), length: 1.0, setup: BoneLocal::DEFAULT },
            BoneData { name: "root".into(), parent: None, length: 1.0, setup: BoneLocal::DEFAULT },
        ];
        let result = Skeleton::try_from_data(&data);
        assert!(result.is_err(), "成环应返回错误");
        let err = result.unwrap_err();
        assert!(err.contains("前序") || err.contains("环"), "错误信息应提及前序/环: {err}");
    }

    #[test]
    fn try_from_data_parent_out_of_range_returns_error() {
        // parent 索引越界
        let data = vec![BoneData {
            name: "a".into(), parent: Some(5), length: 1.0, setup: BoneLocal::DEFAULT,
        }];
        let result = Skeleton::try_from_data(&data);
        assert!(result.is_err(), "parent 越界应返回错误");
        assert!(result.unwrap_err().contains("越界"));
    }

    #[test]
    fn bind_inverse_after_setup() {
        let data = vec![BoneData {
            name: "root".into(), parent: None, length: 10.0,
            setup: BoneLocal { x: 7.0, y: 2.0, ..BoneLocal::DEFAULT },
        }];
        let mut sk = Skeleton::from_data(&data);
        sk.update_world();
        sk.precompute_bind_inverse();
        // world · bind_world_inverse ≈ 单位阵
        let w = sk.bones[0].world;
        let bwi = sk.bones[0].bind_world_inverse;
        let prod = multiply(&w, &bwi);
        assert!(approx(prod.a, 1.0) && approx(prod.d, 1.0));
        assert!(approx(prod.wx.abs(), 0.0) && approx(prod.wy.abs(), 0.0));
    }
}
