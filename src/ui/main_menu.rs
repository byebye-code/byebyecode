use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

#[derive(Default)]
pub struct MainMenu {
    selected_item: usize,
    should_quit: bool,
    show_about: bool,
}

#[derive(Debug)]
pub enum MenuResult {
    LaunchConfigurator,
    InitConfig,
    CheckConfig,
    Exit,
}

impl MainMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run() -> Result<Option<MenuResult>, Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = MainMenu::new();
        let result = app.main_loop(&mut terminal)?;

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(result)
    }

    fn main_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<Option<MenuResult>, Box<dyn std::error::Error>> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if self.show_about {
                    // In about dialog, any key closes it
                    self.show_about = false;
                    continue;
                }

                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.should_quit = true;
                    }
                    KeyCode::Up => {
                        if self.selected_item > 0 {
                            self.selected_item -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let menu_items = self.get_menu_items();
                        if self.selected_item < menu_items.len() - 1 {
                            self.selected_item += 1;
                        }
                    }
                    KeyCode::Enter => {
                        return Ok(Some(self.handle_selection()?));
                    }
                    _ => {}
                }
            }

            if self.should_quit {
                return Ok(Some(MenuResult::Exit));
            }
        }
    }

    fn get_menu_items(&self) -> Vec<(&str, &str)> {
        vec![
            (" 配置模式", "进入 TUI 配置界面"),
            (" 初始化配置", "创建默认配置文件"),
            (" 检查配置", "验证配置文件"),
            (" 关于", "显示应用程序信息"),
            (" 退出", "退出 CCometixLine"),
        ]
    }

    fn handle_selection(&mut self) -> Result<MenuResult, Box<dyn std::error::Error>> {
        match self.selected_item {
            0 => Ok(MenuResult::LaunchConfigurator),
            1 => Ok(MenuResult::InitConfig),
            2 => Ok(MenuResult::CheckConfig),
            3 => {
                self.show_about = true;
                // Return to loop to show about dialog
                self.main_loop_once()
            }
            4 => Ok(MenuResult::Exit),
            _ => Ok(MenuResult::Exit),
        }
    }

    fn main_loop_once(&mut self) -> Result<MenuResult, Box<dyn std::error::Error>> {
        // This is a placeholder - in the actual flow, we'd continue the main loop
        // but for now, let's just show about and continue
        Ok(MenuResult::Exit) // This won't actually be used
    }

    fn ui(&mut self, f: &mut Frame) {
        let size = f.area();

        // Main layout
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Header
                Constraint::Min(10),   // Menu
                Constraint::Length(3), // Footer
            ])
            .split(size);

        // Header
        let header_text = Text::from(vec![
            Line::from(vec![
                Span::styled(
                    "CCometixLine",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" v", Style::default().fg(Color::Gray)),
                Span::styled(
                    env!("CARGO_PKG_VERSION"),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "High-performance Claude Code StatusLine Configuration",
                Style::default().fg(Color::Gray),
            )),
        ]);

        let header = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL).title("Welcome"))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(header, main_layout[0]);

        // Menu
        let menu_items = self.get_menu_items();
        let list_items: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, (title, desc))| {
                let style = if i == self.selected_item {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };

                let content = Line::from(vec![
                    Span::styled(*title, style),
                    Span::styled(format!(" - {}", desc), Style::default().fg(Color::Gray)),
                ]);

                ListItem::new(content).style(style)
            })
            .collect();

        let menu_list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Main Menu")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
            .highlight_symbol("▶ ");

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_item));

        f.render_stateful_widget(menu_list, main_layout[1], &mut list_state);

        // Footer
        let footer_text = Text::from(vec![Line::from(vec![
            Span::styled(
                "[↑↓]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Navigate  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "[Enter]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Select  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "[Esc/Q]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Exit", Style::default().fg(Color::Gray)),
        ])]);

        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(footer, main_layout[2]);

        // About dialog overlay
        if self.show_about {
            self.render_about_dialog(f, size);
        }
    }

    fn render_about_dialog(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        // Calculate popup area (centered)
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(area)[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(popup_area)[1];

        // Clear the background
        f.render_widget(Clear, popup_area);

        let about_text = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "CCometixLine ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("v", Style::default().fg(Color::Gray)),
                Span::styled(
                    env!("CARGO_PKG_VERSION"),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Features:",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("• 🎨 TUI Configuration Interface"),
            Line::from("• 🎯 Multiple Built-in Themes"),
            Line::from("• ⚡ Real-time Usage Tracking"),
            Line::from("• 💰 Cost Monitoring"),
            Line::from("• 📊 Session Statistics"),
            Line::from("• 🎨 Nerd Font Support"),
            Line::from("• 🔧 Highly Customizable"),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key to continue...",
                Style::default().fg(Color::Yellow),
            )),
        ]);

        let about_dialog = Paragraph::new(about_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("About CCometixLine")
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(about_dialog, popup_area);
    }
}
