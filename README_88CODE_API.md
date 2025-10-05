# 88code API 集成说明

## 概述

byebyecode 自动从 `~/.claude/settings.json` 读取 API key,并通过缓存机制实现高效的用量和订阅信息展示。

## API Key 自动读取

### 工作原理

1. **自动检测**: 程序启动时自动读取 `~/.claude/settings.json`
2. **条件验证**: 仅当 `ANTHROPIC_BASE_URL` 包含 `88code.org` 时才使用 API key
3. **跨平台支持**:
   - Windows: `C:\Users\<用户>\.claude\settings.json`
   - Linux/macOS: `~/.claude/settings.json`

### settings.json 格式

```json
{
  "env": {
    "ANTHROPIC_AUTH_TOKEN": "88_ea77c08c3b175dfc23731619c8a7f266b27558e83bae73e2819f82353dd891eb",
    "ANTHROPIC_BASE_URL": "https://www.88code.org/api"
  }
}
```

## 缓存机制

### 缓存策略

- **缓存有效期**: 10秒
- **缓存位置**: `~/.claude/ccline/.cache/`
  - `usage.json` - 用量数据缓存
  - `subscription.json` - 订阅数据缓存

### 工作流程

```
请求状态栏
    ↓
检查缓存是否有效(10秒内)?
    ↓ 是
    返回缓存数据 (快速)
    ↓ 否
调用 88code API
    ↓ 成功
    保存到缓存 → 返回最新数据
    ↓ 失败
    返回最后成功的数据 (标记"缓存")
    ↓ 完全无数据
    显示"等待API"
```

### 显示状态

| 情况 | 显示效果 | 说明 |
|------|---------|------|
| 缓存有效 | `已用625M/1000M 剩余375M` | 10秒内的数据 |
| API成功 | `已用625M/1000M 剩余375M` | 最新数据并刷新缓存 |
| API失败(有历史) | `已用625M/1000M 剩余375M (缓存)` | 使用最后成功的数据 |
| 完全无数据 | `等待API` | 从未成功获取过数据 |

## API 端点规范

### 1. 用量查询 API

**请求**:
```http
POST https://www.88code.org/api/usage
Authorization: Bearer 88_ea77c08c3b175dfc23731619c8a7f266b27558e83bae73e2819f82353dd891eb
Content-Type: application/json
```

**响应**:
```json
{
  "total_tokens": 1000000000,
  "used_tokens": 625000000,
  "remaining_tokens": 375000000,
  "percentage_used": 62.5
}
```

**字段说明**:
- `total_tokens`: 总 token 数
- `used_tokens`: 已使用 token 数
- `remaining_tokens`: 剩余 token 数
- `percentage_used`: 使用百分比

**显示效果**:
```
88 已用625M/1000M 剩余375M
```

### 2. 订阅查询 API

**请求**:
```http
POST https://www.88code.org/api/subscription
Authorization: Bearer 88_ea77c08c3b175dfc23731619c8a7f266b27558e83bae73e2819f82353dd891eb
Content-Type: application/json
```

**响应**:
```json
{
  "plan_name": "Pro",
  "plan_price": "¥99/月",
  "is_active": true,
  "expires_at": "2025-11-05"
}
```

**字段说明**:
- `plan_name`: 套餐名称 (Free/Pro/Enterprise)
- `plan_price`: 价格显示 (如 "¥99/月", "¥0/月")
- `is_active`: 是否有效 (true/false)
- `expires_at`: 过期时间 (可选,格式: YYYY-MM-DD)

**显示效果**:
```
SUB Pro ¥99/月 (有效)
SUB Free ¥0/月 (已过期)
```

## 性能优化

### 为什么需要缓存?

1. **减少API调用**: Claude Code 每次更新状态栏都会调用 byebyecode
2. **提高响应速度**: 从缓存读取比 HTTP 请求快100倍以上
3. **降低服务器压力**: 10秒轮询间隔,而不是每秒多次请求
4. **离线容错**: API 临时故障时仍可显示历史数据

### 缓存更新时机

- **定时更新**: 每10秒过期后下次请求时自动刷新
- **失败容错**: API失败时继续使用旧数据,不中断显示
- **启动即用**: 启动后立即读取缓存,无需等待API

## 测试 API 集成

### 创建测试缓存

```bash
# 创建缓存目录
mkdir -p ~/.claude/ccline/.cache

# 创建用量缓存
cat > ~/.claude/ccline/.cache/usage.json << EOF
{
  "data": {
    "total_tokens": 1000000000,
    "used_tokens": 625000000,
    "remaining_tokens": 375000000,
    "percentage_used": 62.5
  },
  "timestamp": $(date +%s)
}
EOF

# 创建订阅缓存
cat > ~/.claude/ccline/.cache/subscription.json << EOF
{
  "data": {
    "plan_name": "Pro",
    "plan_price": "¥99/月",
    "is_active": true,
    "expires_at": "2025-11-05"
  },
  "timestamp": $(date +%s)
}
EOF
```

### 验证显示效果

```bash
echo '{"model":{"id":"test","display_name":"Test"},"workspace":{"current_dir":"."},"transcript_path":"test.jsonl"}' | byebyecode
```

预期输出:
```
🤖 Test | 📁 . | 🌿 master ● | ⚡️ - · - tokens | 88 已用625M/1000M 剩余375M | SUB Pro ¥99/月 (有效)
```

## 故障排查

### 问题1: 一直显示"等待API"

**原因**:
- API 端点未实现
- settings.json 中没有正确的 API key
- ANTHROPIC_BASE_URL 不包含 `88code.org`

**解决**:
1. 检查 `~/.claude/settings.json` 格式
2. 手动创建测试缓存(见上方)
3. 实现服务端 API 端点

### 问题2: 显示"(缓存)"标记

**原因**: API 调用失败,使用历史数据

**解决**:
- 检查 88code.org API 是否正常运行
- 检查网络连接
- 查看 API 返回的错误信息

### 问题3: 数据不更新

**原因**: 缓存仍在10秒有效期内

**解决**:
- 等待10秒后自动更新
- 或删除缓存文件: `rm ~/.claude/ccline/.cache/*.json`

## 代码位置

- **API key 读取**: `src/api/mod.rs` - `get_api_key_from_claude_settings()`
- **缓存管理**: `src/api/cache.rs`
- **用量段**: `src/core/segments/byebyecode_usage.rs`
- **订阅段**: `src/core/segments/byebyecode_subscription.rs`
- **API 客户端**: `src/api/client.rs`

## 下一步

1. **实现服务端 API**: 在 88code.org 上实现 `/api/usage` 和 `/api/subscription` 端点
2. **测试集成**: 使用真实 API key 测试完整流程
3. **监控性能**: 观察缓存命中率和 API 响应时间
4. **用户反馈**: 收集用户对显示效果的反馈
