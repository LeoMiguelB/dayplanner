use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::model::ViewMode;

pub fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // App Title & Help
            Constraint::Min(25),    // Active Date Range
            Constraint::Length(26), // View mode tabs
            Constraint::Length(26), // Zoom / Granularity
        ])
        .split(area);

    // 1. App Title
    let title_line = Line::from(vec![
        Span::styled(" 📅 Dayplanner ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("[? Help]", Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(title_line), chunks[0]);

    // 2. Active Date Range
    let start_date = app.view_start_date();
    let end_date = app.view_end_date();
    let date_str = if start_date == end_date {
        start_date.format("%A, %B %d, %Y").to_string()
    } else {
        format!(
            "Week {} ({} – {})",
            start_date.format("%U"),
            start_date.format("%b %d"),
            end_date.format("%b %d, %Y")
        )
    };
    let date_line = Line::from(vec![
        Span::styled("◄ ", Style::default().fg(Color::Yellow)),
        Span::styled(date_str, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" ►", Style::default().fg(Color::Yellow)),
    ]);
    f.render_widget(Paragraph::new(date_line).alignment(Alignment::Center), chunks[1]);

    // 3. View Mode Switcher
    let day_style = if app.view_mode == ViewMode::Day {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let three_style = if app.view_mode == ViewMode::ThreeDay {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let week_style = if app.view_mode == ViewMode::Week {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let view_line = Line::from(vec![
        Span::raw("View: "),
        Span::styled("[1]Day", day_style),
        Span::raw(" "),
        Span::styled("[3]3-Day", three_style),
        Span::raw(" "),
        Span::styled("[w]Week", week_style),
    ]);
    f.render_widget(Paragraph::new(view_line), chunks[2]);

    // 4. Granularity / Zoom Control
    let zoom_label = app.granularity.label();
    let zoom_line = Line::from(vec![
        Span::raw("Zoom: "),
        Span::styled("[-]", Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(
            format!("[{}]", zoom_label),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled("[+]", Style::default().fg(Color::Yellow)),
        Span::styled(" (z)", Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(zoom_line).alignment(Alignment::Right), chunks[3]);
}
