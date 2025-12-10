# ByeByeCode 项目开发指南

> ⚠️ **第一优先级规则**：**未经 JohnYe 本人明确同意，严禁向上游仓库（upstream）提交 PR！** 所有代码变更默认只推送到 fork 仓库（origin），需要提交 PR 时必须先询问并获得许可。

## 项目概述

ByeByeCode 是一个 Rust 编写的 Claude Code 状态栏增强工具，用于显示 88code/Packy 中转站的套餐用量和订阅信息。

- **仓库**: https://github.com/byebye-code/byebyecode
- **语言**: Rust
- **用途**: Claude Code 状态栏插件

## Git 分支说明

### 远程仓库

| 远程名 | 地址 | 说明 |
|--------|------|------|
| `origin` | https://github.com/johnnyee/byebyecode | Fork 仓库（个人开发用） |
| `upstream` | https://github.com/byebye-code/byebyecode | 上游官方仓库 |

### 分支列表

| 分支名 | 用途 | 对应 PR | 状态 |
|--------|------|---------|------|
| `main` | 主分支 | - | ✅ 活跃 |
| `fix/issue-9-subscription-usage-display` | Issue #9 修复 | PR #10 | ✅ 已合并 |
| `feature/progress-bar-usage-display` | 进度条功能 | PR #11 | ✅ 已合并 |
| `fix/skip-free-subscription` | 跳过 FREE 套餐 | PR #12 | ✅ 已合并 |
| `feature/simplify-subscription-display` | 精简订阅显示格式 | PR #15 | 🔄 待审核 |
| `feature/support-new-88code-domains` | 支持新域名 88code.ai | PR #16 | 🔄 待审核 |

### 分支工作流

```bash
# 同步上游代码
git fetch upstream
git checkout main
git merge upstream/main

# 创建功能分支
git checkout -b feature/xxx

# 提交 PR 后合并，删除本地分支
git branch -d feature/xxx
```

## 项目结构

```
byebyecode/
├── src/
│   ├── api/
│   │   ├── mod.rs          # API 数据结构定义
│   │   ├── client.rs       # API 客户端实现
│   │   └── cache.rs        # 缓存管理
│   ├── core/
│   │   └── segments/
│   │       ├── byebyecode_usage.rs        # 用量显示段
│   │       └── byebyecode_subscription.rs # 订阅显示段
│   ├── config/             # 配置管理
│   └── main.rs             # 入口
├── Cargo.toml              # 项目依赖
└── npm/                    # npm 发布相关
```

## 构建命令

### Windows 环境构建

Windows 需要 Visual Studio Build Tools：

```powershell
# 安装 MSVC Build Tools
choco install visualstudio2022buildtools visualstudio2022-workload-vctools -y

# 构建
cargo build --release
```

**注意**: Git 的 `link.exe` 可能与 MSVC 的 `link.exe` 冲突，需要配置 `.cargo/config.toml`：

```toml
[target.x86_64-pc-windows-msvc]
linker = "D:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\link.exe"
```

### Linux/macOS

```bash
cargo build --release
```

## 本地测试

1. 编译项目：`cargo build --release`
2. 修改 `~/.claude/settings.json`：
```json
{
  "statusLine": {
    "command": "D:/Dev/OpenSource/byebyecode/target/release/byebyecode.exe",
    "type": "command"
  }
}
```
3. 重启 Claude Code

## 状态栏段（Segments）配置

byebyecode 支持多个状态栏段，可以根据需要启用或禁用。

### 可用段列表

| 段 ID | 名称 | 说明 |
|-------|------|------|
| `model` | 模型 | 显示当前使用的 AI 模型 |
| `directory` | 目录 | 显示当前工作目录 |
| `git` | Git | 显示 Git 分支和状态 |
| `context_window` | 上下文窗口 | 显示上下文窗口使用情况 |
| `usage` | 用量 | 显示 API 用量（原生） |
| `cost` | 费用 | 显示会话费用 |
| `session` | 会话 | 显示会话信息 |
| `output_style` | 输出样式 | 显示当前输出样式 |
| `update` | 更新 | 显示更新提示 |
| `byebyecode_usage` | 88code 用量 | 显示 88code/Packy 套餐用量（带进度条） |
| `byebyecode_subscription` | 88code 订阅 | 显示所有订阅套餐详情（含重置次数） |
| `byebyecode_status` | 88code 状态 | 显示 88code 服务状态 |

### 88code 专用段详解

#### `byebyecode_usage` - 用量段

显示当前正在扣费的套餐用量，带进度条可视化：

```
88code $34.53/$50 ▓▓▓▓▓▓▓░░░
```

**特性**：
- 自动跳过 FREE 套餐（FREE 不支持 Claude Code）
- 额度用完时显示重置提示
- 支持 88code 和 Packy 两种服务

#### `byebyecode_subscription` - 订阅段

显示所有活跃订阅的详细信息：

```
订阅 PLUS ¥198/月付 (可重置2次, 剩余53天) | PAYGO ¥66/年付 (剩余989天)
```

**特性**：
- 显示所有活跃套餐
- 包含重置次数、剩余天数
- 每个套餐有独特的颜色标识
- 仅支持 88code（Packy 不显示）

### 配置示例

完整配置示例（`~/.claude/settings.json`）：

```json
{
  "statusLine": {
    "command": "byebyecode",
    "type": "command",
    "config": {
      "segments": [
        { "id": "model", "enabled": true },
        { "id": "directory", "enabled": true },
        { "id": "git", "enabled": true },
        { "id": "context_window", "enabled": true },
        { "id": "byebyecode_usage", "enabled": true },
        { "id": "byebyecode_subscription", "enabled": true }
      ]
    }
  }
}
```

### 自定义 API 配置

如果需要自定义 API 地址或密钥，可以在段的 `options` 中配置：

```json
{
  "id": "byebyecode_usage",
  "enabled": true,
  "options": {
    "api_key": "your-api-key",
    "usage_url": "https://www.88code.org/api/usage"
  }
}
```

**说明**：
- 如果不配置 `api_key`，会自动从 `~/.claude/settings.json` 的 `ANTHROPIC_AUTH_TOKEN` 读取
- 如果不配置 `usage_url`，会根据 `ANTHROPIC_BASE_URL` 自动判断使用 88code 或 Packy

## 代码规范

### 格式检查

提交前必须运行 `cargo fmt`，CI 会检查格式：

```bash
cargo fmt           # 自动格式化
cargo fmt -- --check  # 检查格式（CI 使用）
```

### 函数签名格式

多参数函数需要换行：

```rust
// 正确
pub fn get_subscriptions(
    &self,
    model: Option<&str>,
) -> Result<Vec<SubscriptionData>, Box<dyn std::error::Error>> {

// 错误（CI 会失败）
pub fn get_subscriptions(&self, model: Option<&str>) -> Result<Vec<SubscriptionData>, Box<dyn std::error::Error>> {
```

## 88code API 说明

### 套餐扣费逻辑

| 套餐 | 支持 Claude Code | 支持 Codex | 扣费顺序 |
|------|------------------|------------|----------|
| FREE | ❌ 不支持 | ✅ 支持 | 1️⃣ 最先 |
| PLUS/PRO/MAX | ✅ 支持 | ✅ 支持 | 2️⃣ 其次 |
| PAYGO | ✅ 支持 | ✅ 支持 | 3️⃣ 最后 |

### API 接口

- `/api/usage` - 获取用量数据
- `/api/subscription` - 获取订阅信息

**重要**: 需要传入 `model` 参数才能获取正确套餐的用量，否则 API 默认返回 FREE 套餐数据。

### API 返回结构

```json
{
  "creditLimit": 20.0,        // 顶层数据（可能是 FREE）
  "currentCredits": 20.0,
  "subscriptionEntityList": [  // 实际套餐数据在这里
    {
      "subscriptionName": "FREE",
      "creditLimit": 20,
      "currentCredits": 20
    },
    {
      "subscriptionName": "PLUS",
      "creditLimit": 50,
      "currentCredits": 45.47   // 正在使用的套餐
    }
  ]
}
```

## 已完成的功能

### Issue #9 修复 (PR #10, #12)

**问题**: 状态栏始终显示 Free 套餐用量（$0/$20），即使 Plus 套餐正在被扣费。

**解决方案**:
1. 解析 `subscriptionEntityList` 获取真实套餐数据
2. Claude Code 环境下跳过 FREE 套餐
3. 选择第一个有消费的非 FREE 套餐显示

**关键代码** (`src/api/mod.rs`):
```rust
let active_subscription = self
    .subscription_entity_list
    .iter()
    .filter(|s| s.is_active)
    .filter(|s| s.subscription_name.to_uppercase() != "FREE") // 跳过 FREE
    .find(|s| s.current_credits < s.credit_limit);
```

### 进度条功能 (PR #11)

用进度条替代冗余的文字显示：

**改进前**: `$13.86/$50 剩$36.13`
**改进后**: `$13.86/$50 ▓▓▓░░░░░░░`

**关键代码** (`src/core/segments/byebyecode_usage.rs`):
```rust
let bar_length = 10;
let filled = ((percentage / 100.0) * bar_length as f64).round() as usize;
let empty = bar_length - filled;
let progress_bar = format!("{}{}", "▓".repeat(filled), "░".repeat(empty));
```

## PR 提交清单

提交 PR 前确保：

- [ ] `cargo fmt` 格式化代码
- [ ] `cargo build --release` 编译通过
- [ ] 本地测试功能正常
- [ ] 只提交必要的代码文件（不要提交 `.cargo/`、`build.ps1` 等本地配置）
- [ ] commit message 使用中文描述

## 已提交的 PR

| PR | 状态 | 内容 |
|----|------|------|
| #10 | ✅ 已合并 | 修复状态栏错误显示 Free 套餐用量的问题 |
| #11 | ✅ 已合并 | 用进度条可视化用量显示 |
| #12 | ✅ 已合并 | Claude Code 环境下跳过 FREE 套餐 |

## 常见问题

### Windows 编译报错 `linking with link.exe failed`

Git 的 `link.exe` 干扰了 MSVC 的 `link.exe`。解决方案：

1. 创建 `.cargo/config.toml` 指定正确的 linker 路径
2. 或设置 `LIB` 和 `PATH` 环境变量指向 MSVC 工具链

### CI 格式检查失败

运行 `cargo fmt` 后重新提交。

### 状态栏显示 FREE 套餐用量

确保代码包含跳过 FREE 的逻辑（PR #12）。
