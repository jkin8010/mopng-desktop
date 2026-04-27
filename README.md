# 🎨 MoPNG Desktop — 模图桌面版

基于 **Tauri + React + BiRefNet** 的本地 AI 抠图客户端，所有图片处理均在本地完成，无需上传云端，保护您的隐私与数据安全。

[![Version](https://img.shields.io/badge/version-0.2.0-blue)](https://github.com/jkin8010/mopng-desktop)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8D8?logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-18-61DAFB?logo=react)](https://react.dev)

> 💡 在线版请访问 [https://mopng.cn](https://mopng.cn)

---

## ✨ 功能特性

| 功能 | 状态 | 说明 |
|------|:----:|------|
| 🖼️ 批量图片导入 | ✅ | 支持拖放或文件选择，一次处理多张 |
| 🤖 AI 智能抠图 | ✅ | 基于 BiRefNet ONNX 模型，本地推理 |
| 🎭 前景/背景分离 | ✅ | 一键提取主体或移除背景 |
| 👁️ 实时预览 | ✅ | 支持原图 / 遮罩 / 透明 / 白底 / 黑底 |
| 📐 尺寸调整 | ✅ | 等比缩放或指定宽高 |
| 📦 批量导出 | ✅ | PNG / JPG / WebP 格式 |
| 🖥️ 无边框界面 | ✅ | 现代化毛玻璃 UI，沉浸式体验 |
| 🔒 完全本地 | ✅ | 图片不离开您的电脑 |

---

## 🖥️ 界面预览

<!-- TODO: 添加应用截图 -->

```
┌─────────────────────────────────────────────────────────────┐
│  MoPNG Desktop - 模图桌面版                            _ □ ✕ │
├──────────┬────────────────────────────────────┬─────────────┤
│          │                                    │   控制面板   │
│  缩略图   │         预览画布                    │  ┌─────────┐│
│  列表     │         (原图/透明/白底...)          │  │ 模式选择 ││
│          │                                    │  │ 前景提取 ││
│ [img1]   │                                    │  │ 背景移除 ││
│ [img2] ✓ │                                    │  └─────────┘│
│ [img3]   │                                    │  ┌─────────┐│
│ [img4]   │                                    │  │ 输出格式 ││
│          │                                    │  │ PNG ✓   ││
│          │                                    │  │ JPG     ││
│          │                                    │  │ WebP    ││
│          │                                    │  └─────────┘│
│          │                                    │  ┌─────────┐│
│          │                                    │  │ 背景设置 ││
│          │                                    │  │ 透明 ✓  ││
│          │                                    │  │ 白色    ││
│          │                                    │  │ 自定义色 ││
├──────────┴────────────────────────────────────┴─────────────┤
│  📁 导出路径: ~/Downloads/mopng-export    [导出全部] [开始处理]│
└─────────────────────────────────────────────────────────────┘
```

---

## 🚀 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) ≥ 18
- [Rust](https://www.rust-lang.org/) ≥ 1.70 (通过 [rustup](https://rustup.rs/) 安装)
- macOS 10.13+ / Windows 10+ / Linux

### 安装依赖

```bash
# 克隆仓库
git clone https://github.com/jkin8010/mopng-desktop.git
cd mopng-desktop

# 安装前端依赖
npm install

# Rust 依赖会在首次构建时自动安装
```

### 下载 ONNX 模型

> ⚠️ **首次使用前必须下载 BiRefNet ONNX 模型**

```bash
# 创建模型目录
mkdir -p ~/.mopng-desktop/models

# 下载模型 (约 400MB)
curl -L -o ~/.mopng-desktop/models/birefnet.onnx \
  "https://huggingface.co/onnx-community/BiRefNet_lite-ONNX/resolve/main/onnx/model.onnx"

# 或使用 Python 下载
python3 -c "
from huggingface_hub import hf_hub_download
hf_hub_download(repo_id='onnx-community/BiRefNet_lite-ONNX', filename='onnx/model.onnx',
                local_dir='~/.mopng-desktop/models')
"
```

### 开发模式

```bash
# 启动热重载开发服务器
npm run tauri dev
```

### 构建生产版本

```bash
# 构建发行版应用
npm run tauri build

# 输出位置:
# macOS: src-tauri/target/release/bundle/macos/MoPNG Desktop.app
# Windows: src-tauri/target/release/bundle/msi/
# Linux: src-tauri/target/release/bundle/deb/
```

---

## 🏗️ 项目结构

```
mopng-desktop/
├── 📄 package.json              # 前端依赖与脚本
├── 📄 vite.config.ts            # Vite 构建配置
├── 📄 tailwind.config.js        # Tailwind CSS 配置
│
├── 🖥️ src/                      # 前端 (React + TypeScript)
│   ├── App.tsx                  # 根组件
│   ├── main.tsx                 # 入口文件
│   ├── index.css                # 全局样式
│   │
│   ├── components/              # UI 组件
│   │   ├── TitleBar.tsx         # 自定义标题栏
│   │   ├── DropZone.tsx         # 拖放区域
│   │   ├── ThumbnailList.tsx    # 缩略图列表
│   │   ├── PreviewCanvas.tsx    # 预览画布
│   │   ├── ControlPanel.tsx     # 右侧控制面板
│   │   ├── TaskBar.tsx          # 底部任务栏
│   │   └── ui/                  # shadcn/ui 组件
│   │
│   ├── store/                   # 状态管理 (Zustand)
│   │   └── useStore.ts
│   │
│   ├── types/                   # TypeScript 类型
│   │   └── index.ts
│   │
│   ├── lib/                     # 工具函数
│   │   ├── utils.ts
│   │   └── id.ts
│   │
│   └── hooks/                   # 自定义 Hooks
│       └── useTauri.ts
│
├── ⚙️ src-tauri/                # 后端 (Rust + Tauri)
│   ├── Cargo.toml               # Rust 依赖
│   ├── tauri.conf.json          # Tauri 配置
│   ├── build.rs                 # 构建脚本
│   │
│   ├── src/
│   │   ├── main.rs              # 入口与命令注册
│   │   ├── commands/            # Tauri 命令
│   │   │   ├── mod.rs
│   │   │   ├── process.rs       # 图片处理
│   │   │   ├── file.rs          # 文件操作
│   │   │   └── export.rs        # 导出功能
│   │   └── models/              # 数据模型
│   │       ├── mod.rs
│   │       ├── session.rs       # 会话管理
│   │       └── birefnet.rs      # BiRefNet 模型封装
│   │
│   ├── icons/                   # 应用图标
│   └── capabilities/
│       └── default.json         # 权限配置
│
└── 📄 README.md                 # 本文档
```

---

## 🧩 技术栈

### 前端
- **[React 18](https://react.dev)** — UI 框架
- **[TypeScript](https://www.typescriptlang.org/)** — 类型安全
- **[Tailwind CSS](https://tailwindcss.com)** — 原子化样式
- **[shadcn/ui](https://ui.shadcn.com)** — 无障碍 UI 组件
- **[Zustand](https://zustand-demo.pmnd.rs)** — 轻量状态管理
- **[Lucide Icons](https://lucide.dev)** — 图标库

### 后端
- **[Tauri v2](https://tauri.app)** — Rust 桌面应用框架
- **[Rust](https://www.rust-lang.org/)** — 系统编程语言
- **[image](https://docs.rs/image)** — Rust 图像处理
- **[ndarray](https://docs.rs/ndarray)** — N 维数组计算

### AI 模型
- **[BiRefNet](https://github.com/ZhengPeng7/BiRefNet)** — 双边参考网络抠图模型
- **ONNX Runtime** — 跨平台推理引擎

---

## 📦 发布计划

| 版本 | 功能 | 时间 |
|:----:|------|:----:|
| v0.1.0 | 基础抠图、批量处理、导出 | ✅ 当前 |
| v0.2.0 | 模型自动下载、配置持久化 | ✅ 当前 |
| v0.3.0 | 批量背景替换、尺寸模板 | ✅ 当前 |
| v0.4.0 | 插件系统、自定义模型 | 📋 |
| v1.0.0 | 稳定版、多语言、自动更新 | 📋 |

---

## 🤝 相关项目

| 项目 | 描述 | 链接 |
|------|------|------|
| **MoPNG** | 在线 AI 修图工具 | [mopng.cn](https://mopng.cn) |
| **image-matting** | Rust 抠图核心库 (BiRefNet ONNX) | [GitHub](https://github.com/jkin8010/image-matting) |
| **BiRefNet** | 原始研究项目 | [GitHub](https://github.com/ZhengPeng7/BiRefNet) |

---

## 📄 许可证

本项目采用 [MIT License](LICENSE) 开源。

---

## 💬 反馈与支持

- 🐛 提交 Issue: [github.com/jkin8010/mopng-desktop/issues](https://github.com/jkin8010/mopng-desktop/issues)
- 📧 联系作者: jkin8010@163.com
- 🌐 在线版: [https://mopng.cn](https://mopng.cn)

---

<p align="center">
  Made with ❤️ by <a href="https://github.com/jkin8010">@jkin8010</a>
</p>
