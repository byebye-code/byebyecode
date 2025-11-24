#!/usr/bin/env node
const { spawnSync, execSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

// ==================== 辅助函数 ====================

/**
 * 检查指定 PID 的进程是否仍在运行
 */
function isProcessRunning(pid) {
  try {
    const platform = process.platform;

    if (platform === 'win32') {
      const output = execSync(`tasklist /FI "PID eq ${pid}"`, {
        encoding: 'utf8',
        stdio: ['ignore', 'pipe', 'ignore']
      });
      return output.includes(pid.toString());
    } else {
      // Unix-like systems
      const output = execSync(`ps -p ${pid}`, {
        encoding: 'utf8',
        stdio: ['ignore', 'pipe', 'ignore']
      });
      return output.includes(pid.toString());
    }
  } catch (e) {
    return false; // 进程不存在
  }
}

/**
 * 清理过期或无效的锁文件
 */
function cleanupStaleLock(lockFile) {
  if (!fs.existsSync(lockFile)) return;

  try {
    const lockData = JSON.parse(fs.readFileSync(lockFile, 'utf8'));
    const age = Date.now() - lockData.timestamp;
    const MAX_LOCK_AGE = 5 * 60 * 1000; // 5 分钟

    // 如果锁文件超过 5 分钟，或 PID 已不存在，清理锁
    if (age > MAX_LOCK_AGE || !isProcessRunning(lockData.pid)) {
      fs.unlinkSync(lockFile);
    }
  } catch (e) {
    // 锁文件损坏，直接删除
    try {
      fs.unlinkSync(lockFile);
    } catch (err) {
      // Ignore
    }
  }
}

// ==================== 启动时检查待更新 ====================

/**
 * 在执行二进制前，检查是否有待安装的更新
 * 如果有，执行更新并清理标记文件
 */
function checkAndInstallPendingUpdate() {
  const configDir = path.join(os.homedir(), '.claude', 'byebyecode');
  const pendingFile = path.join(configDir, '.update_pending');
  const lockFile = path.join(configDir, '.update_lock');

  // 如果没有待更新文件，直接返回
  if (!fs.existsSync(pendingFile)) {
    return;
  }

  // 清理过期锁文件
  cleanupStaleLock(lockFile);

  // 检查是否有其他进程正在更新
  if (fs.existsSync(lockFile)) {
    try {
      const lockData = JSON.parse(fs.readFileSync(lockFile, 'utf8'));

      // 如果锁文件对应的进程仍在运行，跳过更新
      if (isProcessRunning(lockData.pid)) {
        return;
      }
    } catch (e) {
      // 锁文件损坏，继续尝试更新
    }
  }

  // 创建锁文件
  try {
    fs.mkdirSync(configDir, { recursive: true });
    fs.writeFileSync(lockFile, JSON.stringify({
      pid: process.pid,
      timestamp: Date.now()
    }));
  } catch (e) {
    // 无法创建锁文件，跳过更新
    return;
  }

  // 读取待更新信息
  let pendingUpdate;
  try {
    pendingUpdate = JSON.parse(fs.readFileSync(pendingFile, 'utf8'));
  } catch (e) {
    // 文件损坏，清理后退出
    try {
      fs.unlinkSync(pendingFile);
      fs.unlinkSync(lockFile);
    } catch (err) {
      // Ignore
    }
    return;
  }

  // 执行更新
  console.error('');
  console.error('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.error(`🔄 正在更新 byebyecode 到 v${pendingUpdate.latestVersion}...`);
  console.error('   (这可能需要几秒钟)');
  console.error('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.error('');

  try {
    execSync('npm install -g @88code/byebyecode@latest', {
      stdio: 'inherit',
      timeout: 120000
    });

    console.error('');
    console.error('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.error(`✓ 更新成功！已安装 v${pendingUpdate.latestVersion}`);
    console.error('  正在继续执行...');
    console.error('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.error('');

    // 清理文件
    try {
      fs.unlinkSync(pendingFile);

      const noticeFile = path.join(configDir, '.update_notice');
      if (fs.existsSync(noticeFile)) {
        fs.unlinkSync(noticeFile);
      }
    } catch (e) {
      // Ignore cleanup errors
    }
  } catch (error) {
    console.error('');
    console.error('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.error('⚠ 自动更新失败');
    console.error('  请稍后重试，或手动运行:');
    console.error('  npm update -g @88code/byebyecode');
    console.error('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.error('');
    // 不删除 pending 文件，下次启动时继续尝试
  } finally {
    // 清理锁文件
    try {
      fs.unlinkSync(lockFile);
    } catch (e) {
      // Ignore
    }
  }
}

// ==================== 后台版本检查 ====================

/**
 * 检查是否有新版本可用
 * 如果有，创建 .update_pending 文件，不立即更新
 */
function checkVersionAndNotify() {
  const configDir = path.join(os.homedir(), '.claude', 'byebyecode');
  const versionCheckFile = path.join(configDir, '.last_version_check');
  const pendingFile = path.join(configDir, '.update_pending');
  const noticeFile = path.join(configDir, '.update_notice');

  try {
    // 频率限制：每小时最多检查一次
    if (fs.existsSync(versionCheckFile)) {
      const lastCheck = parseInt(fs.readFileSync(versionCheckFile, 'utf8'));
      const hoursSinceCheck = (Date.now() - lastCheck) / (1000 * 60 * 60);

      if (hoursSinceCheck < 1) {
        // 如果已有待更新提示，显示它
        if (fs.existsSync(noticeFile)) {
          const notice = fs.readFileSync(noticeFile, 'utf8');
          console.error(notice);
        }
        return;
      }
    }

    // 记录检查时间
    fs.mkdirSync(configDir, { recursive: true });
    fs.writeFileSync(versionCheckFile, Date.now().toString());

    // 获取当前版本
    const packageJsonPath = path.join(__dirname, '..', 'package.json');
    const currentVersion = require(packageJsonPath).version;

    // 从 npm registry 获取最新版本
    const latestVersion = execSync('npm view @88code/byebyecode version', {
      encoding: 'utf8',
      timeout: 5000,
      stdio: ['ignore', 'pipe', 'ignore']
    }).trim();

    // 发现新版本
    if (latestVersion && latestVersion !== currentVersion) {
      // 创建待更新文件
      fs.writeFileSync(pendingFile, JSON.stringify({
        currentVersion,
        latestVersion,
        detectedAt: Date.now()
      }));

      // 创建提示信息
      const notice = `
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📦 发现 byebyecode 新版本！
   当前版本: v${currentVersion}
   最新版本: v${latestVersion}

💡 更新将在您下次启动 Claude Code 时自动进行
   (或手动运行: npm update -g @88code/byebyecode)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      `.trim();

      fs.writeFileSync(noticeFile, notice);
      console.error(notice);
    } else {
      // 没有新版本，清理旧的提示文件
      if (fs.existsSync(pendingFile)) {
        fs.unlinkSync(pendingFile);
      }
      if (fs.existsSync(noticeFile)) {
        fs.unlinkSync(noticeFile);
      }
    }
  } catch (error) {
    // 静默忽略错误（网络问题等）
  }
}

// ==================== 主流程 ====================

// 步骤 1: 启动时检查并安装待更新
checkAndInstallPendingUpdate();

// 步骤 2: 确定平台对应的二进制
const platform = process.platform;
const arch = process.arch;

let platformKey = `${platform}-${arch}`;
if (platform === 'linux') {
  // 检测是否需要静态链接版本 (glibc < 2.35)
  function shouldUseStaticBinary() {
    try {
      const lddOutput = execSync('ldd --version 2>/dev/null || echo ""', {
        encoding: 'utf8',
        timeout: 1000
      });

      const match = lddOutput.match(/(?:GNU libc|GLIBC).*?(\d+)\.(\d+)/);
      if (match) {
        const major = parseInt(match[1]);
        const minor = parseInt(match[2]);
        return major < 2 || (major === 2 && minor < 35);
      }
    } catch (e) {
      return false;
    }
    return false;
  }

  if (shouldUseStaticBinary()) {
    platformKey = 'linux-x64-musl';
  }
}

const packageMap = {
  'darwin-x64': '@88code/byebyecode-darwin-x64',
  'darwin-arm64': '@88code/byebyecode-darwin-arm64',
  'linux-x64': '@88code/byebyecode-linux-x64',
  'linux-x64-musl': '@88code/byebyecode-linux-x64-musl',
  'win32-x64': '@88code/byebyecode-win32-x64',
  'win32-ia32': '@88code/byebyecode-win32-x64',
};

const packageName = packageMap[platformKey];
if (!packageName) {
  console.error(`Error: Unsupported platform ${platformKey}`);
  console.error('Supported platforms: darwin (x64/arm64), linux (x64), win32 (x64)');
  console.error('Please visit https://github.com/byebye-code/byebyecode for manual installation');
  process.exit(1);
}

const binaryName = platform === 'win32' ? 'byebyecode.exe' : 'byebyecode';
// 步骤 3: 确定二进制文件路径
// 优先级: 
// 1. ~/.claude/byebyecode/byebyecode (由 postinstall 安装或手动安装)
// 2. node_modules 中的对应包 (支持 npm/yarn/pnpm)

const globalConfigDir = path.join(os.homedir(), '.claude', 'byebyecode');
const globalBinaryPath = path.join(globalConfigDir, binaryName);

// 查找二进制文件的辅助函数 (支持 pnpm)
const findBinaryPathInNodeModules = () => {
  const possiblePaths = [
    // npm/yarn: nested in node_modules
    path.join(__dirname, '..', 'node_modules', packageName, binaryName),
    // pnpm: try require.resolve first
    (() => {
      try {
        const packagePath = require.resolve(packageName + '/package.json');
        return path.join(path.dirname(packagePath), binaryName);
      } catch {
        return null;
      }
    })(),
    // pnpm: flat structure fallback with version detection
    (() => {
      const currentPath = __dirname;
      const pnpmMatch = currentPath.match(/(.+\.pnpm)[\\/]([^\\//]+)[\\/]/);
      if (pnpmMatch) {
        const pnpmRoot = pnpmMatch[1];
        const packageNameEncoded = packageName.replace('/', '+');
        
        try {
          // Try to find any version of the package
          const pnpmContents = fs.readdirSync(pnpmRoot);
          const packagePattern = new RegExp(`^${packageNameEncoded.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}@`);
          const matchingPackage = pnpmContents.find(dir => packagePattern.test(dir));
          
          if (matchingPackage) {
            return path.join(pnpmRoot, matchingPackage, 'node_modules', packageName, binaryName);
          }
        } catch {
          // Fallback to current behavior if directory reading fails
        }
      }
      return null;
    })()
  ].filter(p => p !== null);

  for (const testPath of possiblePaths) {
    if (fs.existsSync(testPath)) {
      return testPath;
    }
  }
  return null;
};

let binaryPath;

// 1. 检查全局配置目录
if (fs.existsSync(globalBinaryPath)) {
  binaryPath = globalBinaryPath;
} else {
  // 2. 检查 node_modules
  binaryPath = findBinaryPathInNodeModules();
}

if (!binaryPath || !fs.existsSync(binaryPath)) {
  console.error(`Error: Binary not found for platform ${platformKey}`);
  console.error(`Expected package: ${packageName}`);
  console.error(`Expected binary: ${binaryName}`);
  console.error('');
  console.error('Troubleshooting:');
  console.error('1. Try reinstalling with force:');
  console.error('   npm install -g @88code/byebyecode --force');
  console.error('');
  console.error('2. If using pnpm, try installing with --shamefully-hoist:');
  console.error('   pnpm add -g @88code/byebyecode --shamefully-hoist');
  console.error('');
  console.error('3. Manually download the binary from GitHub Releases and place it at:');
  console.error(`   ${globalBinaryPath}`);
  
  process.exit(1);
}

// 步骤 3: 执行二进制
const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  shell: false
});

// 步骤 4: 执行完毕后，异步检查版本
setImmediate(() => checkVersionAndNotify());

process.exit(result.status || 0);
