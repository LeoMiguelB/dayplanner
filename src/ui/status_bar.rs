use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::model::AppMode;

pub fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // Mode Badge
            Constraint::Min(30),    // Selection / Status Message / Command
            Constraint::Length(45), // Active DB Path & Position
        ])
        .split(area);

    // 1. Mode Badge
    let (mode_str, mode_style) = match app.mode {
        AppMode::Normal => (" NORMAL ", Style::default().fg(Color::Black).bg(Color::Blue).add_modifier(Modifier::BOLD)),
        AppMode::Visual => (" VISUAL ", Style::default().fg(Color::White).bg(Color::Magenta).add_modifier(Modifier::BOLD)),
        AppMode::Insert => (" INSERT ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
        AppMode::Command => (" COMMAND ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
        AppMode::FirstRunWizard => (" SETUP ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        AppMode::GranularityPicker => (" ZOOM ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
        AppMode::Help => (" HELP ", Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)),
    };

    f.render_widget(Paragraph::new(Span::styled(mode_str, mode_style)), chunks[0]);

    // 2. Middle Message or Command Line
    if app.mode == AppMode::Command {
        let cmd_line = Line::from(vec![
            Span::styled(":", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&app.command_buffer, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]);
        f.render_widget(Paragraph::new(cmd_line), chunks[1]);
    } else if let Some((msg, instant)) = &app.status_message {
        if instant.elapsed().as_secs() < 4 {
            let status_line = Line::from(vec![
                Span::styled("ℹ ", Style::default().fg(Color::Cyan)),
                Span::styled(msg, Style::default().fg(Color::White)),
            ]);
            f.render_widget(Paragraph::new(status_line), chunks[1]);
        } else {
            render_default_selection_info(f, app, chunks[1]);
        }
    } else {
        render_default_selection_info(f, app, chunks[1]);
    }

    // 3. Database Path & Position
    let db_path_str = if let Some(db) = &app.db {
        let p = &db.path_display;
        if p.len() > 28 {
            format!("...{}", &p[p.len() - 25..])
        } else {
            p.clone()
        }
    } else {
        "No DB".to_string()
    };

    let pos_line = Line::from(vec![
        Span::styled("DB: ", Style::default().fg(Color::DarkGray)),
        Span::styled(db_path_str, Style::default().fg(Color::White)),
        Span::raw(" | "),
        Span::styled(
            format!("{:02}:{:02}", app.cursor_minute / 60, app.cursor_minute % 60),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
    ]);

    f.render_widget(
        Paragraph::new(pos_line).alignment(ratatui::layout::Alignment::Right),
        chunks[2],
    );
}

fn render_default_selection_info(f: &mut Frame, app: &App, area: Rect) {
    if let Some(block) = app.get_selected_block() {
        let category = app
            .categories
            .iter()
            .find(|c| c.id == block.category_id);
        let cat_color = category.map(|c| c.ratatui_color()).unwrap_or(Color::Cyan);

        let info = Line::from(vec![
            Span::styled(
                format!("{} - {} ", block.start_time.format("%H:%M"), block.end_time.format("%H:%M")),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(format!("\"{}\" ", block.title), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("[{}]", category.map(|c| c.name.as_str()).unwrap_or("Task")),
                Style::default().fg(cat_color),
            ),
        ]);
        f.render_widget(Paragraph::new(info), area);
    } else {
        let info = Line::from(vec![
            Span::styled(
                format!("Slot: {:02}:{:02} ", app.cursor_minute / 60, app.cursor_minute % 60),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("({})", app.cursor_date().format("%a %b %d")),
                Style::default().fg(Color::White),
            ),
            Span::raw(" | Press 'i' to add event, 'v' to select range"),
        ]);
        f.render_widget(Paragraph::new(info), area);
    }
}
