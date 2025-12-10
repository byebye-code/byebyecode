# ByeByeCode 项目开发指南

## 项目概述

ByeByeCode 是一个 Rust 编写的 Claude Code 状态栏增强工具，用于显示 88code/Packy 中转站的套餐用量和订阅信息。

- **仓库**: https://github.com/byebye-code/byebyecode
- **语言**: Rust
- **用途**: Claude Code 状态栏插件

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
