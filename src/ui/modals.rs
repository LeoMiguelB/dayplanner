use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::config::ConfigManager;
use crate::model::{FormField, Granularity};

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_form_modal(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(65, 75, area);
    f.render_widget(Clear, popup_area);

    let is_edit = app.form_block_id.is_some();
    let title = if is_edit {
        " ✏ Edit Time Block "
    } else {
        " ➕ New Time Block "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Category
            Constraint::Length(3), // Date, Start, End row
            Constraint::Min(4),    // Description
            Constraint::Length(2), // Action hints
        ])
        .split(inner);

    // 1. Title Input
    let title_focused = app.form_field == FormField::Title;
    let title_border_color = if title_focused { Color::Green } else { Color::DarkGray };
    let title_cursor = if title_focused { "█" } else { "" };
    let title_p = Paragraph::new(format!("{}{}", app.form_title, title_cursor))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Title ")
                .border_style(Style::default().fg(title_border_color)),
        );
    f.render_widget(title_p, chunks[0]);

    // 2. Category Selector
    let cat_focused = app.form_field == FormField::Category;
    let cat_border_color = if cat_focused { Color::Green } else { Color::DarkGray };
    let current_cat = app.categories.get(app.form_category_idx);
    let cat_name = current_cat.map(|c| c.name.as_str()).unwrap_or("General");
    let cat_color = current_cat.map(|c| c.ratatui_color()).unwrap_or(Color::Cyan);

    let cat_spans = vec![
        Span::styled(format!(" [■ {}] ", cat_name), Style::default().fg(cat_color).add_modifier(Modifier::BOLD)),
        Span::styled(" (Press 'c' or Space to cycle categories)", Style::default().fg(Color::DarkGray)),
    ];
    let cat_p = Paragraph::new(Line::from(cat_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Category ")
            .border_style(Style::default().fg(cat_border_color)),
    );
    f.render_widget(cat_p, chunks[1]);

    // 3. Date, Start, End Row
    let row_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(chunks[2]);

    let date_focused = app.form_field == FormField::Date;
    let date_color = if date_focused { Color::Green } else { Color::DarkGray };
    let date_p = Paragraph::new(app.form_date.format("%Y-%m-%d").to_string()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Date (YYYY-MM-DD) ")
            .border_style(Style::default().fg(date_color)),
    );
    f.render_widget(date_p, row_chunks[0]);

    let start_focused = app.form_field == FormField::StartTime;
    let start_color = if start_focused { Color::Green } else { Color::DarkGray };
    let start_p = Paragraph::new(app.form_start_time.format("%H:%M").to_string()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Start (HH:MM) ")
            .border_style(Style::default().fg(start_color)),
    );
    f.render_widget(start_p, row_chunks[1]);

    let end_focused = app.form_field == FormField::EndTime;
    let end_color = if end_focused { Color::Green } else { Color::DarkGray };
    let end_p = Paragraph::new(app.form_end_time.format("%H:%M").to_string()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" End (HH:MM) ")
            .border_style(Style::default().fg(end_color)),
    );
    f.render_widget(end_p, row_chunks[2]);

    // 4. Description / Notes
    let desc_focused = app.form_field == FormField::Description;
    let desc_color = if desc_focused { Color::Green } else { Color::DarkGray };
    let desc_cursor = if desc_focused { "█" } else { "" };
    let desc_p = Paragraph::new(format!("{}{}", app.form_description, desc_cursor))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Description / Notes ")
                .border_style(Style::default().fg(desc_color)),
        );
    f.render_widget(desc_p, chunks[3]);

    // 5. Action Hints
    let hints = Line::from(vec![
        Span::styled("<Enter> ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("Save Block  │  "),
        Span::styled("<Tab> / <Shift-Tab> ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("Next/Prev Field  │  "),
        Span::styled("<Esc> ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("Cancel"),
    ]);
    f.render_widget(Paragraph::new(hints).alignment(Alignment::Center), chunks[4]);
}

pub fn render_wizard_modal(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(70, 65, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Welcome to Dayplanner - First-Time Setup ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let vault_path = ConfigManager::vault_preset_path().to_string_lossy().to_string();
    let xdg_path = ConfigManager::xdg_data_default_path().to_string_lossy().to_string();
    let temp_path = ConfigManager::temp_fallback_path().to_string_lossy().to_string();

    let options = [
        ("1. Vault Misc Directory (Recommended)", vault_path.as_str()),
        ("2. Standard User Data Directory (XDG)", xdg_path.as_str()),
        ("3. Custom SQLite Database Path", &app.wizard_custom_path),
        ("4. Ephemeral / Temporary Session (No persistence)", temp_path.as_str()),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from("Dayplanner stores your schedule locally in an embedded SQLite database."));
    lines.push(Line::from("Please select where you would like your database file to live:"));
    lines.push(Line::from(""));

    for (i, (title, path)) in options.iter().enumerate() {
        let is_selected = i == app.wizard_selected_option;
        let radio = if is_selected { "(●) " } else { "( ) " };
        let opt_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(radio, opt_style),
            Span::styled(*title, opt_style),
        ]));

        if i == 2 && is_selected {
            // Show custom input box
            let cursor = "█";
            lines.push(Line::from(vec![
                Span::raw("    Path: [ "),
                Span::styled(format!("{}{}", path, cursor), Style::default().fg(Color::Green)),
                Span::raw(" ]"),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(format!("    {}", path), Style::default().fg(Color::DarkGray)),
            ]));
        }
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![
        Span::styled("[1-4 / j / k]: ", Style::default().fg(Color::Cyan)),
        Span::raw("Navigate  │  "),
        Span::styled("[Enter]: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("Confirm & Launch"),
    ]));

    f.render_widget(Paragraph::new(lines), inner);
}

pub fn render_zoom_picker(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(38, 30, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Time Granularity ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let options = [
        (Granularity::FifteenMinutes, "1. 15 Minutes (Precise)"),
        (Granularity::ThirtyMinutes, "2. 30 Minutes (Standard)"),
        (Granularity::SixtyMinutes, "3. 60 Minutes (Compact)"),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    for (g, label) in options {
        let is_active = app.granularity == g;
        let radio = if is_active { " (•) " } else { " ( ) " };
        let style = if is_active {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(vec![
            Span::styled(radio, style),
            Span::styled(label, style),
        ]));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![
        Span::styled("[1/2/3] ", Style::default().fg(Color::Cyan)),
        Span::raw("Select  │  "),
        Span::styled("[Esc] ", Style::default().fg(Color::Red)),
        Span::raw("Close"),
    ]));

    f.render_widget(Paragraph::new(lines), inner);
}

pub fn render_help_modal(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(75, 75, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Dayplanner Keybindings (Vim Motions) ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let help_text = vec![
        Line::from(vec![
            Span::styled("NAVIGATION MOTIONS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("                     "),
            Span::styled("BLOCK EDITING & ACTIONS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  h / l        ", Style::default().fg(Color::Yellow)),
            Span::raw("Move Day Left/Right    "),
            Span::styled("  i / a        ", Style::default().fg(Color::Yellow)),
            Span::raw("Insert new block at cursor"),
        ]),
        Line::from(vec![
            Span::styled("  j / k        ", Style::default().fg(Color::Yellow)),
            Span::raw("Move Time Up/Down      "),
            Span::styled("  c / cw       ", Style::default().fg(Color::Yellow)),
            Span::raw("Edit selected block"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-d / u   ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump 4 Hours Down/Up   "),
            Span::styled("  dd           ", Style::default().fg(Color::Yellow)),
            Span::raw("Delete selected block"),
        ]),
        Line::from(vec![
            Span::styled("  gg / G       ", Style::default().fg(Color::Yellow)),
            Span::raw("Top / Bottom of 24h Day"),
            Span::styled("  J / K        ", Style::default().fg(Color::Yellow)),
            Span::raw("Nudge block time ±15 mins"),
        ]),
        Line::from(vec![
            Span::styled("  t            ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump to Current Time   "),
            Span::styled("  > / <        ", Style::default().fg(Color::Yellow)),
            Span::raw("Resize duration ±15 mins"),
        ]),
        Line::from(vec![
            Span::styled("  w / b        ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump Next / Prev Block "),
            Span::styled("  y / p        ", Style::default().fg(Color::Yellow)),
            Span::raw("Yank (copy) & Paste block"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("VIEW & GRANULARITY", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("                     "),
            Span::styled("COMMAND MODE (:)", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  1 / 3 / w    ", Style::default().fg(Color::Yellow)),
            Span::raw("Day / 3-Day / Week View"),
            Span::styled("  :w           ", Style::default().fg(Color::Yellow)),
            Span::raw("Sync database"),
        ]),
        Line::from(vec![
            Span::styled("  + / -        ", Style::default().fg(Color::Yellow)),
            Span::raw("Zoom In / Out (15/30/60)"),
            Span::styled("  :q / :q!     ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit Dayplanner"),
        ]),
        Line::from(vec![
            Span::styled("  z            ", Style::default().fg(Color::Yellow)),
            Span::raw("Granularity Picker Menu"),
            Span::styled("  :db <path>   ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch database path"),
        ]),
        Line::from(vec![
            Span::styled("  Tab          ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle Inspector Pane  "),
            Span::styled("  :goto <date> ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump to YYYY-MM-DD or today"),
        ]),
        Line::from(vec![
            Span::styled("  v            ", Style::default().fg(Color::Yellow)),
            Span::raw("Visual range select    "),
            Span::styled("  ?            ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this Help overlay"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press <Esc> or <Enter> to close help.", Style::default().fg(Color::Green)),
        ]),
    ];

    f.render_widget(Paragraph::new(help_text), inner);
}
