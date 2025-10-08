pub mod injector;
pub mod io_interceptor;

use std::path::PathBuf;

/// Find Claude Code executable from PATH environment variable
pub fn find_claude_code() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Try to find 'claude' command in PATH
    match which::which("claude") {
        Ok(path) => Ok(path),
        Err(_) => {
            // Try common installation paths on different platforms
            #[cfg(target_os = "windows")]
            {
                // Windows: Check AppData locations
                if let Ok(appdata) = std::env::var("APPDATA") {
                    let claude_path = PathBuf::from(appdata).join("npm").join("claude.cmd");
                    if claude_path.exists() {
                        return Ok(claude_path);
                    }
                }
            }

            #[cfg(target_os = "macos")]
            {
                // macOS: Check common npm global paths
                let paths = vec!["/usr/local/bin/claude", "/opt/homebrew/bin/claude"];
                for path in paths {
                    let p = PathBuf::from(path);
                    if p.exists() {
                        return Ok(p);
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                // Linux: Check common paths
                let paths = vec!["/usr/local/bin/claude", "/usr/bin/claude"];
                for path in paths {
                    let p = PathBuf::from(path);
                    if p.exists() {
                        return Ok(p);
                    }
                }
            }

            Err("Claude Code executable not found in PATH or common locations".into())
        }
    }
}
