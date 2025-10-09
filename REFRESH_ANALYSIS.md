# Statusline 自动刷新方案可行性分析

## 问题核心

当前注入的代码假设存在 `statusLineComponent.update()`，但这可能不正确。

## Claude Code Statusline 真实机制

从搜索结果看到：
- `statusLineText` 变量存储状态栏文本
- 通过执行用户配置的 command 获取输出
- 需要找到**触发命令执行**的函数

## 三种可能的实现方案

### 方案 A：找到现有刷新函数（最理想）
如果 Claude Code 有定期刷新机制，直接调用：
```javascript
setInterval(() => {
  refreshStatusLine(); // Claude Code 内部函数
}, 60000);
```

**需要验证**：
- [ ] 是否存在 `refreshStatusLine` 或类似函数
- [ ] 该函数是否可以被 setInterval 调用

### 方案 B：模拟用户输入触发刷新（曲线救国）
```javascript
setInterval(() => {
  // 发送空字符或特殊键触发刷新
  process.stdin.write('\x00');
}, 60000);
```

**风险**：可能干扰用户输入

### 方案 C：直接重新执行 statusLine.command（最可靠）
```javascript
setInterval(() => {
  const { execSync } = require('child_process');
  const settings = /* 读取配置 */;
  if (settings.statusLine?.command) {
    try {
      const output = execSync(settings.statusLine.command);
      // 更新显示
    } catch(e) {}
  }
}, 60000);
```

**优点**：最接近真实刷新逻辑
**缺点**：需要找到更新显示的代码位置

## 实际验证步骤

1. **打补丁测试**：
   ```bash
   byebyecode --patch "真实cli.js路径"
   ```

2. **启动 Claude Code，观察**：
   - 是否有控制台错误？
   - 60 秒后状态栏是否更新？

3. **如果不工作**：
   - 需要深入分析 cli.js 的混淆代码
   - 找到真实的刷新触发点
   - 调整注入策略

## 结论

**当前方案是理论性的**，需要实际测试验证。如果不工作，需要：
1. 反混淆 cli.js 部分代码
2. 找到真实的状态栏更新逻辑
3. 重新设计注入代码
