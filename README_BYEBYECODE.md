# byebyecode

> 基于 CCometixLine 的 Claude Code 增强包装器 - 支持中英文双向翻译、用量监控和增强功能

## 功能特性

### 🌍 双向翻译
- **中文 → 英文**: 自动将中文输入翻译成信达雅的英文发送给 Claude Code
- **英文 → 中文**: Claude Code 的回复翻译成中文显示,同时保留英文原文供对照
- 使用 GLM-4.5-Flash 免费翻译 API

### 📊 实时用量监控
- **StatusLine 集成**: 在 Claude Code 状态栏实时显示 byebyecode 套餐使用情况
- **用量追踪**: 显示已用Token百分比和剩余容量
- **订阅状态**: 显示当前套餐类型和状态

### 🎨 增强用户体验
- **启动广告**: 启动时展示 byebyecode 套餐信息和价格
- **动画消息**: "byebyecode 正持续为您服务" 水波纹流动变色效果
- **Token 耗尽提醒**: 自动检测套餐用尽,展示付款二维码

### ⚙️ 自动化配置
- 自动检测 Claude Code 路径(从 PATH 环境变量)
- 一键安装和配置
- 跨平台支持(Windows/Linux/macOS)

## 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/byebyecode/byebyecode.git
cd byebyecode

# 构建 release 版本
cargo build --release

# 二进制文件位于 target/release/byebyecode 或 byebyecode.exe (Windows)
```

## 配置

### 1. 设置 API Keys

```bash
# 自动配置(推荐)
byebyecode --glm-key YOUR_GLM_API_KEY --byebyecode-key YOUR_BYEBYECODE_API_KEY

# 这将:
# - 创建配置文件 ~/.claude/ccline/config.toml
# - 保存 API keys 到 ~/.claude/ccline/api_keys.toml
# - 安装二进制文件到 ~/.claude/ccline/byebyecode
```

### 2. 获取 API Keys

**GLM API Key (翻译功能)**
- 访问: https://docs.bigmodel.cn/
- 注册并获取免费的 GLM-4.5-Flash API Key

**byebyecode API Key (用量监控)**
- 访问: https://www.byebyecode.org/
- 注册并获取 API Key
- 查看套餐: https://www.byebyecode.org/#pricing

## 使用方法

### 基本使用

```bash
# 启动 byebyecode 包装模式
byebyecode --wrap

# 启动时显示广告
byebyecode --show-ad

# 启用/禁用翻译功能
byebyecode --wrap --translation true
byebyecode --wrap --translation false
```

### 配置 Claude Code StatusLine

在 Claude Code 中配置 byebyecode 作为 statusline 工具:

```bash
# 使用配置器 TUI
byebyecode --config

# 或手动编辑 ~/.claude/ccline/config.toml
```

配置示例:
```toml
[style]
mode = "powerline"
separator = ""

[[segments]]
id = "bye_bye_code_usage"
enabled = true

[segments.icon]
plain = "BYE"
nerd_font = ""

[segments.colors]
icon = { c256 = 214 }
text = { c256 = 255 }
background = { c256 = 236 }

[segments.options]
api_key = "YOUR_BYEBYECODE_API_KEY"

[[segments]]
id = "bye_bye_code_subscription"
enabled = true

[segments.icon]
plain = "SUB"
nerd_font = ""

[segments.colors]
icon = { c256 = 39 }
text = { c256 = 255 }
background = { c256 = 236 }

[segments.options]
api_key = "YOUR_BYEBYECODE_API_KEY"
```

## 命令行选项

```
byebyecode [OPTIONS]

OPTIONS:
  -c, --config              进入 TUI 配置模式
  -t, --theme <THEME>       设置主题
      --print               打印当前配置
      --init                初始化配置文件
      --check               检查配置有效性
  -u, --update              检查更新
      --patch <PATH>        修补 Claude Code cli.js
      --wrap                启动包装模式(注入 Claude Code)
      --translation <BOOL>  启用/禁用翻译功能
      --show-ad             显示启动广告
      --glm-key <KEY>       设置 GLM API Key
      --byebyecode-key <KEY>  设置 byebyecode API Key
  -h, --help                显示帮助信息
  -V, --version             显示版本信息
```

## byebyecode 套餐

访问 https://www.byebyecode.org/#pricing 查看最新套餐:

- **Free**: ¥0/月 - 10M tokens/月
- **Pro**: ¥99/月 - 500M tokens/月 + Claude Sonnet 4.5
- **Enterprise**: ¥299/月 - 无限 tokens + 所有模型 + API 访问

## API 端点

byebyecode 使用以下 API:

- **用量查询**: `GET https://www.byebyecode.org/api/usage`
- **订阅查询**: `GET https://www.byebyecode.org/api/subscription`
- **认证方式**: `Authorization: Bearer YOUR_API_KEY`

返回格式:

```json
// Usage API
{
  "total_tokens": 1000000000,
  "used_tokens": 625000000,
  "remaining_tokens": 375000000,
  "percentage_used": 62.5
}

// Subscription API
{
  "plan_name": "Pro",
  "plan_price": "¥99/月",
  "expires_at": "2025-12-31T23:59:59Z",
  "is_active": true
}
```

## 项目架构

```
byebyecode/
├── src/
│   ├── main.rs                   # 主入口
│   ├── cli.rs                    # CLI 参数解析
│   ├── translation/              # 翻译模块
│   │   ├── mod.rs
│   │   └── glm.rs               # GLM-4.5-Flash 实现
│   ├── api/                     # byebyecode API 客户端
│   │   ├── mod.rs
│   │   └── client.rs
│   ├── wrapper/                 # Claude Code 包装器
│   │   ├── mod.rs
│   │   ├── injector.rs         # 进程注入和拦截
│   │   └── io_interceptor.rs
│   ├── ui/                      # UI 组件
│   │   ├── banner.rs           # 广告横幅
│   │   ├── animated_message.rs # 动画消息
│   │   └── payment_qr.rs       # 支付二维码
│   ├── auto_config/            # 自动配置
│   │   └── mod.rs
│   └── core/segments/          # StatusLine segments
│       ├── byebyecode_usage.rs
│       └── byebyecode_subscription.rs
└── Cargo.toml
```

## 技术栈

- **语言**: Rust 2021 Edition
- **UI**: ratatui + crossterm (TUI)
- **HTTP**: reqwest (blocking)
- **翻译**: GLM-4.5-Flash API
- **二维码**: qrcode (可选)

## 开发

```bash
# 运行开发版本
cargo run -- --help

# 运行测试
cargo test

# 代码检查
cargo check
cargo clippy -- -D warnings

# 格式化
cargo fmt
```

## 许可证

MIT License

## 相关链接

- byebyecode 官网: https://www.byebyecode.org/
- GLM API 文档: https://docs.bigmodel.cn/
- CCometixLine 原项目: https://github.com/username/CCometixLine

## 常见问题

### Q: 如何从 PATH 找到 Claude Code?

byebyecode 会自动搜索:
1. `which claude` (所有平台)
2. Windows: `%APPDATA%\npm\claude.cmd`
3. macOS: `/usr/local/bin/claude`, `/opt/homebrew/bin/claude`
4. Linux: `/usr/local/bin/claude`, `/usr/bin/claude`

### Q: 翻译功能失败怎么办?

检查:
1. GLM API Key 是否正确
2. 网络连接是否正常
3. GLM API 配额是否用尽

### Q: StatusLine 不显示 byebyecode 信息?

确保:
1. byebyecode API Key 已配置
2. Segments 已在配置中启用
3. Claude Code 配置了正确的 statusline 命令

## 贡献

欢迎提交 Issue 和 Pull Request!
