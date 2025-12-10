# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

byebyecode 是一个基于 Rust 的高性能 Claude Code 状态栏工具，基于 CCometixLine 实现。主要功能包括：

- **状态栏生成**：显示 Git 分支状态、模型信息、目录、上下文窗口等
- **TUI 配置界面**：交互式主题和段落配置
- **Claude Code 增强**：禁用上下文警告、启用详细模式
- **API 集成**：88Code 和 ByeByeCode 用量监控

## 开发命令

```bash
# 构建开发版本
cargo build

# 运行测试
cargo test

# 构建发布版本（优化大小）
cargo build --release

# 运行（从 stdin 读取 JSON）
echo '{"model":"claude-3-5-sonnet","cwd":"/path"}' | cargo run

# 测试 TUI 配置界面
cargo run -- --config

# 测试初始化配置
cargo run -- --init
```

## 核心架构

### 模块结构

```
src/
├── main.rs              # CLI 入口，处理命令行参数和主流程
├── lib.rs               # 库入口，导出公共模块
├── cli.rs               # clap 命令行参数定义
├── core/
│   ├── statusline.rs    # StatusLineGenerator - 状态栏渲染核心
│   └── segments/        # 各类段落的数据收集器
│       ├── mod.rs       # Segment trait 定义
│       ├── git.rs       # Git 分支/状态段落
│       ├── model.rs     # 模型名称段落
│       ├── directory.rs # 目录段落
│       └── ...          # 其他段落
├── config/
│   ├── models.rs        # Config、SegmentConfig 等结构体
│   ├── types.rs         # InputData、SegmentId 等类型
│   ├── loader.rs        # TOML 配置加载器
│   └── defaults.rs      # 默认配置值
├── ui/
│   ├── app.rs           # TUI 主应用状态
│   ├── main_menu.rs     # 主菜单界面
│   ├── components/      # TUI 组件（编辑器、颜色选择器等）
│   └── themes/          # 预设主题定义
├── api/                 # 88Code/ByeByeCode API 客户端
├── utils/
│   ├── claude_code_patcher.rs  # cli.js 补丁工具
│   └── credentials.rs          # 凭证读取
└── wrapper/             # Claude Code 包装器（查找/启动 Claude）
```

### 数据流

1. **Claude Code 调用** → stdin 传入 JSON（`InputData`）
2. **配置加载** → 从 `~/.claude/byebyecode/config.toml` 读取
3. **段落收集** → 各 `Segment::collect()` 提取数据
4. **状态栏生成** → `StatusLineGenerator::generate()` 渲染 ANSI 输出
5. **stdout 输出** → Claude Code 显示状态栏

### 关键类型

- `Config` - 完整配置结构，包含所有段落和样式设置
- `SegmentConfig` - 单个段落的配置（颜色、图标、启用状态）
- `SegmentData` - 段落收集的数据（primary/secondary 文本 + metadata）
- `InputData` - Claude Code 传入的 JSON 数据
- `StatusLineGenerator` - 状态栏渲染器，处理 ANSI 颜色和分隔符

## Cargo Features

```toml
[features]
default = ["tui", "self-update", "dirs"]
tui = ["ratatui", "crossterm", "ansi_term", "ansi-to-tui", "chrono"]  # TUI 界面
self-update = ["ureq", "semver", "chrono", "dirs"]                    # 自动更新
```

## npm 发布结构

```
npm/
├── main/                # 主包 @88code/byebyecode
│   ├── package.json
│   ├── bin/byebyecode.js    # 入口脚本，查找平台二进制
│   └── scripts/postinstall.js
└── platforms/           # 各平台二进制包
    ├── darwin-arm64/
    ├── darwin-x64/
    ├── linux-x64/
    ├── linux-x64-musl/
    └── win32-x64/
```

## 配置文件位置

- 配置目录：`~/.claude/byebyecode/`
- 主配置：`~/.claude/byebyecode/config.toml`
- 主题目录：`~/.claude/byebyecode/themes/`
- 旧配置目录 `~/.claude/88code/` 会自动迁移

## 添加新段落

1. 在 `src/config/types.rs` 的 `SegmentId` 枚举添加新 ID
2. 在 `src/core/segments/` 创建新段落模块，实现 `Segment` trait
3. 在 `src/core/segments/mod.rs` 导出新模块
4. 在 `src/core/statusline.rs` 的 `collect_all_segments()` 添加匹配分支
5. 在 `src/config/defaults.rs` 添加默认配置

## 注意事项

- 状态栏输出必须是单行，Claude Code 按行解析
- ANSI 颜色序列需正确闭合（`\x1b[0m` 重置）
- Powerline 箭头需要 Nerd Font 支持
- TUI 功能通过 feature flag 控制，可选编译
