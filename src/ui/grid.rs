use chrono::{Duration, Local, Timelike};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::model::AppMode;

pub fn render_grid(f: &mut Frame, app: &App, area: Rect) {
    if area.height < 4 || area.width < 20 {
        return;
    }

    let today = Local::now().date_naive();
    let current_live_minute = Local::now().hour() * 60 + Local::now().minute();

    let step_mins = app.granularity.minutes();
    let header_height = 2;
    let grid_height = area.height.saturating_sub(header_height) as usize;
    let total_visible_minutes = (grid_height as u32) * step_mins;

    // Time column width: 7 chars (" 08:30 ")
    let time_col_width = 8;
    let days_count = app.view_mode.days_count();
    let day_start = app.view_start_date();

    // Top Header: Days of the week
    let header_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: header_height,
    };

    let body_area = Rect {
        x: area.x,
        y: area.y + header_height,
        width: area.width,
        height: area.height.saturating_sub(header_height),
    };

    // Split horizontally: Time Col + Day Cols
    let mut col_constraints = vec![Constraint::Length(time_col_width)];
    for _ in 0..days_count {
        col_constraints.push(Constraint::Ratio(1, days_count as u32));
    }

    let header_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(&col_constraints)
        .split(header_area);

    let body_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(&col_constraints)
        .split(body_area);

    // Render Time header
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            " Time",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
        )]))
        .block(Block::default().borders(Borders::BOTTOM)),
        header_cols[0],
    );

    // Render Day headers
    for (i, col_rect) in header_cols.iter().skip(1).enumerate() {
        let date = day_start + Duration::days(i as i64);
        let is_today = date == today;
        let is_cursor_col = i == app.cursor_day_offset;

        let day_label = date.format("%a %m/%d").to_string();
        let mut spans = Vec::new();

        if is_today {
            spans.push(Span::styled("● ", Style::default().fg(Color::Green)));
        }

        let header_style = if is_cursor_col {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else if is_today {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        spans.push(Span::styled(day_label, header_style));

        f.render_widget(
            Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::BOTTOM)),
            *col_rect,
        );
    }

    // Render Time Column Ruler
    let scroll_min = app.scroll_offset_minutes;
    let mut time_lines = Vec::new();
    for row in 0..grid_height {
        let slot_min = scroll_min + (row as u32 * step_mins);
        if slot_min >= 1440 {
            time_lines.push(Line::from(""));
            continue;
        }

        let h = slot_min / 60;
        let m = slot_min % 60;

        let line = if m == 0 {
            Line::from(vec![Span::styled(
                format!(" {:02}:{:02} ", h, m),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )])
        } else if m == 30 {
            Line::from(vec![Span::styled(
                format!(" {:02}:{:02} ", h, m),
                Style::default().fg(Color::DarkGray),
            )])
        } else {
            Line::from(vec![Span::styled(
                format!("    :{:02} ", m),
                Style::default().fg(Color::DarkGray),
            )])
        };
        time_lines.push(line);
    }

    f.render_widget(
        Paragraph::new(time_lines).block(Block::default().borders(Borders::RIGHT)),
        body_cols[0],
    );

    // Render Day Columns with Time Blocks and Cursor
    for (day_idx, col_rect) in body_cols.iter().skip(1).enumerate() {
        let date = day_start + Duration::days(day_idx as i64);
        let is_cursor_day = day_idx == app.cursor_day_offset;

        let mut lines = Vec::new();

        for row in 0..grid_height {
            let slot_start = scroll_min + (row as u32 * step_mins);
            let slot_end = slot_start + step_mins;

            if slot_start >= 1440 {
                lines.push(Line::from(""));
                continue;
            }

            let is_cursor_slot = is_cursor_day
                && app.cursor_minute >= slot_start
                && app.cursor_minute < slot_end;

            // Visual mode selection check
            let is_visual_selected = if is_cursor_day && app.mode == AppMode::Visual {
                if let Some(v_start) = app.visual_start_minute {
                    let min_sel = v_start.min(app.cursor_minute);
                    let max_sel = v_start.max(app.cursor_minute) + step_mins;
                    slot_start >= min_sel && slot_start < max_sel
                } else {
                    false
                }
            } else {
                false
            };

            // Overlapping block check
            let matching_blocks: Vec<_> = app
                .blocks
                .iter()
                .filter(|b| {
                    if b.overlaps_day(date) {
                        let b_start = b.start_minute_of_day(date);
                        let b_end = b.end_minute_of_day(date);
                        b_start < slot_end && b_end > slot_start
                    } else {
                        false
                    }
                })
                .collect();

            // Determine rendering spans
            let mut spans = Vec::new();

            if !matching_blocks.is_empty() {
                // Render block representation
                let block = matching_blocks[0];
                let b_start = block.start_minute_of_day(date);
                let b_end = block.end_minute_of_day(date);
                let is_block_top = slot_start <= b_start && b_start < slot_end;
                let is_block_bottom = slot_end >= b_end;

                let category = app
                    .categories
                    .iter()
                    .find(|c| c.id == block.category_id);
                let block_color = category
                    .map(|c| c.ratatui_color())
                    .unwrap_or(Color::Cyan);

                let is_selected_block = app.selected_block_id.as_deref() == Some(&block.id);

                let block_style = if is_selected_block {
                    Style::default()
                        .fg(Color::Black)
                        .bg(block_color)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(block_color)
                };

                let col_width = col_rect.width as usize;
                let inner_width = col_width.saturating_sub(4);

                let content = if is_block_top {
                    let title = if block.title.len() > inner_width {
                        &block.title[..inner_width.min(block.title.len())]
                    } else {
                        &block.title
                    };
                    format!("┌─ {:width$} ─┐", title, width = inner_width)
                } else if is_block_bottom {
                    format!("└{:─<width$}┘", "", width = col_width.saturating_sub(2))
                } else {
                    format!("│ {:width$} │", "", width = inner_width)
                };

                spans.push(Span::styled(content, block_style));
            } else if is_cursor_slot {
                let col_width = col_rect.width as usize;
                let marker = format!("▒{:░<width$}▒", "", width = col_width.saturating_sub(2));
                spans.push(Span::styled(
                    marker,
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ));
            } else if is_visual_selected {
                let col_width = col_rect.width as usize;
                let marker = format!("▓{:▓<width$}▓", "", width = col_width.saturating_sub(2));
                spans.push(Span::styled(marker, Style::default().fg(Color::Magenta)));
            } else if date == today
                && current_live_minute >= slot_start
                && current_live_minute < slot_end
            {
                // Live time marker line
                let col_width = col_rect.width as usize;
                let line_str = format!("─{:─<width$}─", " Now ", width = col_width.saturating_sub(7));
                spans.push(Span::styled(line_str, Style::default().fg(Color::Red)));
            } else {
                // Empty time slot with subtle dot
                let col_width = col_rect.width as usize;
                let slot_str = if slot_start % 60 == 0 {
                    format!("·{: <width$}", "", width = col_width.saturating_sub(1))
                } else {
                    format!("{: <width$}", "", width = col_width)
                };
                spans.push(Span::styled(slot_str, Style::default().fg(Color::DarkGray)));
            }

            lines.push(Line::from(spans));
        }

        f.render_widget(
            Paragraph::new(lines).block(Block::default().borders(Borders::RIGHT)),
            *col_rect,
        );
    }

    // Scroll indicators at right margin
    if scroll_min > 0 {
        let top_indicator = Rect {
            x: area.x + area.width.saturating_sub(2),
            y: area.y + header_height,
            width: 1,
            height: 1,
        };
        f.render_widget(
            Paragraph::new("▲").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            top_indicator,
        );
    }

    if scroll_min + total_visible_minutes < 1440 {
        let bot_indicator = Rect {
            x: area.x + area.width.saturating_sub(2),
            y: area.y + area.height.saturating_sub(1),
            width: 1,
            height: 1,
        };
        f.render_widget(
            Paragraph::new("▼").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            bot_indicator,
        );
    }
}
