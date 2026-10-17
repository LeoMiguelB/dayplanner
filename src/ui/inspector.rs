use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render_inspector(f: &mut Frame, app: &App, area: Rect) {
    let block_widget = Block::default()
        .borders(Borders::ALL)
        .title(" Block Inspector [Tab: Hide] ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let inner = block_widget.inner(area);
    f.render_widget(block_widget, area);

    let selected_date = app.cursor_date();

    // Calculate daily statistics
    let today_blocks: Vec<_> = app
        .blocks
        .iter()
        .filter(|b| b.overlaps_day(selected_date))
        .collect();

    let mut total_planned_mins = 0;
    for b in &today_blocks {
        let s = b.start_minute_of_day(selected_date);
        let e = b.end_minute_of_day(selected_date);
        total_planned_mins += e.saturating_sub(s);
    }

    let planned_hours = total_planned_mins / 60;
    let planned_remainder_mins = total_planned_mins % 60;
    let free_mins = 1440_u32.saturating_sub(total_planned_mins);
    let free_hours = free_mins / 60;
    let free_remainder_mins = free_mins % 60;

    let mut lines = Vec::new();

    if let Some(block) = app.get_selected_block() {
        let category = app
            .categories
            .iter()
            .find(|c| c.id == block.category_id);
        let cat_name = category.map(|c| c.name.as_str()).unwrap_or("Task");
        let cat_color = category.map(|c| c.ratatui_color()).unwrap_or(Color::Cyan);

        lines.push(Line::from(vec![
            Span::styled("▣ ", Style::default().fg(cat_color)),
            Span::styled(
                &block.title,
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Category: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("[{}]", cat_name),
                Style::default().fg(cat_color).add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Date:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                block.start_time.format("%a, %b %d, %Y").to_string(),
                Style::default().fg(Color::White),
            ),
        ]));

        let dur_mins = block.duration_minutes();
        let dur_str = if dur_mins >= 60 {
            format!("{}h {}m", dur_mins / 60, dur_mins % 60)
        } else {
            format!("{}m", dur_mins)
        };

        lines.push(Line::from(vec![
            Span::styled("  Time:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{} – {} ({})",
                    block.start_time.format("%H:%M"),
                    block.end_time.format("%H:%M"),
                    dur_str
                ),
                Style::default().fg(Color::Yellow),
            ),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Notes:",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED),
        )]));

        if block.description.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "  (No description entered)",
                Style::default().fg(Color::DarkGray),
            )]));
        } else {
            for note_line in block.description.lines() {
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}", note_line),
                    Style::default().fg(Color::White),
                )]));
            }
        }
    } else {
        lines.push(Line::from(vec![Span::styled(
            "No Block Selected",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![
            Span::raw("Slot: "),
            Span::styled(
                format!(
                    "{:02}:{:02} ({})",
                    app.cursor_minute / 60,
                    app.cursor_minute % 60,
                    app.cursor_date().format("%b %d")
                ),
                Style::default().fg(Color::Yellow),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("Press "),
            Span::styled("i", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" to schedule a block here."),
        ]));
        lines.push(Line::from(vec![
            Span::raw("Press "),
            Span::styled("v", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(" to stretch a time range."),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "───────────────────────────────",
        Style::default().fg(Color::DarkGray),
    )]));

    // Daily Stats
    lines.push(Line::from(vec![
        Span::styled("📊 DAY SUMMARY (", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(selected_date.format("%b %d").to_string(), Style::default().fg(Color::White)),
        Span::styled(")", Style::default().fg(Color::Cyan)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Planned:  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}h {:02}m", planned_hours, planned_remainder_mins),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Free:     ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}h {:02}m", free_hours, free_remainder_mins),
            Style::default().fg(Color::LightBlue),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Events:   ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", today_blocks.len()),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "⚡ QUICK ACTIONS",
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![
        Span::styled("  i/c", Style::default().fg(Color::Cyan)),
        Span::raw(" Add / Edit  "),
        Span::styled("dd", Style::default().fg(Color::Red)),
        Span::raw(" Delete"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  J/K", Style::default().fg(Color::Yellow)),
        Span::raw(" Nudge ±15m  "),
        Span::styled(">/<", Style::default().fg(Color::Yellow)),
        Span::raw(" Resize"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  y/p", Style::default().fg(Color::Green)),
        Span::raw(" Yank & Paste"),
    ]));

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}
