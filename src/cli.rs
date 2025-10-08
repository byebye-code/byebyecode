use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "byebyecode")]
#[command(
    version,
    about = "byebyecode - Claude Code Wrapper with Translation & Enhanced Features"
)]
pub struct Cli {
    /// Enter TUI configuration mode
    #[arg(short = 'c', long = "config")]
    pub config: bool,

    /// Set theme
    #[arg(short = 't', long = "theme")]
    pub theme: Option<String>,

    /// Print current configuration
    #[arg(long = "print")]
    pub print: bool,

    /// Initialize config file
    #[arg(long = "init")]
    pub init: bool,

    /// Check configuration
    #[arg(long = "check")]
    pub check: bool,

    /// Check for updates
    #[arg(short = 'u', long = "update")]
    pub update: bool,

    /// Patch Claude Code cli.js to disable context warnings
    #[arg(long = "patch")]
    pub patch: Option<String>,

    /// Start byebyecode wrapper mode (inject into Claude Code)
    #[arg(long = "wrap")]
    pub wrap: bool,

    /// Enable/disable translation feature
    #[arg(long = "translation")]
    pub translation: Option<bool>,

    /// Set GLM API key for translation
    #[arg(long = "glm-key")]
    pub glm_key: Option<String>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
