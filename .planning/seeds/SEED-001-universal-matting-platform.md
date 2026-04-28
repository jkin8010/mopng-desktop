---
id: SEED-001
status: dormant
planted: 2026-04-28
planted_during: develop (post-v0.3.2)
trigger_when: After plugin system foundations
scope: Large
---

# SEED-001: 通用抠图平台 — 插件化架构与工业化应用

## Why This Matters

**Expand market reach** — 当前项目专注于 BiRefNet 单模型抠图。通过插件化架构支持各类抠图模型（BiRefNet、RMBG、U2Net、SAM 等），同时保持统一的用户界面和操作习惯，可以覆盖更广泛的用户群体。插件系统还能支撑抠图周边场景（制作谷子/徽章、精确测量尺标、工业化批量生产），使产品从"工具"升级为"平台"，显著扩大市场边界。

## When to Surface

**Trigger:** After plugin system foundations

This seed should be presented during `/gsd-new-milestone` when the milestone scope matches any of these conditions:
- 开始设计或实现插件系统基础设施
- 准备支持第二个模型（当前仅有 BiRefNet）
- 讨论架构重构以支持多模型
- 规划工业化/批量化功能

## Scope Estimate

**Large** — 这是一个完整的里程碑级别工作，涉及：

1. **插件协议设计** — 定义模型插件接口（模型加载、预处理、推理、后处理）
2. **插件发现与加载机制** — 动态发现、热加载、版本管理
3. **统一 UI 适配层** — 不同模型通过插件暴露参数，UI 自动生成控制面板
4. **周边应用插件** — 谷子/徽章制作（模板、排版、出血线）、测量尺标工具
5. **工业化生产支持** — 批量处理优化、色彩管理（CMYK）、打印规格导出

## Breadcrumbs

当前代码库中相关的文件和决策：

- `src/App.tsx` — 主应用结构，当前为单模型设计
- `src/components/ModelDialog.tsx` — 模型选择对话框，当前仅支持 ModelScope 单一下载源
- `src/components/ControlPanel.tsx` — 控制面板，尺寸模板、背景色等
- `src/components/konva/mattingEngine.ts` — Konva 抠图引擎，渲染管线核心
- `src/components/PreviewCanvas.tsx` — 预览画布
- `src/types/index.ts` — 类型定义
- `src/components/TaskBar.tsx` — 任务栏/工具栏
- `src/components/BatchProgress.tsx` — 批量处理进度（工业化基础）
- `src-tauri/src/models/` — Rust 后端模型定义
- `src-tauri/src/commands/` — Rust 命令层

## Notes

- 当前阶段：刚完成 v0.3.2，BiRefNet 单模型抠图可用
- 用户已有批量处理 UI 基础（BatchProgress）
- Konva 渲染管线已建立，插件化时可复用
- 尺寸模板功能（含自定义/等比锁定）刚完成，可作为插件参数暴露的参考模式
- 模型下载当前硬编码 ModelScope 路径，插件化后需抽象为模型注册表
