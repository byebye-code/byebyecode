# byebyecode 使用教程：让你的 Claude Code 开启超神模式

## 写在前面：我与 byebyecode 的邂逅

作为一个重度 Claude Code 用户，我每天都在与 AI 对话写代码。但说实话，原生的 Claude Code 用起来总有些不够「得劲」：

- **看不见进度**：token 用了多少？还剩多少额度？一问三不知
- **状态栏太素**：就一行文字，啥信息都没有
- **中文输入累**：每次都要绞尽脑汁用英文描述需求
- **警告烦人**：`Context low` 的提示看得我心烦

直到我遇到了 byebyecode，这款基于 Rust 的高性能状态栏增强工具，彻底改变了我的开发体验。现在，我的 Claude Code 状态栏不仅能显示 Git 分支、模型信息、token 使用情况，还能自动监控 88code 套餐状态，甚至支持中英文双向翻译！

这篇教程，我会以第一人称的角度，手把手带你配置 byebyecode，让你的 Claude Code 也能开启「超神模式」。

---

## 第一章：初识 byebyecode

### 1.1 它到底是什么？

byebyecode 是一个基于 Rust 开发的 Claude Code 增强工具，核心特性包括：

**🎨 强大的状态栏系统**
- 实时显示 Git 分支和状态（干净/有修改/有冲突）
- 显示当前使用的 Claude 模型（Sonnet 4, Claude 3.5 等）
- 显示当前工作目录
- 显示上下文窗口使用率（基于 transcript 分析）

**💰 88code API 集成**
- 实时监控 token 用量（已用/总额/剩余）
- 显示套餐类型和价格
- 显示订阅到期时间
- 服务健康状态监控

**🌐 智能翻译功能**
- 使用 GLM-4-Flash 模型
- 中文输入自动翻译成英文发送给 Claude
- Claude 的英文回复自动翻译成中文
- 同时保留原文供参考

**⚡ Claude Code 增强**
- 禁用烦人的 `Context low` 警告
- 启用详细输出模式
- 自动备份原始文件

**🎨 交互式配置**
- 基于 ratatui 的 TUI 界面
- 实时预览配置效果
- 9 种内置主题（Powerline、Gruvbox、Nord 等）
- 完全自定义颜色和图标

### 1.2 系统要求

在开始之前，确保你的系统满足：

- **Git**: 1.5+ (建议 2.22+)
- **终端**: 必须支持 Nerd Font 图标
  - 推荐中文用户使用 [Maple Font](https://github.com/subframe7536/maple-font)
  - 其他可选：[Nerd Fonts](https://www.nerdfonts.com/)
- **Claude Code**: 已安装并可正常使用
- **操作系统**: Linux / macOS / Windows

---

## 第二章：快速上手

### 2.1 下载安装

**方式一：从 Release 下载（推荐）**

前往项目的 [Releases 页面](https://github.com/byebye-code/byebyecode/releases)，根据你的系统下载对应版本：

- **Windows**: `byebyecode-x86_64-pc-windows-msvc.zip`
- **macOS Intel**: `byebyecode-x86_64-apple-darwin.tar.gz`
- **macOS ARM**: `byebyecode-aarch64-apple-darwin.tar.gz`
- **Linux 动态链接**: `byebyecode-x86_64-unknown-linux-gnu.tar.gz`
- **Linux 静态链接**: `byebyecode-x86_64-unknown-linux-musl.tar.gz`

解压后，将 `byebyecode` 可执行文件放到系统 PATH 中，或者记住它的路径。

**验证安装**

```bash
byebyecode --version
```

如果看到版本号输出，说明安装成功！

### 2.2 初始化配置

安装好后，第一步是初始化配置文件：

```bash
byebyecode --init
```

这个命令会：
1. 创建配置目录 `~/.claude/byebyecode/`
2. 生成默认配置文件 `config.toml`
3. 创建主题目录 `themes/`

你会看到类似输出：

```
✓ Created config directory: /Users/你的用户名/.claude/byebyecode
✓ Configuration initialized at: /Users/你的用户名/.claude/byebyecode/config.toml
```

### 2.3 配置 Claude Code 状态栏

要让 byebyecode 接管 Claude Code 的状态栏，需要修改 Claude 的配置文件。

**自动配置（推荐）**

```bash
byebyecode --auto-config
```

这个命令会自动：
- 定位 Claude Code 的配置文件位置
- 将 `statusline` 字段指向 byebyecode
- 备份原始配置

**手动配置**

如果自动配置失败，可以手动编辑 Claude 配置文件：

**macOS/Linux**: `~/.claude/settings.json`
**Windows**: `%USERPROFILE%\.claude\settings.json`

找到或添加 `statusline` 字段：

```json
{
  "statusline": "/path/to/byebyecode"
}
```

Windows 用户注意路径格式：

```json
{
  "statusline": "C:\\Users\\YourName\\.claude\\byebyecode\\byebyecode.exe"
}
```

### 2.4 启动 Claude Code 测试

保存配置后，重启 Claude Code：

```bash
claude
```

现在你应该能看到状态栏显示了丰富的信息！默认显示：

```
目录名 | Git分支 ✓ | Sonnet 4 | 25% · 50k
```

如果状态栏显示正常，恭喜你完成了基础配置！🎉

---

## 第三章：88code API 集成

作为 88code 的用户，实时监控用量和订阅状态是刚需。byebyecode 原生支持 88code API。

### 3.1 获取 API Key

1. 访问 [88code.org](https://www.88code.org/)
2. 登录你的账号
3. 进入「API 设置」或「个人中心」
4. 复制你的 API Key

### 3.2 配置 API Key

**方式一：自动配置**

如果你的 Claude Code 已经配置了 88code，byebyecode 会自动读取 `~/.claude/settings.json` 中的 API Key。

**方式二：手动配置**

编辑 `~/.claude/byebyecode/config.toml`，在对应段落中添加 API Key：

```toml
[[segments]]
id = "ByeByeCodeUsage"
enabled = true

[segments.icon]
plain = "88code"
nerd_font = ""

[segments.options]
api_key = "你的88code_API_Key"

[[segments]]
id = "ByeByeCodeSubscription"
enabled = true

[segments.icon]
plain = "订阅"
nerd_font = ""

[segments.options]
api_key = "你的88code_API_Key"
```

**方式三：使用 --auto-config 自动设置**

```bash
byebyecode --auto-config --88code-key "你的API_Key"
```

### 3.3 功能说明

配置好后，状态栏会显示三个新段落：

**用量监控 (ByeByeCodeUsage)**
- 格式：`$已用/$总额 剩$剩余`
- 示例：`$12.50/$50.00 剩$37.50`
- 自动刷新，无需重启

**订阅信息 (ByeByeCodeSubscription)**
- 格式：`套餐名 价格 (剩余:N天)`
- 示例：`专业版 $49.99/月 (剩余:15天)`
- 显示套餐类型和到期时间

**服务状态 (ByeByeCodeStatus)**
- 正常：`88code正持续为您服务`（彩色显示）
- 异常：`88code服务断开`（红色警告）
- 30 秒缓存，避免频繁请求

### 3.4 我的使用体验

配置好 88code 集成后，我的开发体验有了质的提升：

1. **不再担心额度**：状态栏实时显示剩余额度，用到 20% 我就知道该充值了
2. **订阅不过期**：剩余天数一目了然，不会忘记续费
3. **服务稳定性**：万一 88code 服务波动，状态栏会立即提醒我

---

## 第四章：翻译功能 - 用中文写代码

### 4.1 为什么需要翻译？

作为中文母语者，我经常遇到这样的场景：

- 脑子里是中文思路，但要用英文描述给 Claude
- 翻译过程中可能丢失细节或表达不准确
- Claude 的回复太长，读英文累

byebyecode 的翻译功能完美解决了这个问题：

**你只需用中文输入**，byebyecode 自动翻译成英文发给 Claude
**Claude 的英文回复**，自动翻译成中文，同时保留原文

### 4.2 配置翻译功能

翻译功能使用智谱 AI 的 GLM-4-Flash 模型，需要：

**步骤 1：获取 GLM API Key**

1. 访问 [智谱 AI 开放平台](https://open.bigmodel.cn/)
2. 注册并登录
3. 创建 API Key（GLM-4-Flash 模型价格非常便宜）

**步骤 2：启动包装模式**

byebyecode 的翻译功能需要以「包装模式」启动，即通过 byebyecode 启动 Claude Code：

```bash
byebyecode --wrap --glm-key "你的GLM_API_Key"
```

**参数说明：**

- `--wrap`: 启动包装模式
- `--glm-key`: 指定 GLM API Key
- `--translation true`: 启用翻译（默认启用）
- `--translation false`: 禁用翻译

### 4.3 实际使用

**场景 1：中文提问**

我输入中文：

```
帮我写一个 Python 函数，计算斐波那契数列的第 n 项，要求使用动态规划，时间复杂度 O(n)
```

byebyecode 自动翻译成英文发送给 Claude：

```
Help me write a Python function to calculate the nth term of the Fibonacci sequence, using dynamic programming with O(n) time complexity.
```

**场景 2：中文回复**

Claude 返回英文代码和解释，byebyecode 自动翻译成中文：

```python
# 中文翻译
以下是使用动态规划实现的斐波那契数列函数：

def fibonacci(n):
    if n <= 1:
        return n
    dp = [0] * (n + 1)
    dp[1] = 1
    for i in range(2, n + 1):
        dp[i] = dp[i-1] + dp[i-2]
    return dp[n]

这个函数的时间复杂度是 O(n)，空间复杂度也是 O(n)。

[原文] Here's a Fibonacci function implemented using dynamic programming:

def fibonacci(n):
    if n <= 1:
        return n
    dp = [0] * (n + 1)
    dp[1] = 1
    for i in range(2, n + 1):
        dp[i] = dp[i-1] + dp[i-2]
    return dp[n]

The time complexity of this function is O(n), and the space complexity is also O(n).
```

**注意**：原文会保留在下方，方便对照。

### 4.4 翻译质量

GLM-4-Flash 的翻译质量非常高：

- **信达雅**：中译英准确传达意图
- **流畅自然**：英译中符合中文表达习惯
- **上下文理解**：能识别技术术语，不会乱翻译

我使用了一个月，基本没遇到翻译错误的情况。

### 4.5 性能影响

你可能担心翻译会拖慢响应速度。实际测试：

- **中译英延迟**：~200-500ms（GLM-4-Flash 很快）
- **英译中延迟**：~300-800ms（根据回复长度）
- **总体影响**：几乎无感知，因为 Claude 思考时间远大于翻译时间

---

## 第五章：TUI 配置界面

byebyecode 最让我惊艳的是它的交互式配置界面，基于 ratatui 构建，使用体验堪比桌面应用。

### 5.1 启动配置界面

```bash
byebyecode --config
```

或者直接运行（无 stdin 输入时自动显示菜单）：

```bash
byebyecode
```

选择「配置器」进入 TUI 界面。

### 5.2 界面布局

配置界面分为三个区域：

```
┌─────────────────────────────────────────────────────────┐
│  实时预览区                                             │
│  目录名 | Git分支 ✓ | Sonnet 4 | 25% · 50k            │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│  段落列表                                               │
│  [✓] 目录                                               │
│  [✓] Git 状态                                           │
│  [✓] 模型                                               │
│  [✓] 上下文窗口                                         │
│  [✓] 88code 用量                                        │
│  [✓] 88code 订阅                                        │
│  [ ] 88code 状态                                        │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│  详细设置区                                             │
│  段落 ID: Directory                                     │
│  图标: 📁                                               │
│  前景色: RGB(255, 255, 255)                            │
│  背景色: RGB(38, 38, 38)                               │
└─────────────────────────────────────────────────────────┘
```

### 5.3 操作指南

**基本操作**

- `Tab` / `Shift+Tab`: 在区域间切换
- `↑` / `↓`: 在列表中移动
- `Enter`: 进入编辑模式 / 确认
- `Space`: 切换段落启用状态
- `Esc`: 退出编辑 / 返回上级
- `s`: 保存配置
- `q`: 退出配置器

**编辑段落**

1. 在段落列表中选中要编辑的段落
2. 按 `Enter` 进入详细设置
3. 修改图标、颜色、样式等
4. 按 `s` 保存

**切换主题**

1. 按 `t` 打开主题选择器
2. 用 `↑` / `↓` 浏览主题
3. 实时预览效果
4. 按 `Enter` 应用主题

**内置主题**

- **default**: 经典默认主题
- **cometix**: Cometix 风格
- **minimal**: 极简风格
- **gruvbox**: Gruvbox 配色
- **nord**: Nord 配色
- **powerline-dark**: Powerline 暗色
- **powerline-light**: Powerline 亮色
- **powerline-rose-pine**: Rose Pine 配色
- **powerline-tokyo-night**: Tokyo Night 配色

### 5.4 自定义段落

你可以完全自定义每个段落的外观：

**修改图标**

```toml
[segments.icon]
plain = "文件夹"
nerd_font = ""  # Nerd Font 图标
```

**修改颜色**

```toml
[segments.colors]
icon = { c256 = 214 }          # 256色模式
text = { c256 = 255 }          # 白色文字
background = { r = 38, g = 38, b = 38 }  # RGB模式
```

**修改样式**

```toml
[segments.styles]
text_bold = true  # 加粗文字
```

### 5.5 我的配置分享

经过一番折腾，我最终选择了 **powerline-tokyo-night** 主题，并做了微调：

- 启用了 `88code用量`、`88code订阅`、`88code状态` 三个段落
- 禁用了 `时间` 和 `成本` 段落（我不需要）
- 将 Git 段落的图标改成了 ``
- 调整了背景色，与我的终端主题一致

最终效果：

```
📁 CCometixLine | main ✓ ↑2 | Sonnet 4 | 45% · 90k | $12.50/$50 剩$37.50 | 专业版 $49.99 (剩15天)
```

---

## 第六章：Claude Code 增强功能

除了状态栏，byebyecode 还能增强 Claude Code 本身的行为。

### 6.1 禁用上下文警告

Claude Code 在上下文使用超过一定比例时，会频繁弹出 `Context low` 警告，非常烦人。

byebyecode 提供了补丁工具：

```bash
byebyecode --patch /path/to/claude-code/cli.js
```

**常见路径**

**Linux (fnm)**:
```bash
byebyecode --patch ~/.local/share/fnm/node-versions/v24.4.1/installation/lib/node_modules/@anthropic-ai/claude-code/cli.js
```

**macOS (Homebrew)**:
```bash
byebyecode --patch /opt/homebrew/lib/node_modules/@anthropic-ai/claude-code/cli.js
```

**Windows**:
```bash
byebyecode --patch "C:\Users\你的用户名\AppData\Roaming\npm\node_modules\@anthropic-ai\claude-code\cli.js"
```

### 6.2 补丁效果

补丁会：

1. **禁用上下文警告**：移除 `Context low` 提示
2. **启用详细模式**：增强输出详细信息
3. **自动备份**：创建 `.backup` 文件，支持恢复

补丁使用正则表达式替换，可以适应大部分 Claude Code 版本更新。

### 6.3 恢复原始版本

如果需要恢复：

```bash
# 找到备份文件
ls /path/to/claude-code/cli.js.backup

# 恢复
mv /path/to/claude-code/cli.js.backup /path/to/claude-code/cli.js
```

---

## 第七章：高级用法

### 7.1 命令行参数速查

```bash
# 查看帮助
byebyecode --help

# 查看版本
byebyecode --version

# 初始化配置
byebyecode --init

# 检查配置有效性
byebyecode --check

# 打印当前配置
byebyecode --print

# 进入 TUI 配置模式
byebyecode --config
byebyecode -c

# 使用指定主题（临时覆盖）
byebyecode --theme cometix
byebyecode --theme gruvbox

# 包装模式（翻译功能）
byebyecode --wrap
byebyecode --wrap --glm-key "KEY"
byebyecode --wrap --translation false  # 禁用翻译

# 自动配置
byebyecode --auto-config
byebyecode --auto-config --88code-key "KEY" --glm-key "KEY"

# 补丁 Claude Code
byebyecode --patch /path/to/cli.js
```

### 7.2 环境变量

byebyecode 支持通过环境变量配置：

```bash
# 设置代理
export HTTP_PROXY=http://127.0.0.1:7890
export HTTPS_PROXY=http://127.0.0.1:7890

# GLM API Key
export GLM_API_KEY="你的Key"

# 88code API Key
export BYEBYECODE_API_KEY="你的Key"
```

### 7.3 代理配置

如果你在国内使用 Claude Code，可能需要配置代理：

**方法 1：环境变量**

```bash
export HTTP_PROXY=http://127.0.0.1:7890
export HTTPS_PROXY=http://127.0.0.1:7890
byebyecode --wrap
```

**方法 2：配置文件**

编辑 `~/.claude/byebyecode/config.toml`：

```toml
[proxy]
http = "http://127.0.0.1:7890"
https = "http://127.0.0.1:7890"
```

### 7.4 自定义主题

除了内置主题，你还可以创建自己的主题：

1. 创建主题文件 `~/.claude/byebyecode/themes/my-theme.toml`
2. 参考内置主题的结构编写
3. 使用 `byebyecode --theme my-theme` 加载

**示例主题**

```toml
[global]
separator = ""
use_nerd_fonts = true

[[segments]]
id = "Directory"
enabled = true

[segments.icon]
plain = "目录"
nerd_font = ""

[segments.colors]
icon = { c256 = 39 }
text = { c256 = 255 }
background = { r = 30, g = 30, b = 30 }

[segments.styles]
text_bold = false

# ... 其他段落配置
```

### 7.5 性能优化

byebyecode 已经非常快（启动 < 50ms，内存 < 10MB），但如果你想进一步优化：

**减少段落数量**

只启用你需要的段落，禁用其他：

```bash
byebyecode --config
# 在 TUI 中用 Space 切换段落状态
```

**调整缓存**

某些段落（如 88code 状态）使用缓存减少 API 调用：

- 健康状态缓存：30 秒
- 用量/订阅数据：无缓存（实时查询）

---

## 第八章：常见问题

### Q1: 状态栏显示乱码

**原因**：终端未使用 Nerd Font 字体

**解决**：
1. 安装 Nerd Font（推荐 Maple Font）
2. 在终端设置中切换字体
3. 或者禁用 Nerd Font 图标：

```toml
[global]
use_nerd_fonts = false
```

### Q2: 88code API 调用失败

**可能原因**：
- API Key 错误或过期
- 网络连接问题
- 88code 服务暂时不可用

**解决**：
1. 检查 API Key 是否正确
2. 测试网络连接：`curl https://www.88code.org`
3. 查看 `88code状态` 段落的服务状态

### Q3: 翻译功能不工作

**检查清单**：
- [ ] 使用 `--wrap` 启动了包装模式
- [ ] 提供了正确的 GLM API Key
- [ ] GLM API 额度充足
- [ ] 网络可以访问智谱 AI API

**测试方法**：

```bash
byebyecode --wrap --glm-key "你的Key"
# 输入中文测试
```

### Q4: Git 段落不显示

**原因**：当前目录不是 Git 仓库

**解决**：进入 Git 仓库目录，或者禁用 Git 段落

### Q5: 补丁失败

**可能原因**：
- Claude Code 版本过新，正则匹配失败
- cli.js 路径错误

**解决**：
1. 确认 cli.js 路径正确
2. 查看 `.backup` 文件是否生成
3. 提交 issue 报告你的 Claude Code 版本

### Q6: Windows 路径问题

Windows 用户注意：
- 使用双反斜杠 `\\` 或正斜杠 `/`
- 路径中有空格要加引号

**正确示例**：

```bash
byebyecode --patch "C:/Program Files/nodejs/node_modules/@anthropic-ai/claude-code/cli.js"
```

---

## 第九章：最佳实践

### 9.1 我的日常工作流

1. **早上启动**

```bash
# 启动包装模式（翻译 + 状态栏）
byebyecode --wrap --glm-key $GLM_KEY
```

2. **开始工作**

用中文描述需求，byebyecode 自动翻译，Claude 回复也自动翻译成中文。

3. **监控用量**

状态栏实时显示 88code 用量，快用完时及时充值。

4. **主题切换**

白天用 `powerline-light`，晚上用 `powerline-tokyo-night`：

```bash
# 不需要重启 Claude，只需重新加载状态栏
# 方法：切换到其他目录再切回来，或者运行 git 命令
```

### 9.2 推荐配置

**极简主义者**

```toml
# 只启用 3 个段落
[[segments]]
id = "Directory"
enabled = true

[[segments]]
id = "Git"
enabled = true

[[segments]]
id = "Model"
enabled = true
```

**信息狂魔**

```toml
# 启用所有段落
[[segments]]
id = "Directory"
enabled = true

[[segments]]
id = "Git"
enabled = true

[[segments]]
id = "Model"
enabled = true

[[segments]]
id = "Usage"
enabled = true

[[segments]]
id = "ByeByeCodeUsage"
enabled = true

[[segments]]
id = "ByeByeCodeSubscription"
enabled = true

[[segments]]
id = "ByeByeCodeStatus"
enabled = true

[[segments]]
id = "Time"
enabled = true

[[segments]]
id = "Cost"
enabled = true
```

### 9.3 性能建议

- 使用静态 Linux 二进制（musl 版本）可以在任何 Linux 发行版上运行
- 禁用不需要的段落减少状态栏生成时间
- 88code 状态段落使用 30 秒缓存，避免频繁 API 调用
- 翻译功能按需启用，不需要时可以不加 `--glm-key` 参数

---

## 第十章：总结与展望

### 10.1 我的使用感受

使用 byebyecode 三个月后，我的开发效率提升明显：

**效率提升**
- 用中文描述需求，省去了翻译的心智负担
- 实时监控用量，再也不会突然没额度
- 状态栏信息丰富，一眼掌握项目状态

**体验改善**
- 没有了烦人的 `Context low` 警告
- TUI 配置界面非常直观，调整配置很方便
- 多种主题让状态栏既实用又好看

**性能优秀**
- Rust 编写，启动速度 < 50ms
- 内存占用 < 10MB
- 翻译延迟几乎无感知

### 10.2 未来期待

作为用户，我希望 byebyecode 未来能加入：

- 更多翻译模型支持（如 Claude 自己做翻译）
- 插件系统，允许自定义段落
- 更多 API 集成（OpenRouter、Azure OpenAI 等）
- 状态栏点击交互（切换主题、查看详细信息）
- 移动端支持（Termux）

### 10.3 致谢

感谢 byebyecode 的开发者，也感谢原始 CCometixLine 项目提供的优秀基础。这个工具真正做到了「以用户体验为中心」，每个功能都很实用。

---

## 附录：快速参考

### 常用命令

```bash
# 初始化
byebyecode --init

# 启动翻译模式
byebyecode --wrap --glm-key "KEY"

# 配置界面
byebyecode --config

# 切换主题
byebyecode --theme tokyo-night

# 补丁 Claude
byebyecode --patch /path/to/cli.js
```

### 配置文件位置

- 主配置：`~/.claude/byebyecode/config.toml`
- API Keys：`~/.claude/byebyecode/api_keys.toml`
- 自定义主题：`~/.claude/byebyecode/themes/*.toml`
- 缓存：`~/.claude/88code/.cache/`

### 状态栏段落

| 段落 ID | 说明 | 默认启用 |
|---------|------|----------|
| Directory | 当前目录 | ✓ |
| Git | Git 分支和状态 | ✓ |
| Model | Claude 模型 | ✓ |
| Usage | 上下文窗口使用率 | ✓ |
| ByeByeCodeUsage | 88code 用量 | ✗ |
| ByeByeCodeSubscription | 88code 订阅 | ✗ |
| ByeByeCodeStatus | 88code 服务状态 | ✗ |
| Time | 当前时间 | ✗ |
| Cost | 估算成本 | ✗ |
| OutputStyle | 输出风格 | ✗ |

### 主题列表

- `default` - 默认主题
- `cometix` - Cometix 风格
- `minimal` - 极简风格
- `gruvbox` - Gruvbox 配色
- `nord` - Nord 配色
- `powerline-dark` - Powerline 暗色
- `powerline-light` - Powerline 亮色
- `powerline-rose-pine` - Rose Pine 配色
- `powerline-tokyo-night` - Tokyo Night 配色

---

## 结语

写这篇教程的初衷，是因为我真的很喜欢 byebyecode 这个工具。它不是那种「看起来很炫但不实用」的玩具，而是真正能提升日常开发效率的利器。

如果你也是 Claude Code 的重度用户，如果你也为中文输入和用量监控而烦恼，不妨试试 byebyecode。我相信它不会让你失望。

最后，如果这篇教程对你有帮助，欢迎给项目点个 Star！也欢迎在 issue 中分享你的使用体验和建议。

Happy Coding! 🚀

---

**项目地址**: [https://github.com/byebye-code/byebyecode](https://github.com/byebye-code/byebyecode)

**原始项目**: [https://github.com/cometix-ai/ccline](https://github.com/cometix-ai/ccline)

**相关链接**:
- [88code](https://www.88code.org/)
- [智谱 AI](https://open.bigmodel.cn/)
- [Maple Font](https://github.com/subframe7536/maple-font)

---

*教程版本: v1.0 | 最后更新: 2025年1月*
