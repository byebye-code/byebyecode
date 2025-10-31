use regex::Regex;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LocationResult {
    pub start_index: usize,
    pub end_index: usize,
    pub variable_name: Option<String>,
}

#[derive(Debug)]
pub struct ClaudeCodePatcher {
    file_content: String,
    file_path: String,
}

impl ClaudeCodePatcher {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = file_path.as_ref();
        let content = fs::read_to_string(path)?;

        Ok(Self {
            file_content: content,
            file_path: path.to_string_lossy().to_string(),
        })
    }

    /// Find the verbose property location in Claude Code's cli.js
    /// Based on the pattern from patching.ts getVerbosePropertyLocation function
    pub fn get_verbose_property_location(&self) -> Option<LocationResult> {
        // Step 1: Find createElement pattern with spinnerTip and overrideMessage
        let create_element_pattern =
            Regex::new(r"createElement\([$\w]+,\{[^}]+spinnerTip[^}]+overrideMessage[^}]+\}")
                .ok()?;

        let create_element_match = create_element_pattern.find(&self.file_content)?;
        let extracted_string =
            &self.file_content[create_element_match.start()..create_element_match.end()];

        println!(
            "Found createElement match at: {}-{}",
            create_element_match.start(),
            create_element_match.end()
        );
        println!(
            "Extracted string: {}",
            &extracted_string[..std::cmp::min(200, extracted_string.len())]
        );

        // Step 2: Find verbose property within the createElement match
        let verbose_pattern = Regex::new(r"verbose:[^,}]+").ok()?;
        let verbose_match = verbose_pattern.find(extracted_string)?;

        println!(
            "Found verbose match at: {}-{}",
            verbose_match.start(),
            verbose_match.end()
        );
        println!("Verbose string: {}", verbose_match.as_str());

        // Calculate absolute positions in the original file
        let absolute_verbose_start = create_element_match.start() + verbose_match.start();
        let absolute_verbose_end = absolute_verbose_start + verbose_match.len();

        Some(LocationResult {
            start_index: absolute_verbose_start,
            end_index: absolute_verbose_end,
            variable_name: None,
        })
    }

    /// Write the verbose property with new value
    pub fn write_verbose_property(
        &mut self,
        value: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let location = self
            .get_verbose_property_location()
            .ok_or("Failed to find verbose property location")?;

        let new_code = format!("verbose:{}", value);

        let new_content = format!(
            "{}{}{}",
            &self.file_content[..location.start_index],
            new_code,
            &self.file_content[location.end_index..]
        );

        self.show_diff(&new_code, location.start_index, location.end_index);
        self.file_content = new_content;

        Ok(())
    }

    /// Save the modified content back to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(&self.file_path, &self.file_content)?;
        Ok(())
    }

    /// Get a reference to the file content (for testing purposes)
    pub fn get_file_content(&self) -> &str {
        &self.file_content
    }

    /// Show a diff of the changes (for debugging)
    fn show_diff(&self, injected_text: &str, start_index: usize, end_index: usize) {
        let context_start = start_index.saturating_sub(50);
        let context_end_old = std::cmp::min(self.file_content.len(), end_index + 50);

        let old_before = &self.file_content[context_start..start_index];
        let old_changed = &self.file_content[start_index..end_index];
        let old_after = &self.file_content[end_index..context_end_old];

        println!("\n--- Verbose Property Diff ---");
        println!(
            "OLD: {}\x1b[31m{}\x1b[0m{}",
            old_before, old_changed, old_after
        );
        println!(
            "NEW: {}\x1b[32m{}\x1b[0m{}",
            old_before, injected_text, old_after
        );
        println!("--- End Diff ---\n");
    }

    /// Find the context low message location in Claude Code's cli.js
    /// Pattern: "Context low (",B,"% remaining) · Run /compact to compact & continue"
    /// where B is a variable name
    pub fn get_context_low_message_location(&self) -> Option<LocationResult> {
        // Pattern to match: "Context low (",{variable},"% remaining) · Run /compact to compact & continue"
        let context_low_pattern = Regex::new(
            r#""Context low \(",([^,]+),"% remaining\) · Run /compact to compact & continue""#,
        )
        .ok()?;

        let context_low_match = context_low_pattern.find(&self.file_content)?;

        println!(
            "Found context low match at: {}-{}",
            context_low_match.start(),
            context_low_match.end()
        );
        println!("Context low string: {}", context_low_match.as_str());

        // Extract the variable name from the capture group
        let captures = context_low_pattern.captures(&self.file_content)?;
        let variable_name = captures.get(1)?.as_str();

        println!("Variable name: {}", variable_name);

        Some(LocationResult {
            start_index: context_low_match.start(),
            end_index: context_low_match.end(),
            variable_name: Some(variable_name.to_string()),
        })
    }

    /// Core robust function locator using anchor-based expansion
    /// Uses stable text patterns to survive Claude Code version updates
    pub fn find_context_low_function_robust(&self) -> Option<LocationResult> {
        // Step 1: Locate stable anchor text that survives obfuscation
        let primary_anchor = "Context low (";
        let anchor_pos = self.file_content.find(primary_anchor)?;

        // Step 2: Search backward within reasonable range to find function declarations
        let search_range = 800; // Optimized range based on actual function size (~466 chars)
        let search_start = anchor_pos.saturating_sub(search_range);
        let backward_text = &self.file_content[search_start..anchor_pos];

        // Find the function declaration that contains our anchor
        let mut function_candidates = Vec::new();
        let mut start = 0;

        while let Some(func_pos) = backward_text[start..].find("function ") {
            let absolute_func_pos = search_start + start + func_pos;

            // Check if this function contains the expected stable patterns
            let func_to_anchor_text = &self.file_content[absolute_func_pos..anchor_pos + 100];

            if func_to_anchor_text.contains("tokenUsage:") {
                function_candidates.push(absolute_func_pos);
                println!("Found function candidate at: {}", absolute_func_pos);
            }

            start += func_pos + 9; // Move past "function "
        }

        // Use the closest function to anchor (last candidate found)
        if let Some(&func_start) = function_candidates.last() {
            println!("Selected function start at: {}", func_start);

            // We only need the function start for condition replacement
            // Return a minimal range that includes the condition
            let condition_search_end = anchor_pos + 100; // Small range after anchor

            Some(LocationResult {
                start_index: func_start,
                end_index: condition_search_end,
                variable_name: Some("context_function".to_string()),
            })
        } else {
            println!("❌ No suitable function candidate found");
            None
        }
    }

    /// Core robust condition locator that finds the if statement to patch
    /// Returns the exact location of 'if(!Q||D)return null' for replacement with 'if(true)return null'
    pub fn get_context_low_condition_location_robust(&self) -> Option<LocationResult> {
        // Find the function using stable patterns
        let function_location = self.find_context_low_function_robust()?;
        let function_content =
            &self.file_content[function_location.start_index..function_location.end_index];

        // Look for if condition pattern using regex - match any condition that returns null
        let if_pattern = Regex::new(r"if\([^)]+\)return null").ok()?;

        if let Some(if_match) = if_pattern.find(function_content) {
            let absolute_start = function_location.start_index + if_match.start();
            let absolute_end = function_location.start_index + if_match.end();

            println!("Found if condition: '{}'", if_match.as_str());

            Some(LocationResult {
                start_index: absolute_start,
                end_index: absolute_end,
                variable_name: Some(if_match.as_str().to_string()),
            })
        } else {
            println!("❌ Could not find if condition in context function");
            None
        }
    }

    /// Disable context low warnings by modifying the if condition to always return null
    /// Uses robust pattern matching based on stable identifiers
    pub fn disable_context_low_warnings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(location) = self.get_context_low_condition_location_robust() {
            let replacement_condition = "if(true)return null";

            let new_content = format!(
                "{}{}{}",
                &self.file_content[..location.start_index],
                replacement_condition,
                &self.file_content[location.end_index..]
            );

            self.show_diff(
                replacement_condition,
                location.start_index,
                location.end_index,
            );
            self.file_content = new_content;

            println!("✅ Context low warnings disabled successfully");
            Ok(())
        } else {
            Err("Could not locate context low condition using robust method".into())
        }
    }

    /// Write a replacement for the context low message
    pub fn write_context_low_message(
        &mut self,
        new_message: &str,
        variable_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let location = self
            .get_context_low_message_location()
            .ok_or("Failed to find context low message location")?;

        let new_code = format!(
            r#""{}","{}","{}""#,
            new_message.split(',').nth(0).unwrap_or(new_message),
            variable_name,
            new_message.split(',').nth(1).unwrap_or("")
        );

        let new_content = format!(
            "{}{}{}",
            &self.file_content[..location.start_index],
            new_code,
            &self.file_content[location.end_index..]
        );

        self.show_diff(&new_code, location.start_index, location.end_index);
        self.file_content = new_content;

        Ok(())
    }

    /// Find the ternary condition for esc/interrupt display
    /// Pattern: ...CONDITION?[...{key:"esc"}...,"to interrupt"...]:[]
    /// Returns the position of CONDITION that needs to be replaced with (false)
    fn find_esc_interrupt_condition(&self) -> Option<LocationResult> {
        let anchor1 = r#"{key:"esc"}"#;
        let anchor2 = r#""to interrupt""#;

        let mut search_start = 0;
        while let Some(anchor1_offset) = self.file_content[search_start..].find(anchor1) {
            let anchor1_pos = search_start + anchor1_offset;

            let search_window_end = (anchor1_pos + 200).min(self.file_content.len());
            let window = &self.file_content[anchor1_pos..search_window_end];

            if window.contains(anchor2) {
                println!(
                    "Found both anchors: {{key:\"esc\"}} at {} and \"to interrupt\" nearby",
                    anchor1_pos
                );

                let before_anchor = &self.file_content[..anchor1_pos];
                if let Some(spread_offset) = before_anchor.rfind("...") {
                    let spread_pos = spread_offset;
                    println!("  Found spread operator at: {}", spread_pos);

                    let between_spread_and_anchor = &self.file_content[spread_pos..anchor1_pos];
                    if let Some(question_offset) = between_spread_and_anchor.find('?') {
                        let question_pos = spread_pos + question_offset;

                        let condition_start = spread_pos + 3;
                        let condition_end = question_pos;

                        let condition = &self.file_content[condition_start..condition_end];
                        println!(
                            "  Found condition '{}' at {}-{}",
                            condition.trim(),
                            condition_start,
                            condition_end
                        );

                        return Some(LocationResult {
                            start_index: condition_start,
                            end_index: condition_end,
                            variable_name: Some(condition.trim().to_string()),
                        });
                    }
                }
            }

            search_start = anchor1_pos + 1;
        }

        None
    }

    /// Disable "esc to interrupt" display by replacing ternary condition with (false)
    /// Changes: ...H1?[esc elements]:[] → ...(false)?[esc elements]:[]
    pub fn disable_esc_interrupt_display(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let location = self
            .find_esc_interrupt_condition()
            .ok_or("Could not find esc/interrupt ternary condition")?;

        let original_condition = location
            .variable_name
            .as_ref()
            .ok_or("No condition variable found")?;

        println!(
            "Replacing condition '{}' with '(false)' at position {}-{}",
            original_condition, location.start_index, location.end_index
        );

        self.show_diff("(false)", location.start_index, location.end_index);

        let new_content = format!(
            "{}(false){}",
            &self.file_content[..location.start_index],
            &self.file_content[location.end_index..]
        );

        self.file_content = new_content;
        println!("✅ ESC interrupt display disabled successfully");

        Ok(())
    }

    /// Find statusline command execution location for injecting auto-refresh
    /// Searches for the statusline.command execution pattern
    fn find_statusline_execution_location(&self) -> Option<LocationResult> {
        // Look for patterns that indicate statusline execution
        // Pattern 1: execSync or spawn with statusline command
        let patterns = vec![
            r"execSync\([^)]*statusLine",
            r"spawn\([^)]*statusLine",
            r"\.command\s*&&\s*execSync",
        ];

        for pattern_str in patterns {
            if let Ok(pattern) = Regex::new(pattern_str) {
                if let Some(match_result) = pattern.find(&self.file_content) {
                    println!("Found statusline execution pattern: {}", pattern_str);
                    println!("Match: {}", match_result.as_str());

                    return Some(LocationResult {
                        start_index: match_result.start(),
                        end_index: match_result.end(),
                        variable_name: Some(match_result.as_str().to_string()),
                    });
                }
            }
        }

        // Fallback: search for any function that contains "statusLine"
        if let Some(statusline_pos) = self.file_content.find("statusLine") {
            println!("Found statusLine reference at position: {}", statusline_pos);

            // Find the async function definition before this reference
            let search_start = statusline_pos.saturating_sub(300);
            let search_text = &self.file_content[search_start..statusline_pos];

            if let Some(func_pos) = search_text.rfind("async function ") {
                let absolute_func_pos = search_start + func_pos;
                println!("Found async function at: {}", absolute_func_pos);

                // Find the END of this function - look for pattern: }async or }function
                let search_end = (statusline_pos + 3000).min(self.file_content.len());
                let remaining_text = &self.file_content[statusline_pos..search_end];

                // Look for function end: } followed by 'async' or 'function' or capital letter
                if let Ok(end_pattern) = Regex::new(r"\}\s*(async|function|[A-Z])") {
                    if let Some(end_match) = end_pattern.find(remaining_text) {
                        let injection_pos = statusline_pos + end_match.start() + 1; // After '}'
                        println!("Found function end at position: {}", injection_pos);

                        return Some(LocationResult {
                            start_index: absolute_func_pos,
                            end_index: injection_pos,
                            variable_name: Some("statusline_function_end".to_string()),
                        });
                    }
                }
            }
        }

        None
    }

    /// Extract the signal handler initialization function name
    /// Looks for the function that sets up SIGINT and SIGTERM handlers
    /// Pattern: var XYZ=AA(()=>{process.on("SIGINT"...process.on("SIGTERM"...)})
    fn extract_signal_handler_init_function(&self) -> Option<String> {
        // Strategy 1: Look for SIGINT followed by SIGTERM within reasonable distance
        // More lenient pattern to handle nested parentheses and different formatting
        if let Some(sigint_pos) = self.file_content.find(r#"process.on("SIGINT""#) {
            // Search backward for variable declaration
            let search_start = sigint_pos.saturating_sub(500);
            let before_text = &self.file_content[search_start..sigint_pos];

            // Find the last variable declaration before SIGINT
            if let Ok(var_pattern) = Regex::new(r"var\s+([a-zA-Z0-9_]+)\s*=") {
                if let Some(captures_iter) = var_pattern.captures_iter(before_text).last() {
                    let func_name = captures_iter.get(1)?.as_str();

                    // Verify this is the right function by checking if SIGTERM appears nearby
                    let check_end = (sigint_pos + 200).min(self.file_content.len());
                    let check_text = &self.file_content[sigint_pos..check_end];

                    if check_text.contains(r#"process.on("SIGTERM""#) {
                        println!("🎯 Found signal handler init function: {}", func_name);
                        println!("   Located via SIGINT/SIGTERM pattern");
                        return Some(func_name.to_string());
                    }
                }
            }
        }

        println!("❌ Could not find signal handler init function");
        None
    }

    /// Extract the function name that handles statusline execution
    /// Searches for: async function XYZ(A,B,Q=...){...statusLine...}
    /// Uses multiple strategies to handle different Claude Code versions
    fn extract_statusline_function_name(&self, _from_pos: usize) -> Option<String> {
        // Strategy 1: Look for the exact pattern with nA()?.statusLine
        // This is the most specific pattern found in current versions
        let specific_pattern =
            Regex::new(r"async function ([a-zA-Z0-9_]+)\([^)]*\)\{[^}]*nA\(\)\?\.statusLine")
                .ok()?;
        if let Some(capture) = specific_pattern.captures(&self.file_content) {
            let func_name = capture.get(1)?.as_str();
            println!(
                "🎯 Found statusline function (strategy 1 - nA pattern): {}",
                func_name
            );
            return Some(func_name.to_string());
        }

        // Strategy 2: Look for function that contains both statusLine and Ye1 (hook executor)
        let hook_pattern = Regex::new(
            r"async function ([a-zA-Z0-9_]+)\([^)]*\)\{[^}]{0,500}statusLine[^}]{0,500}Ye1",
        )
        .ok()?;
        if let Some(capture) = hook_pattern.captures(&self.file_content) {
            let func_name = capture.get(1)?.as_str();
            println!(
                "🎯 Found statusline function (strategy 2 - hook pattern): {}",
                func_name
            );
            return Some(func_name.to_string());
        }

        // Strategy 3: Original pattern - function with statusLine close to definition
        let pattern =
            Regex::new(r"async function ([a-zA-Z0-9_]+)\([^)]*\)\{[^}]{0,200}statusLine").ok()?;
        if let Some(capture) = pattern.captures(&self.file_content) {
            let func_name = capture.get(1)?.as_str();
            println!(
                "🎯 Found statusline function (strategy 3 - close proximity): {}",
                func_name
            );
            return Some(func_name.to_string());
        }

        // Strategy 4: Broader search - last async function before statusLine reference
        if let Some(statusline_pos) = self.file_content.find("statusLine") {
            let search_start = statusline_pos.saturating_sub(500); // Search further back
            let search_text = &self.file_content[search_start..statusline_pos];

            if let Ok(func_pattern) = Regex::new(r"async function ([a-zA-Z0-9_]+)\(") {
                let mut last_match = None;
                for capture in func_pattern.captures_iter(search_text) {
                    last_match = Some(capture);
                }

                if let Some(capture) = last_match {
                    let func_name = capture.get(1)?.as_str();
                    println!(
                        "🎯 Found statusline function (strategy 4 - last async func): {}",
                        func_name
                    );
                    return Some(func_name.to_string());
                }
            }
        }

        println!("❌ Could not extract statusline function name with any strategy");
        None
    }

    /// Add auto-refresh interval for statusline
    /// Injects a setInterval that periodically refreshes the statusline display
    pub fn add_statusline_refresh_interval(
        &mut self,
        interval_ms: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if already patched
        if self.file_content.contains("setInterval(function(){try{")
            && (self.file_content.contains("VZA({})")
                || self.file_content.contains("refreshStatusLine"))
        {
            println!("⚠️  Statusline auto-refresh already patched, skipping...");
            return Ok(());
        }

        // BEST STRATEGY: Inject after signal handler initialization
        // Find the function that sets up SIGINT and SIGTERM handlers, then find where it's called
        let injection_pos = if let Some(init_func_name) =
            self.extract_signal_handler_init_function()
        {
            println!("✅ Found signal handler init function: {}", init_func_name);

            // Now find where this function is called
            let call_pattern = format!("{}()", init_func_name);
            if let Some(call_pos) = self.file_content.find(&call_pattern) {
                println!("✅ Found {} call at position: {}", call_pattern, call_pos);

                // Find the try-catch block that contains this call
                // Look backward to find 'try{'
                let search_back_start = call_pos.saturating_sub(500);
                let before_text = &self.file_content[search_back_start..call_pos];

                if let Some(try_offset) = before_text.rfind("try{") {
                    let try_pos = search_back_start + try_offset;
                    println!("✅ Found try block at position: {}", try_pos);

                    // Now find the end of the try-catch block
                    // Look for pattern: }catch(...){...}
                    let search_forward_start = call_pos;
                    let remaining = &self.file_content[search_forward_start..];

                    // Find the matching closing brace for the try-catch
                    // Look for "}});" pattern which typically ends the function
                    if let Ok(end_pattern) = Regex::new(r"\}\}\);") {
                        if let Some(end_match) = end_pattern.find(remaining) {
                            let pos = search_forward_start + end_match.end();
                            println!("✅ Injecting after try-catch block at position: {}", pos);
                            pos
                        } else {
                            println!("⚠️  Could not find try-catch end, using fallback");
                            return Err("Could not find try-catch block end".into());
                        }
                    } else {
                        println!("⚠️  Regex error, using fallback");
                        return Err("Regex compilation failed".into());
                    }
                } else {
                    println!("⚠️  Could not find try block, using fallback");
                    return Err("Could not find try block".into());
                }
            } else {
                println!(
                    "⚠️  Could not find {}() call, using fallback",
                    init_func_name
                );
                return Err("Could not find signal handler init call".into());
            }
        } else if let Some(location) = self.find_statusline_execution_location() {
            // Strategy 2: Try to find statusline-specific location (original strategy)
            println!("✓ Using statusline-specific injection point");
            location.end_index
        } else {
            // Strategy 3: Fallback to general initialization patterns
            println!("⚠ Using fallback injection strategy");

            let init_patterns = vec!["process.on(\"SIGINT\"", "process.on(\"exit\"", ".render();"];

            let mut injection_point = None;

            for pattern in init_patterns {
                if let Some(pos) = self.file_content.rfind(pattern) {
                    let search_start = pos + pattern.len();
                    let remaining = &self.file_content[search_start..];

                    if let Some(semicolon_offset) = remaining.find(';') {
                        injection_point = Some(search_start + semicolon_offset + 1);
                        println!(
                            "Found injection point after: {} at position {}",
                            pattern,
                            injection_point.unwrap()
                        );
                        break;
                    }
                }
            }

            injection_point.ok_or("Could not find suitable injection point")?
        };

        // Create the refresh interval code
        // Strategy: Find the statusline execution function and call it periodically
        // Look for the function that was just defined (likely contains statusLine)
        let refresh_code = if let Some(func_name) =
            self.extract_statusline_function_name(injection_pos)
        {
            // Call the discovered function with empty object as parameter
            format!(
                "setInterval(function(){{try{{{}({{}})}}catch(e){{}}}},{});",
                func_name, interval_ms
            )
        } else {
            // Fallback: try common patterns
            format!(
                "setInterval(function(){{try{{if(typeof refreshStatusLine==='function')refreshStatusLine();else if(typeof updateStatusLine==='function')updateStatusLine();}}catch(e){{}}}},{});",
                interval_ms
            )
        };

        println!("\n🔄 Injecting statusline auto-refresh...");
        println!("Interval: {}ms ({}s)", interval_ms, interval_ms / 1000);
        println!("Code: {}", refresh_code);

        // Show context around injection point
        let context_start = injection_pos.saturating_sub(100);
        let context_end = (injection_pos + 100).min(self.file_content.len());

        println!("\n--- Injection Context ---");
        println!(
            "BEFORE: {}",
            &self.file_content[context_start..injection_pos]
        );
        println!(">>> INJECT: \x1b[32m{}\x1b[0m", refresh_code);
        println!("AFTER: {}", &self.file_content[injection_pos..context_end]);
        println!("--- End Context ---\n");

        // Inject the code
        let new_content = format!(
            "{}{}{}",
            &self.file_content[..injection_pos],
            refresh_code,
            &self.file_content[injection_pos..]
        );

        self.file_content = new_content;
        println!("✅ Statusline auto-refresh interval added successfully");

        Ok(())
    }
}
