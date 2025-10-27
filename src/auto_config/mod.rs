use crate::config::Config;
use std::fs;
use std::path::PathBuf;

pub mod claude_settings;
pub use claude_settings::ClaudeSettingsConfigurator;

pub struct AutoConfigurator {
    config_dir: PathBuf,
}

impl AutoConfigurator {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;

        let config_dir = home.join(".claude/byebyecode");
        Ok(Self { config_dir })
    }

    pub fn ensure_config_dir(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)?;
            println!("✓ Created config directory: {}", self.config_dir.display());
        }
        Ok(())
    }

    pub fn setup_byebyecode(
        &self,
        api_key: Option<String>,
        glm_key: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_config_dir()?;

        // Load or create default config
        let mut config = Config::load().unwrap_or_else(|_| Config::default());

        // Add byebyecode segments if not present
        use crate::config::{
            AnsiColor, ColorConfig, IconConfig, SegmentConfig, SegmentId, TextStyleConfig,
        };
        use std::collections::HashMap;

        let has_usage = config
            .segments
            .iter()
            .any(|s| matches!(s.id, SegmentId::ByeByeCodeUsage));
        let has_sub = config
            .segments
            .iter()
            .any(|s| matches!(s.id, SegmentId::ByeByeCodeSubscription));

        if !has_usage {
            let mut options = HashMap::new();
            if let Some(key) = &api_key {
                options.insert(
                    "api_key".to_string(),
                    serde_json::Value::String(key.clone()),
                );
            }

            config.segments.push(SegmentConfig {
                id: SegmentId::ByeByeCodeUsage,
                enabled: true,
                icon: IconConfig {
                    plain: "88code".to_string(),
                    nerd_font: "".to_string(),
                },
                colors: ColorConfig {
                    icon: Some(AnsiColor::Color256 { c256: 214 }),
                    text: Some(AnsiColor::Color256 { c256: 255 }),
                    background: Some(AnsiColor::Color256 { c256: 236 }),
                },
                styles: TextStyleConfig { text_bold: false },
                options,
            });
            println!("✓ 已添加 88code 用量监控段");
        }

        if !has_sub {
            let mut options = HashMap::new();
            if let Some(key) = &api_key {
                options.insert(
                    "api_key".to_string(),
                    serde_json::Value::String(key.clone()),
                );
            }

            config.segments.push(SegmentConfig {
                id: SegmentId::ByeByeCodeSubscription,
                enabled: true,
                icon: IconConfig {
                    plain: "订阅".to_string(),
                    nerd_font: "".to_string(),
                },
                colors: ColorConfig {
                    icon: Some(AnsiColor::Color256 { c256: 39 }),
                    text: Some(AnsiColor::Color256 { c256: 255 }),
                    background: Some(AnsiColor::Color256 { c256: 236 }),
                },
                styles: TextStyleConfig { text_bold: false },
                options,
            });
            println!("✓ 已添加 88code 订阅信息段");
        }

        // Save config
        let config_path = self.config_dir.join("config.toml");
        let toml_string = toml::to_string_pretty(&config)?;
        fs::write(&config_path, toml_string)?;
        println!("✓ Configuration saved to: {}", config_path.display());

        // Save API keys to separate config file
        if api_key.is_some() || glm_key.is_some() {
            use serde::{Deserialize, Serialize};

            #[derive(Serialize, Deserialize)]
            struct ApiKeys {
                #[serde(skip_serializing_if = "Option::is_none")]
                byebyecode_api_key: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                glm_api_key: Option<String>,
            }

            let keys = ApiKeys {
                byebyecode_api_key: api_key,
                glm_api_key: glm_key,
            };

            let keys_path = self.config_dir.join("api_keys.toml");
            let keys_toml = toml::to_string_pretty(&keys)?;
            fs::write(&keys_path, keys_toml)?;
            println!("✓ API keys saved to: {}", keys_path.display());
        }

        Ok(())
    }

    pub fn install_binary(&self) -> Result<(), Box<dyn std::error::Error>> {
        let current_exe = std::env::current_exe()?;
        let target_path = self.config_dir.join(if cfg!(windows) {
            "byebyecode.exe"
        } else {
            "byebyecode"
        });

        if target_path.exists() {
            println!("Binary already exists at: {}", target_path.display());
            println!("Do you want to overwrite? (y/n)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                return Ok(());
            }
        }

        fs::copy(&current_exe, &target_path)?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&target_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&target_path, perms)?;
        }

        println!("✓ Binary installed to: {}", target_path.display());
        println!(
            "  Add this to your PATH or use directly: {}",
            target_path.display()
        );

        Ok(())
    }
}
