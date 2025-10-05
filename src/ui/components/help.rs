use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Default)]
pub struct HelpComponent;

impl HelpComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        status_message: Option<&str>,
        color_picker_open: bool,
        icon_selector_open: bool,
    ) {
        let help_items = if color_picker_open {
            vec![
                ("[↑↓]", "导航"),
                ("[Tab]", "模式"),
                ("[Enter]", "选择"),
                ("[Esc]", "取消"),
            ]
        } else if icon_selector_open {
            vec![
                ("[↑↓]", "导航"),
                ("[Tab]", "样式"),
                ("[C]", "自定义"),
                ("[Enter]", "选择"),
                ("[Esc]", "取消"),
            ]
        } else {
            vec![
                ("[Tab]", "切换面板"),
                ("[Enter]", "切换/编辑"),
                ("[Shift+↑↓]", "重新排序"),
                ("[1-4]", "主题"),
                ("[P]", "切换主题"),
                ("[R]", "重置"),
                ("[E]", "编辑分隔符"),
                ("[S]", "保存配置"),
                ("[W]", "写入主题"),
                ("[Ctrl+S]", "保存主题"),
                ("[Esc]", "退出"),
            ]
        };

        let status = status_message.unwrap_or("");

        // Build help text with smart wrapping - keep each shortcut as a unit
        let content_width = area.width.saturating_sub(2); // Remove borders
        let mut lines = Vec::new();
        let mut current_line_spans = Vec::new();
        let mut current_width = 0usize;

        for (i, (key, description)) in help_items.iter().enumerate() {
            // Calculate item display width
            let item_width = key.chars().count() + description.chars().count() + 1; // +1 for space

            // Add separator for non-first items on the same line
            let needs_separator = i > 0 && !current_line_spans.is_empty();
            let separator_width = if needs_separator { 2 } else { 0 };
            let total_width = item_width + separator_width;

            // Check if item fits on current line
            if current_width + total_width <= content_width as usize {
                // Item fits, add to current line
                if needs_separator {
                    current_line_spans.push(Span::styled("  ", Style::default()));
                    current_width += 2;
                }

                // Add highlighted key and description
                current_line_spans.push(Span::styled(
                    *key,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
                current_line_spans.push(Span::styled(
                    format!(" {}", description),
                    Style::default().fg(Color::Gray),
                ));
                current_width += item_width;
            } else {
                // Item doesn't fit, start new line
                if !current_line_spans.is_empty() {
                    lines.push(Line::from(current_line_spans));
                    current_line_spans = Vec::new();
                }

                // Start new line with this item
                current_line_spans.push(Span::styled(
                    *key,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
                current_line_spans.push(Span::styled(
                    format!(" {}", description),
                    Style::default().fg(Color::Gray),
                ));
                current_width = item_width;
            }
        }

        // Add last line if not empty
        if !current_line_spans.is_empty() {
            lines.push(Line::from(current_line_spans));
        }

        // Add status message if present
        if !status.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                status,
                Style::default().fg(Color::Green),
            )));
        }

        let help_text = Text::from(lines);
        let help_paragraph = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("帮助"))
            .wrap(ratatui::widgets::Wrap { trim: false });
        f.render_widget(help_paragraph, area);
    }
}
