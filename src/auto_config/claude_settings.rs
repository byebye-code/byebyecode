use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

/// 自动配置 Claude Code settings.json
pub struct ClaudeSettingsConfigurator;

impl ClaudeSettingsConfigurator {
    /// 获取 Claude settings.json 的路径
    pub fn get_settings_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".claude").join("settings.json"))
    }

    /// 获取当前二进制文件的绝对路径
    pub fn get_binary_path() -> Result<String, Box<dyn std::error::Error>> {
        let exe_path = std::env::current_exe()?;
        let absolute_path = fs::canonicalize(exe_path)?;

        // 在 Windows 上处理路径
        #[cfg(target_os = "windows")]
        {
            let path_str = absolute_path.to_string_lossy().to_string();
            // 移除 Windows UNC 前缀 \\?\
            let clean_path = if path_str.starts_with(r"\\?\") {
                path_str
                    .strip_prefix(r"\\?\")
                    .unwrap_or(&path_str)
                    .to_string()
            } else {
                path_str
            };
            // 转换单反斜杠为双反斜杠 (JSON 转义)
            Ok(clean_path.replace("\\", "\\\\"))
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(absolute_path.to_string_lossy().to_string())
        }
    }

    /// 配置 statusLine 设置
    pub fn configure_statusline() -> Result<(), Box<dyn std::error::Error>> {
        let settings_path =
            Self::get_settings_path().ok_or("无法找到 Claude settings.json 路径")?;

        // 如果文件不存在，创建默认配置
        let mut settings: Value = if settings_path.exists() {
            let content = fs::read_to_string(&settings_path)?;
            serde_json::from_str(&content)?
        } else {
            json!({})
        };

        // 获取当前二进制路径
        let binary_path = Self::get_binary_path()?;

        let mut modified = false;

        // 检查 statusLine 字段是否存在
        if let Some(obj) = settings.as_object_mut() {
            if !obj.contains_key("statusLine") {
                // 不存在，添加新的 statusLine 配置
                obj.insert(
                    "statusLine".to_string(),
                    json!({
                        "type": "command",
                        "command": binary_path,
                        "padding": 0
                    }),
                );
                println!("✓ 已添加 statusLine 配置到 settings.json");
                modified = true;
            } else {
                // 已存在，检查 command 路径
                let mut needs_update = false;
                if let Some(status_line) = obj.get("statusLine") {
                    if let Some(sl_obj) = status_line.as_object() {
                        if let Some(current_command) = sl_obj.get("command") {
                            if let Some(cmd_str) = current_command.as_str() {
                                // 比较路径（忽略反斜杠转义差异）
                                let current_normalized = cmd_str.replace("\\\\", "\\");
                                let new_normalized = binary_path.replace("\\\\", "\\");
                                if current_normalized != new_normalized {
                                    needs_update = true;
                                } else {
                                    println!("✓ statusLine.command 已经是当前二进制路径，无需更新");
                                }
                            }
                        }
                    }
                }

                // 只在需要时更新
                if needs_update {
                    if let Some(status_line) = obj.get_mut("statusLine") {
                        if let Some(sl_obj) = status_line.as_object_mut() {
                            sl_obj.insert("command".to_string(), json!(binary_path));
                            println!("✓ 已更新 statusLine.command 路径");
                            modified = true;
                        }
                    }
                }
            }
        }

        // 只在有修改时写回文件
        if modified {
            let formatted = serde_json::to_string_pretty(&settings)?;
            fs::write(&settings_path, formatted)?;
            println!("✓ Claude settings.json 配置完成");
            println!("  路径: {}", settings_path.display());
        }

        Ok(())
    }
}
