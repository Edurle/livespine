# Livepine

> 开源的 2D 骨骼动画系统，功能对标 Spine。核心是一个纯数学/数据内核，与 UI/渲染/引擎解耦；AI 可操作性是一等公民。

当前阶段：**P0 内核地基**（见 [设计文档](../Obsidian%20Vault/Livepine%20项目规划/00-项目规划总览.md)）。

## 结构

```
crates/
├── lp-core/   # ★ 纯数学 + 数据模型（无 I/O，零外部运行时依赖）
├── lp-io/     # .lp JSON 读写
└── lp-cli/    # 命令行（P0: solve 子命令）
```

## 开发

```bash
cargo test                 # 跑全部测试（含黄金值）
cargo run -p lp-cli -- solve tests/golden/transforms/parent_chain/input.json
```
