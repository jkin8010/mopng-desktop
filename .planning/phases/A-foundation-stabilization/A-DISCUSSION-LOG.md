# Phase A: Foundation Stabilization - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** A-Foundation Stabilization
**Areas discussed:** SHA256 校验与回退策略, 模型 URL 配置整合, ORT 性能优化级别, CI 工作流修复范围, 错误处理与用户反馈, Mutex 拆分的并发细节

---

## SHA256 校验与回退策略

| Option | Description | Selected |
|--------|-------------|----------|
| 下载后校验 + 失败自动回退 | 下载完成 → SHA256 校验 → 失败则自动尝试下一个源（ModelScope→HF Mirror→HuggingFace），无需用户干预。校验和写入 ModelDescriptor。每次 init_model 也做快速校验。 | ✓ |
| 仅下载后校验，失败通知用户 | 下载完成 → SHA256 校验 → 失败弹出提示让用户手动选择其他源。init_model 不做校验。 | |
| 多级校验：下载+加载都校验 | 下载后校验 + init_model 时也校验。最安全但每次启动多一次文件 hash 计算。 | |

**User's choice:** 下载后校验 + 失败自动回退（推荐）
**Notes:** 校验和存在 ModelDescriptor 中，init_model 时也做快速校验

---

## 模型 URL 配置整合

| Option | Description | Selected |
|--------|-------------|----------|
| 模型描述符统一管理 | 所有 URL 定义在 ModelDescriptor.sources 中。.env 只保留用户级覆盖。CI 从 descriptor 读取。 | ✓ |
| 纯配置文件 | 创建 src-tauri/models.conf.toml，所有 URL 集中管理。环境变量只覆盖配置文件路径。 | |
| 保持编译时 + 运行时双层 | 维持当前架构但集中到一个文件消除重复。 | |

**User's choice:** 模型描述符统一管理（推荐）
**Notes:** 优先级：环境变量 > descriptor 默认源 > descriptor 备用源

---

## ORT 性能优化级别

| Option | Description | Selected |
|--------|-------------|----------|
| Basic + 单线程（安全优先） | GraphOptimizationLevel::Basic + intra_threads 保持 1。性能有小幅提升，不影响模型行为。 | ✓ |
| Extended + 多线程 | Extended 含布局优化和内存优化 + intra_threads 改为 num_cpus。可能显著提速但有结果不一致风险。 | |
| 先 Benchmark 再决定 | 写脚本对比四种级别的推理速度和结果一致性，数据驱动决策。 | |

**User's choice:** Basic + 单线程（推荐，安全优先）
**Notes:** Phase B/C 积累推理数据后再评估更高级别

---

## CI 工作流修复范围

| Option | Description | Selected |
|--------|-------------|----------|
| 全部修复 | npm→pnpm + ORT 缓存 + CDN URL 整合。一次性清理干净。 | ✓ |
| 仅修复关键问题 | 只做 npm→pnpm + ORT 缓存，CDN URL 暂不改动。 | |
| 最小修复 | 只修 npm→pnpm，缓存和 URL 留到后续阶段。 | |

**User's choice:** 全部修复（推荐）
**Notes:** ORT 缓存用 GitHub Actions cache，cache key: `ort-{platform}-{version}`

---

## 错误处理与用户反馈

### 下载全部失败

| Option | Description | Selected |
|--------|-------------|----------|
| 错误对话框 + 手动重试 | 弹窗列出每个源的具体错误，提供重试和手动输入 URL 按钮。 | ✓ |
| 静默失败 + 状态标记 | 不弹窗，仅标记 error 状态，hover 看详情。 | |
| 通知 + 自动暂停 | OS 原生通知，模型对话框保持打开。 | |

**User's choice:** 错误对话框 + 手动重试（推荐）

### init_model 校验失败

| Option | Description | Selected |
|--------|-------------|----------|
| 自动重新下载 | 自动删除损坏文件并从第一个源重新下载，全程静默。 | |
| 提示 + 手动确认 | 弹窗"模型文件已损坏，是否重新下载？"，用户确认后自动重下。 | ✓ |
| 标记错误 + 手动操作 | 标记 error 状态，用户需手动进入设置重新下载。 | |

**User's choice:** 提示 + 手动确认
**Notes:** 给用户一个取消的机会（比如想手动替换文件）

---

## Mutex 拆分的并发细节

### init_model 加载策略

| Option | Description | Selected |
|--------|-------------|----------|
| 锁外加载 + 两阶段提交 | 读锁验证 → 锁外 init → 写锁替换。不阻塞 list_models()。 | |
| 锁内加载 | 保持当前模式，简单无竞态但阻塞 1-2 秒。 | |
| 异步加载 + 状态机 | init_model 立即返回 loading 状态，后台线程加载，状态自动更新。完全不阻塞。 | ✓ |

**User's choice:** 异步加载 + 状态机
**Notes:** 前端通过 list_models() 轮询感知状态变化

### infer() 取消支持

| Option | Description | Selected |
|--------|-------------|----------|
| 暂不需要 | 单次推理 3-5 秒，取消收益不大，Phase B 再评估。 | ✓ |
| 需要取消支持 | 添加 CancellationToken，UI 可取消推理。 | |

**User's choice:** 暂不需要（推荐）

---

## Claude's Discretion

- 代码清理范围（dead code 移除、重复代码合并、WebP/PNG 质量修复）
- Mutex 拆分的具体实现（RwLock 粒度、状态转换逻辑）
- CI 缓存的具体 cache key 设计
- 错误对话框的具体 UI 实现

## Deferred Ideas

- 模型池（多 session 并发推理）— Phase B/C
- infer() 取消支持 — Phase B
- GraphOptimizationLevel Extended/All — Phase B/C after benchmark
- intra_threads 多线程 — Phase B/C after benchmark

---

## Session 2: 实施后审查 (2026-05-01)

**Areas discussed:** 实现完成度审查, A-04 前端适配细节, 上下文更新方式

### 实现完成度审查

| Option | Description | Selected |
|--------|-------------|----------|
| 审查 A-01~A-04 实现状态 | 对比 plan 文件与实际代码 | ✓ |
| 跳过 | | |

**User's choice:** 审查实现状态
**Notes:** 全部 4 个 plans 确认为完整实现。A-01: RwLock+Mutex+Mutex 三层锁架构。A-02: SHA256 流式校验+自动回退+DownloadErrorResponse。A-03: CI pnpm 迁移+ORT 缓存。A-04: TS 类型+App.tsx 轮询+ModelDialog 错误展示。

### A-04 前端适配细节

| Option | Description | Selected |
|--------|-------------|----------|
| 讨论 A-04 细节 | 异步轮询、错误展示、重试/手动 URL | ✓ |
| 跳过 | | |

**User's choice:** 讨论 A-04 细节
**Notes:** grep 验证全部通过：ModelInfo.state/checksum, ModelStatus.state, SourceError/DownloadErrorResponse, init_model fire-and-forget, setInterval 500ms 轮询, download_model({ modelId }) 新签名, DownloadErrorResponse JSON 解析, source_errors 渲染, 重试+手动 URL。

### 上下文更新方式

| Option | Description | Selected |
|--------|-------------|----------|
| 全面重写 | 基于实施后状态完整重写 | |
| 增量标注 | 保留现有结构，标注 [已实现]，更新 code_context | ✓ |
| 直接查看现有 | 先看再决定 | ✓ (先查看，再选增量标注) |

**User's choice:** 增量标注
**Notes:** Status→Implemented, D-01~D-15 标注✅, code_context 更新为实施后模式, canonical_refs 新增 rmbg.rs/descriptor.rs, deferred 移除 infer 取消项（Phase B 已覆盖）。

### Claude's Discretion (Session 2)

无需自主决策 — 所有方向由用户选择。

### Deferred Ideas (Session 2 新增)

- 死代码清理（useTauri.ts）— 非 Phase A 范围
- WebP/PNG 质量设置修复 — 非 Phase A 范围
