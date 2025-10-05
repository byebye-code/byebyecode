use crate::config::{Config, SegmentId};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    SegmentList,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldSelection {
    Enabled,
    Icon,
    IconColor,
    TextColor,
    BackgroundColor,
    TextStyle,
    Options,
}

#[derive(Default)]
pub struct SegmentListComponent;

impl SegmentListComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        config: &Config,
        selected_segment: usize,
        selected_panel: &Panel,
    ) {
        let items: Vec<ListItem> = config
            .segments
            .iter()
            .enumerate()
            .map(|(i, segment)| {
                let is_selected = i == selected_segment && *selected_panel == Panel::SegmentList;
                let enabled_marker = if segment.enabled { "●" } else { "○" };
                let segment_name = match segment.id {
                    SegmentId::Model => "模型",
                    SegmentId::Directory => "目录",
                    SegmentId::Git => "Git",
                    SegmentId::ContextWindow => "上下文窗口",
                    SegmentId::Usage => "用量",
                    SegmentId::Cost => "费用",
                    SegmentId::Session => "会话",
                    SegmentId::OutputStyle => "输出样式",
                    SegmentId::Update => "更新",
                    SegmentId::ByeByeCodeUsage => "88code 用量",
                    SegmentId::ByeByeCodeSubscription => "88code 订阅",
                    SegmentId::ByeByeCodeStatus => "88code 状态",
                };

                if is_selected {
                    // Selected item with colored cursor
                    ListItem::new(Line::from(vec![
                        Span::styled("▶ ", Style::default().fg(Color::Cyan)),
                        Span::raw(format!("{} {}", enabled_marker, segment_name)),
                    ]))
                } else {
                    // Non-selected item
                    ListItem::new(format!("  {} {}", enabled_marker, segment_name))
                }
            })
            .collect();
        let segments_block = Block::default()
            .borders(Borders::ALL)
            .title("状态段")
            .border_style(if *selected_panel == Panel::SegmentList {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });
        let segments_list = List::new(items).block(segments_block);
        f.render_widget(segments_list, area);
    }
}
