# Phase A: Foundation Stabilization - Context

**Gathered:** 2026-04-28
**Updated:** 2026-05-01（实施后审查）
**Status:** Implemented — 全部 4 个 plans 已执行完毕，待验证

<domain>
## Phase Boundary

Fix global Mutex contention in registry so frontend polling isn't blocked during inference; add SHA256 download verification with automatic source fallback; cache ORT dependencies in CI; consolidate model URL configuration to a single source of truth.

Delivers REG-08, REG-09, REG-10.
</domain>

<decisions>
## Implementation Decisions

### SHA256 校验与回退策略
- **D-01:** 下载完成后立即 SHA256 校验，失败自动回退链：ModelScope → HF Mirror → HuggingFace ✅ 已实现
- **D-02:** 校验和存储在每个模型的 ModelDescriptor 中，编译时内置 ✅ 已实现
- **D-03:** init_model 时也做快速 SHA256 校验，防止磁盘文件损坏导致推理崩溃 ✅ 已实现

### 模型 URL 配置整合
- **D-04:** 所有模型 URL（默认源 + 备用源）统一在 ModelDescriptor.sources 中管理，download.rs 的编译时常量、birefnet.rs 的硬编码 URL、.env.example 的默认值全部移除 ✅ 已实现
- **D-05:** .env 仅保留 MODEL_URL/MODEL_FILENAME 作为用户级运行时覆盖（优先级：环境变量 > descriptor 默认源 > descriptor 备用源）✅ 已实现
- **D-06:** CI release.yml 的 CDN URL 改为从模型描述符读取，不再硬编码 ✅ 已实现

### ORT 性能优化
- **D-07:** GraphOptimizationLevel 从 Disable 改为 Basic（常量折叠、节点消除等安全优化，不改变模型行为）✅ 已实现 — 实际使用了 Level1
- **D-08:** intra_threads 保持 1（安全优先，Phase B/C 积累推理数据后再评估多线程）✅ 已实现

### CI 工作流修复
- **D-09:** build.yml 和 release.yml 中 `npm ci` / `cache: npm` → `pnpm install --frozen-lockfile` / `cache: pnpm` ✅ 已实现
- **D-10:** ORT 二进制用 GitHub Actions cache 缓存（cache key: `ort-macos-${{ runner.arch }}-v1.18.1`），避免 download-binaries 网络故障 ✅ 已实现
- **D-11:** release.yml 中 hardcoded `mocdn.mopng.cn` URL 改为从模型描述符读取 ✅ 已实现

### 错误处理与用户反馈
- **D-12:** 下载自动回退耗尽所有源 → 错误对话框列出每个源的具体错误（超时/404/校验失败），提供"重试"和"手动输入 URL"按钮 ✅ 已实现
- **D-13:** init_model SHA256 校验失败 → 弹窗"模型文件已损坏，是否重新下载？"，用户确认后自动删除损坏文件并从第一个源重新下载 ✅ 已实现

### 并发细节
- **D-14:** init_model 异步加载 + 状态机：命令立即返回（model 状态 = loading），后台线程加载 ONNX session，完成后状态自动更新。前端通过 list_models() 轮询感知状态变化，500ms 间隔 ✅ 已实现
- **D-15:** infer() 暂不支持取消（3-5 秒推理时间太短，取消复杂度收益不大）— Phase B 的 switch_model 通过 drop 旧模型间接处理了中断场景 ✅ 已由 Phase B 覆盖

### Claude's Discretion
- 代码清理范围（dead code 移除、重复代码合并、WebP/PNG 质量修复）— 未在 Phase A 处理，部分属于后续 phase
- Mutex 拆分：`RwLock<Vec<ModelDescriptor>>` + `Mutex<Option<LoadedModel>>` + `Mutex<HashMap<String, ModelState>>` ✅ 已实现
- CI 缓存：`ort-macos-${{ runner.arch }}-v1.18.1` ✅ 已实现
- 错误对话框和加载状态指示器 UI ✅ 已实现
- ORT download-binaries CI 缓存 ✅ 已实现
</decisions>

<canonical_refs>
## Canonical References

### 模型架构（已重构）
- `src-tauri/src/models/registry.rs` — RwLock<Vec<ModelDescriptor>> + Mutex<Option<LoadedModel>> + Mutex<HashMap<String, ModelState>> 三层锁架构
- `src-tauri/src/models/birefnet.rs` — BiRefNet 模型实现，GraphOptimizationLevel::Level1
- `src-tauri/src/models/rmbg.rs` — RMBG 1.4 模型实现（Phase B 新增，使用 Phase A 基础设施）
- `src-tauri/src/models/mod.rs` — MattingModel trait 定义（含 param_schema/capabilities 扩展）
- `src-tauri/src/models/descriptor.rs` — DescriptorJson 结构体，文件系统扫描用
- `src-tauri/src/commands/download.rs` — SHA256 流式校验 + 自动回退下载管线 + DownloadErrorResponse/SourceError 类型
- `.planning/codebase/ARCHITECTURE.md` — 整体架构和注册表模式
- `.planning/codebase/CONCERNS.md` — 已知问题清单（部分已在 Phase A 解决）

### CI/CD
- `.github/workflows/build.yml` — PR 验证构建（已迁移 pnpm + ORT 缓存）
- `.github/workflows/release.yml` — 生产构建 + macOS 签名公证（已迁移 pnpm + ORT 缓存，已移除硬编码 URL）

### 需求
- `.planning/REQUIREMENTS.md` — REG-08 (Mutex), REG-09 (SHA256), REG-10 (ORT CI)
- `.planning/ROADMAP.md` — Phase A 目标和成功标准
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets（实施后）
- 三层锁架构（RwLock + Mutex + Mutex）已是全局单例模式的标准范式，后续注册新模型直接使用
- `download_model_with_fallback` 回退链模式可复用于 Phase B 的多模型下载
- `DownloadErrorResponse` + `SourceError` 类型在 Rust 和 TypeScript 双端同步，成为下载错误的标准格式
- `scan_models_directory()` 文件系统扫描已在 Phase B 中作为插件发现机制使用
- `switch_model_with_dir` 使用了与 init_model 相同的异步加载 + 状态机模式，成为模型管理的统一范式

### Established Patterns（实施后）
- `RwLock<T>` 用于读多写少的描述符列表，`Mutex<T>` 用于互斥的活跃模型和状态表
- `std::thread::spawn` + `catch_unwind` 用于 ONNX session 的异步加载
- `setInterval` 500ms 轮询 `list_models()` 用于前端感知后端状态变更
- `invoke()` fire-and-forget 模式（.catch 链式调用）用于非阻塞异步操作

### Integration Points
- `registry.rs` 的 `init_model()` / `switch_model_with_dir()` → 异步状态机模式，前端通过轮询感知
- `download.rs` 的 `download_model_with_fallback()` → 回退链 + SHA256 校验，前端错误展示用 DownloadErrorResponse JSON
- `App.tsx` → `setInterval` 轮询 + `useStore.getState()` 避免闭包陈旧引用
- `ModelDialog.tsx` → 解析 DownloadErrorResponse JSON，按源错误展示 + 重试/手动 URL
</code_context>

<specifics>
## Specific Ideas

- 下载回退应该在下载层自动处理，不应要求前端逐个尝试源 — ✅ 已通过 `download_model_with_fallback` 实现，一次调用完整遍历回退链
- 错误对话框应列出每个源的具体错误原因，帮助用户判断是网络问题还是服务器问题 — ✅ 已实现 `source_errors` 渲染
- 模型状态机三态：`loading`（后台加载中，前端显示 spinner）→ `loaded`（就绪）→ `error`（加载失败，显示错误信息和恢复按钮）— ✅ 已实现，扩展为 `notDownloaded | loading | loaded | error` 四态
</specifics>

<deferred>
## Deferred Ideas

- 模型池（多 session 实例并发推理）— Phase B/C 性能评估后再决定
- GraphOptimizationLevel 进一步优化（Extended/All）— Phase B/C 积累 benchmark 数据后决定
- intra_threads 多线程 — Phase B/C benchmark 后决定
- 死代码清理（useTauri.ts）— 非 Phase A 范围，属于代码卫生清理
- WebP/PNG 质量设置修复 — 非 Phase A 范围

### Reviewed Todos (not folded)
- 无匹配的待办事项

</deferred>

---

*Phase: A-foundation-stabilization*
*Context gathered: 2026-04-28*
*Updated after implementation review: 2026-05-01*
