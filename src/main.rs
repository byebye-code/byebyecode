use byebyecode::auto_config::AutoConfigurator;
use byebyecode::cli::Cli;
use byebyecode::config::{Config, InputData};
use byebyecode::core::{collect_all_segments, StatusLineGenerator};
use byebyecode::translation::TranslationConfig;
use byebyecode::wrapper::{find_claude_code, injector::ClaudeCodeInjector};
use std::io::{self, IsTerminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse_args();

    // Handle wrapper mode - inject into Claude Code
    if cli.wrap {
        return run_wrapper_mode(&cli);
    }

    // Handle auto-configuration
    if cli.glm_key.is_some() {
        let configurator = AutoConfigurator::new()?;
        configurator.setup_byebyecode(None, cli.glm_key.clone())?;
        configurator.install_binary()?;
        println!("\n✓ byebyecode 自动配置完成!");
        println!("  使用 'byebyecode --wrap' 启动包装模式");
        return Ok(());
    }

    // Handle configuration commands
    if cli.init {
        Config::init()?;

        // 自动配置 Claude Code settings.json
        println!("\n正在配置 Claude Code settings.json...");
        match byebyecode::auto_config::ClaudeSettingsConfigurator::configure_statusline() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("⚠ 配置 Claude settings.json 失败: {}", e);
                eprintln!("  你可以手动配置 statusLine 字段");
            }
        }

        return Ok(());
    }

    if cli.print {
        let mut config = Config::load().unwrap_or_else(|_| Config::default());

        // Apply theme override if provided
        if let Some(theme) = cli.theme {
            config = byebyecode::ui::themes::ThemePresets::get_theme(&theme);
        }

        config.print()?;
        return Ok(());
    }

    if cli.check {
        let config = Config::load()?;
        config.check()?;
        println!("✓ Configuration valid");
        return Ok(());
    }

    if cli.config {
        #[cfg(feature = "tui")]
        {
            byebyecode::ui::run_configurator()?;
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("TUI feature is not enabled. Please install with --features tui");
            std::process::exit(1);
        }
        return Ok(());
    }

    if cli.update {
        #[cfg(feature = "self-update")]
        {
            println!("Update feature not implemented in new architecture yet");
        }
        #[cfg(not(feature = "self-update"))]
        {
            println!("Update check not available (self-update feature disabled)");
        }
        return Ok(());
    }

    // Handle Claude Code patcher
    if let Some(claude_path) = cli.patch {
        use byebyecode::utils::ClaudeCodePatcher;

        println!("🔧 Claude Code Context Warning Disabler");
        println!("Target file: {}", claude_path);

        // Create backup in same directory
        let backup_path = format!("{}.backup", claude_path);
        std::fs::copy(&claude_path, &backup_path)?;
        println!("📦 Created backup: {}", backup_path);

        // Load and patch
        let mut patcher = ClaudeCodePatcher::new(&claude_path)?;

        // Apply all modifications
        println!("\n🔄 Applying patches...");

        // 1. Set verbose property to true
        if let Err(e) = patcher.write_verbose_property(true) {
            println!("⚠️ Could not modify verbose property: {}", e);
        }

        // 2. Disable context low warnings
        patcher.disable_context_low_warnings()?;

        // 3. Disable ESC interrupt display
        if let Err(e) = patcher.disable_esc_interrupt_display() {
            println!("⚠️ Could not disable esc/interrupt display: {}", e);
        }

        patcher.save()?;

        println!("✅ All patches applied successfully!");
        println!("💡 To restore warnings, replace your cli.js with the backup file:");
        println!("   cp {} {}", backup_path, claude_path);

        return Ok(());
    }

    // Load configuration
    let mut config = Config::load().unwrap_or_else(|_| Config::default());

    // Apply theme override if provided
    if let Some(theme) = cli.theme {
        config = byebyecode::ui::themes::ThemePresets::get_theme(&theme);
    }

    // Check if stdin has data
    if io::stdin().is_terminal() {
        // No input data available, show main menu
        #[cfg(feature = "tui")]
        {
            use byebyecode::ui::{MainMenu, MenuResult};

            if let Some(result) = MainMenu::run()? {
                match result {
                    MenuResult::LaunchConfigurator => {
                        byebyecode::ui::run_configurator()?;
                    }
                    MenuResult::InitConfig => {
                        byebyecode::config::Config::init()?;
                        println!("Configuration initialized successfully!");
                    }
                    MenuResult::CheckConfig => {
                        let config = byebyecode::config::Config::load()?;
                        config.check()?;
                        println!("Configuration is valid!");
                    }
                    MenuResult::Exit => {
                        // Exit gracefully
                    }
                }
            }
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("No input data provided and TUI feature is not enabled.");
            eprintln!("Usage: echo '{{...}}' | ccline");
            eprintln!("   or: ccline --help");
        }
        return Ok(());
    }

    // Read Claude Code data from stdin
    let stdin = io::stdin();
    let input: InputData = serde_json::from_reader(stdin.lock())?;

    // Collect segment data
    let segments_data = collect_all_segments(&config, &input);

    // Render statusline
    let generator = StatusLineGenerator::new(config);
    let statusline = generator.generate(segments_data);

    println!("{}", statusline);

    Ok(())
}

fn run_wrapper_mode(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Find Claude Code executable
    let claude_path = find_claude_code()?;
    println!("✓ Found Claude Code at: {}", claude_path.display());

    // Load API keys from config
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let keys_path = home.join(".claude").join("88code").join("api_keys.toml");

    let (_api_key, glm_key) = if keys_path.exists() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct ApiKeys {
            #[serde(default)]
            byebyecode_api_key: Option<String>,
            #[serde(default)]
            glm_api_key: Option<String>,
        }

        let content = std::fs::read_to_string(&keys_path)?;
        let keys: ApiKeys = toml::from_str(&content)?;
        (keys.byebyecode_api_key, keys.glm_api_key)
    } else {
        (None, None)
    };

    // Use keys from config (no CLI override for api_key)
    let final_glm_key = cli.glm_key.clone().or(glm_key);

    // Setup translation config
    let translation_config = if cli.translation.unwrap_or(true) {
        if let Some(glm_key) = final_glm_key {
            Some(TranslationConfig {
                enabled: true,
                api_key: glm_key,
                ..Default::default()
            })
        } else {
            println!("⚠️  翻译功能需要 GLM API Key. 使用 --glm-key 设置");
            None
        }
    } else {
        None
    };

    // Store whether translation is enabled for later display
    let translation_enabled = translation_config.is_some();

    // Create and run injector
    let mut injector = ClaudeCodeInjector::new(claude_path, translation_config)?;

    // Get remaining args to pass to Claude Code
    let claude_args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--wrap")
        .collect();

    println!("\n🚀 启动 Claude Code...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯 88code wrapper 已激活");
    if translation_enabled {
        println!("🌐 翻译功能: 已启用");
    } else {
        println!("🌐 翻译功能: 未启用 (使用 --glm-key 设置)");
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    injector.run_with_interception(claude_args)
}
