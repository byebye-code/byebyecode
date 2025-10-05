# byebyecode

## 88code定制Claude Code 状态栏工具，基于CCometixLine实现

### 使用办法：
下载release二进制
在命令行执行 `byebyecode --init` 初始化配置文件
启动 Claude即可

基于 Rust 的高性能 Claude Code 状态栏工具，集成 Git 信息、使用量跟踪、交互式 TUI 配置和 Claude Code 补丁工具。

> **说明**: 这是一个增强版分支。特别感谢 [@cometix-ai](https://github.com/cometix-ai) 的[原始 CCometixLine 项目](https://github.com/cometix-ai/ccline)，为本项目提供了优秀的基础。

![Language:Rust](https://img.shields.io/static/v1?label=Language&message=Rust&color=orange&style=flat-square)
![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)

## 截图

![byebyecode](assets/img1.png)

状态栏显示：模型 | 目录 | Git 分支状态 | 上下文窗口信息

## 特性

### 核心功能
- **Git 集成** 显示分支、状态和跟踪信息
- **模型显示** 简化的 Claude 模型名称
- **使用量跟踪** 基于转录文件分析
- **目录显示** 显示当前工作空间
- **简洁设计** 使用 Nerd Font 图标

### 交互式 TUI 功能
- **交互式主菜单** 无输入时直接执行显示菜单
- **TUI 配置界面** 实时预览配置效果
- **主题系统** 多种内置预设主题
- **段落自定义** 精细化控制各段落
- **配置管理** 初始化、检查、编辑配置

### Claude Code 增强
- **禁用上下文警告** 移除烦人的"Context low"消息
- **启用详细模式** 增强输出详细信息
- **稳定补丁器** 适应 Claude Code 版本更新
- **自动备份** 安全修改，支持轻松恢复

### 🆕 最近更新 (v1.1.1)

#### 88Code API 集成
- **自动检测 API key** - 自动从 `~/.claude/settings.json` 读取
- **用量监控** - 状态栏实时显示 token 使用情况
- **订阅追踪** - 显示套餐类型、价格和到期状态

详细集成指南请参考 [README_88CODE_API.md](README_88CODE_API.md)。

#### ByeByeCode 集成
- **双向翻译** - 使用 GLM-4.5-Flash 实现中英文互译
- **实时监控** - 状态栏显示 byebyecode 套餐使用情况
- **自动配置** - 自动检测 Claude Code 路径

设置说明请参考 [README_BYEBYECODE.md](README_BYEBYECODE.md)。

#### 其他改进
- **代理支持** - 完整的 HTTP/HTTPS 代理配置功能
- **翻译模块** - 可扩展的翻译框架
- **自动配置工具** - 简化的安装配置流程
- **新增段落** - `byebyecode_usage`、`byebyecode_subscription`、`byebyecode_status`

## 安装

### 快速安装（推荐）

到release去下载最新版release，解压并运行

安装后：
- ✅ 全局命令 `byebyecode` 可在任何地方使用
- ⚙️ 按照下方提示进行配置以集成到 Claude Code
- 🎨 运行 `byebyecode -c` 打开配置面板进行主题选择



### 更新

<summary>手动安装（点击展开）</summary>

或者从 [Releases](https://github.com/byebye-code/byebyecode/releases) 手动下载：


## 使用

### 配置管理

```bash
# 初始化配置文件
byebyecode --init

# 检查配置有效性  
byebyecode --check

# 打印当前配置
byebyecode --print

# 进入 TUI 配置模式
byebyecode --config

# 使用byebyecode 进入Claude
byebyecode --wrap
```

### 主题覆盖

```bash
# 临时使用指定主题（覆盖配置文件设置）
byebyecode --theme cometix
byebyecode --theme minimal
byebyecode --theme gruvbox
byebyecode --theme nord
byebyecode --theme powerline-dark

# 或使用 ~/.claude/byebyecode/themes/ 目录下的自定义主题
byebyecode --theme my-custom-theme
```

### Claude Code 增强

```bash
# 禁用上下文警告并启用详细模式
byebyecode --patch /path/to/claude-code/cli.js

# 常见安装路径示例
byebyecode --patch ~/.local/share/fnm/node-versions/v24.4.1/installation/lib/node_modules/@anthropic-ai/claude-code/cli.js
```

## 默认段落

显示：`目录 | Git 分支状态 | 模型 | 上下文窗口`

### Git 状态指示器

- 带 Nerd Font 图标的分支名
- 状态：`✓` 清洁，`●` 有更改，`⚠` 冲突
- 远程跟踪：`↑n` 领先，`↓n` 落后

### 模型显示

显示简化的 Claude 模型名称：
- `claude-3-5-sonnet` → `Sonnet 3.5`
- `claude-4-sonnet` → `Sonnet 4`

### 上下文窗口显示

基于转录文件分析的令牌使用百分比，包含上下文限制跟踪。

## 配置

byebyecode 支持通过 TOML 文件和交互式 TUI 进行完整配置：

- **配置文件**: `~/.claude/byebyecode/config.toml`
- **交互式 TUI**: `byebyecode --config` 实时编辑配置并预览效果
- **主题文件**: `~/.claude/byebyecode/themes/*.toml` 自定义主题文件
- **自动初始化**: `byebyecode --init` 创建默认配置

### 可用段落

所有段落都支持配置：
- 启用/禁用切换
- 自定义分隔符和图标
- 颜色自定义
- 格式选项

支持的段落：目录、Git、模型、使用量、时间、成本、输出样式


## 系统要求

- **Git**: 版本 1.5+ (推荐 Git 2.22+ 以获得更好的分支检测)
- **终端**: 必须支持 Nerd Font 图标正常显示
  - 安装 [Nerd Font](https://www.nerdfonts.com/) 字体
  - 中文用户推荐: [Maple Font](https://github.com/subframe7536/maple-font) (支持中文的 Nerd Font)
  - 在终端中配置使用该字体
- **Claude Code**: 用于状态栏集成

## 开发

```bash
# 构建开发版本
cargo build

# 运行测试
cargo test

# 构建优化版本
cargo build --release
```

## 路线图

- [x] TOML 配置文件支持
- [x] TUI 配置界面
- [x] 自定义主题
- [x] 交互式主菜单
- [x] Claude Code 增强工具

## 贡献

欢迎贡献！请随时提交 issue 或 pull request。

## 致谢

本项目基于 [@cometix-ai](https://github.com/cometix-ai) 的优秀原作 [CCometixLine](https://github.com/cometix-ai/ccline) 开发。核心架构、状态栏生成和 TUI 界面都基于原项目的基础。我们在此基础上增加了 API 集成、翻译功能和扩展的监控能力等增强特性。

## 相关项目

- [原始 CCometixLine](https://github.com/cometix-ai/ccline) - 本分支基于的原始项目
- [tweakcc](https://github.com/Piebald-AI/tweakcc) - 自定义 Claude Code 主题、思考动词等的命令行工具
- [88Code](https://www.88code.org/) - Claude API 服务提供商
- [ByeByeCode](https://www.byebyecode.org/) - 增强的 Claude Code 使用体验

## 许可证

本项目采用 [MIT 许可证](LICENSE)。

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Haleclipse/CCometixLine&type=Date)](https://star-history.com/#Haleclipse/CCometixLine&Date)