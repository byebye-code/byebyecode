use crate::translation::glm::GLMTranslator;
use crate::translation::{TranslationConfig, Translator};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

pub struct ClaudeCodeInjector {
    claude_path: PathBuf,
    translation_enabled: bool,
    translator: Option<GLMTranslator>,
}

impl ClaudeCodeInjector {
    pub fn new(
        claude_path: PathBuf,
        translation_config: Option<TranslationConfig>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (translation_enabled, translator) = if let Some(config) = translation_config {
            if config.enabled && !config.api_key.is_empty() {
                let translator = GLMTranslator::new(config)?;
                (true, Some(translator))
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };

        Ok(Self {
            claude_path,
            translation_enabled,
            translator,
        })
    }

    pub fn start(&self, args: Vec<String>) -> Result<Child, Box<dyn std::error::Error>> {
        let mut cmd = if cfg!(target_os = "windows")
            && self
                .claude_path
                .extension()
                .map_or(false, |ext| ext == "cmd")
        {
            // On Windows, .cmd files need to be run through cmd.exe
            let mut c = Command::new("cmd");
            c.arg("/C");
            c.arg(&self.claude_path);
            c
        } else {
            Command::new(&self.claude_path)
        };

        cmd.args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variable to indicate wrapper is active
        cmd.env("BYEBYECODE_WRAPPER", "1");
        cmd.env("BYEBYECODE_VERSION", env!("CARGO_PKG_VERSION"));

        let child = cmd.spawn()?;
        Ok(child)
    }

    pub fn intercept_input(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        if self.translation_enabled {
            if let Some(translator) = &self.translator {
                // Detect if input contains Chinese characters
                if input
                    .chars()
                    .any(|c| matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}'))
                {
                    // Translate Chinese to English
                    return translator.translate_to_english(input);
                }
            }
        }
        Ok(input.to_string())
    }

    pub fn intercept_output(&self, output: &str) -> Result<String, Box<dyn std::error::Error>> {
        if self.translation_enabled {
            if let Some(translator) = &self.translator {
                // Translate English response to Chinese
                let chinese = translator.translate_to_chinese(output)?;

                // Format: Chinese first, then original English
                Ok(format!("{}\n\n[原文] {}", chinese, output))
            } else {
                Ok(output.to_string())
            }
        } else {
            Ok(output.to_string())
        }
    }

    pub fn run_with_interception(
        &mut self,
        args: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // If translation is not enabled, just run Claude Code directly without interception
        if !self.translation_enabled {
            let mut cmd = if cfg!(target_os = "windows")
                && self
                    .claude_path
                    .extension()
                    .map_or(false, |ext| ext == "cmd")
            {
                let mut c = Command::new("cmd");
                c.arg("/C");
                c.arg(&self.claude_path);
                c
            } else {
                Command::new(&self.claude_path)
            };

            cmd.args(&args);

            // Set environment variable to indicate wrapper is active
            cmd.env("BYEBYECODE_WRAPPER", "1");
            cmd.env("BYEBYECODE_VERSION", env!("CARGO_PKG_VERSION"));

            let status = cmd.status()?;

            if !status.success() {
                return Err(format!("Claude Code exited with status: {}", status).into());
            }

            return Ok(());
        }

        // Translation enabled - intercept I/O
        let mut child = self.start(args)?;

        let stdin = child.stdin.take().ok_or("Failed to capture stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        // Spawn thread to handle stdout
        let translator_clone = self.translator.clone();
        let translation_enabled = self.translation_enabled;

        let stdout_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if translation_enabled {
                        if let Some(ref translator) = translator_clone {
                            if let Ok(translated) = translator.translate_to_chinese(&line) {
                                println!("{}", translated);
                                println!("[原文] {}", line);
                                continue;
                            }
                        }
                    }
                    println!("{}", line);
                }
            }
        });

        // Spawn thread to handle stderr
        let stderr_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("{}", line);
                }
            }
        });

        // Handle stdin
        let stdin_handle = std::thread::spawn(move || {
            let mut stdin_writer = stdin;
            let stdin_reader = std::io::stdin();
            for line in stdin_reader.lock().lines() {
                if let Ok(line) = line {
                    if let Err(e) = writeln!(stdin_writer, "{}", line) {
                        eprintln!("Error writing to Claude Code stdin: {}", e);
                        break;
                    }
                }
            }
        });

        // Wait for child process
        let status = child.wait()?;

        // Wait for threads
        let _ = stdout_handle.join();
        let _ = stderr_handle.join();
        let _ = stdin_handle.join();

        if !status.success() {
            return Err(format!("Claude Code exited with status: {}", status).into());
        }

        Ok(())
    }
}
