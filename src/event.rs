use chrono::{Datelike, Duration, NaiveTime, Timelike};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration as StdDuration;

use crate::app::App;
use crate::model::{AppMode, FormField, Granularity, ViewMode};

pub struct EventHandler {
    last_key: Option<char>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self { last_key: None }
    }

    pub fn handle_events(&mut self, app: &mut App) -> Result<(), std::io::Error> {
        if event::poll(StdDuration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                self.handle_key(app, key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, app: &mut App, key: KeyEvent) {
        match app.mode {
            AppMode::Normal => self.handle_normal_mode(app, key),
            AppMode::Visual => self.handle_visual_mode(app, key),
            AppMode::Insert => self.handle_insert_mode(app, key),
            AppMode::Command => self.handle_command_mode(app, key),
            AppMode::FirstRunWizard => self.handle_wizard_mode(app, key),
            AppMode::GranularityPicker => self.handle_zoom_picker_mode(app, key),
            AppMode::Help => self.handle_help_mode(app, key),
        }
    }

    fn handle_normal_mode(&mut self, app: &mut App, key: KeyEvent) {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let is_shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            // Quit
            KeyCode::Char('q') if !is_ctrl => {
                app.should_quit = true;
            }
            KeyCode::Char('c') if is_ctrl => {
                app.should_quit = true;
            }

            // Navigation
            KeyCode::Char('h') | KeyCode::Left => {
                app.move_cursor_left();
                self.last_key = Some('h');
            }
            KeyCode::Char('l') | KeyCode::Right => {
                app.move_cursor_right();
                self.last_key = Some('l');
            }
            KeyCode::Char('j') if is_shift => {
                // Shift+J: Nudge block down 15m
                app.nudge_block(15);
                self.last_key = None;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.move_cursor_down();
                self.last_key = Some('j');
            }
            KeyCode::Char('k') if is_shift => {
                // Shift+K: Nudge block up 15m
                app.nudge_block(-15);
                self.last_key = None;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.move_cursor_up();
                self.last_key = Some('k');
            }

            // Paging / Half-day jumps
            KeyCode::Char('d') if is_ctrl => {
                app.jump_hours(4);
            }
            KeyCode::Char('u') if is_ctrl => {
                app.jump_hours(-4);
            }

            // gg / G (Top and Bottom of 24h Day)
            KeyCode::Char('g') => {
                if self.last_key == Some('g') {
                    app.jump_to_start_of_day();
                    self.last_key = None;
                } else {
                    self.last_key = Some('g');
                }
            }
            KeyCode::Char('G') => {
                app.jump_to_end_of_day();
                self.last_key = None;
            }

            // Snap to current time (Now)
            KeyCode::Char('t') => {
                app.jump_to_now();
                self.last_key = Some('t');
            }

            // Jump between blocks
            KeyCode::Char('w') => {
                app.jump_next_block();
                self.last_key = Some('w');
            }
            KeyCode::Char('b') => {
                app.jump_prev_block();
                self.last_key = Some('b');
            }

            // View modes
            KeyCode::Char('1') => {
                app.view_mode = ViewMode::Day;
                app.cursor_day_offset = 0;
                app.refresh_blocks();
            }
            KeyCode::Char('3') => {
                app.view_mode = ViewMode::ThreeDay;
                app.cursor_day_offset = 0;
                app.refresh_blocks();
            }
            KeyCode::Char('W') | KeyCode::Char('7') => {
                app.view_mode = ViewMode::Week;
                app.cursor_day_offset = app.current_date.weekday().num_days_from_monday() as usize;
                app.refresh_blocks();
            }

            // Zoom / Granularity
            KeyCode::Char('+') | KeyCode::Char('=') => {
                app.granularity = app.granularity.zoom_in();
                app.set_status(format!("Granularity: {}", app.granularity.label()));
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                app.granularity = app.granularity.zoom_out();
                app.set_status(format!("Granularity: {}", app.granularity.label()));
            }
            KeyCode::Char('z') => {
                app.mode = AppMode::GranularityPicker;
            }

            // Toggle Inspector
            KeyCode::Tab => {
                app.show_inspector = !app.show_inspector;
            }

            // Block creation & editing
            KeyCode::Char('i') | KeyCode::Char('a') | KeyCode::Char('o') => {
                app.open_create_modal();
            }
            KeyCode::Char('c') => {
                app.open_edit_modal();
            }
            KeyCode::Char('d') => {
                if self.last_key == Some('d') {
                    app.delete_selected_block();
                    self.last_key = None;
                } else {
                    self.last_key = Some('d');
                }
            }

            // Block resizing & copying
            KeyCode::Char('>') => {
                app.resize_block(15);
            }
            KeyCode::Char('<') => {
                app.resize_block(-15);
            }
            KeyCode::Char('y') => {
                app.yank_selected_block();
            }
            KeyCode::Char('p') => {
                app.paste_yanked_block();
            }

            // Modal Transitions
            KeyCode::Char('v') => {
                app.mode = AppMode::Visual;
                app.visual_start_minute = Some(app.cursor_minute);
            }
            KeyCode::Char(':') => {
                app.mode = AppMode::Command;
                app.command_buffer.clear();
            }
            KeyCode::Char('?') => {
                app.mode = AppMode::Help;
            }

            _ => {
                self.last_key = None;
            }
        }
    }

    fn handle_visual_mode(&mut self, app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.move_cursor_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.move_cursor_up();
            }
            KeyCode::Char('c') | KeyCode::Char('a') | KeyCode::Char('i') | KeyCode::Enter => {
                app.open_create_from_visual();
            }
            KeyCode::Char('d') => {
                app.delete_selected_block();
                app.mode = AppMode::Normal;
                app.visual_start_minute = None;
            }
            KeyCode::Esc => {
                app.mode = AppMode::Normal;
                app.visual_start_minute = None;
            }
            _ => {}
        }
    }

    fn handle_insert_mode(&mut self, app: &mut App, key: KeyEvent) {
        let is_shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Esc => {
                app.mode = AppMode::Normal;
            }
            KeyCode::Tab => {
                if is_shift {
                    app.form_field = app.form_field.prev();
                } else {
                    app.form_field = app.form_field.next();
                }
            }
            KeyCode::BackTab => {
                app.form_field = app.form_field.prev();
            }
            KeyCode::Enter => {
                app.save_form();
            }
            _ => {
                // Field-specific inputs
                match app.form_field {
                    FormField::Title => match key.code {
                        KeyCode::Char(c) => app.form_title.push(c),
                        KeyCode::Backspace => {
                            app.form_title.pop();
                        }
                        _ => {}
                    },
                    FormField::Category => match key.code {
                        KeyCode::Char('c') | KeyCode::Char(' ') | KeyCode::Right => {
                            if !app.categories.is_empty() {
                                app.form_category_idx =
                                    (app.form_category_idx + 1) % app.categories.len();
                            }
                        }
                        KeyCode::Left => {
                            if !app.categories.is_empty() {
                                if app.form_category_idx == 0 {
                                    app.form_category_idx = app.categories.len() - 1;
                                } else {
                                    app.form_category_idx -= 1;
                                }
                            }
                        }
                        _ => {}
                    },
                    FormField::Date => match key.code {
                        KeyCode::Char('+') | KeyCode::Up => {
                            app.form_date = app.form_date + Duration::days(1);
                        }
                        KeyCode::Char('-') | KeyCode::Down => {
                            app.form_date = app.form_date - Duration::days(1);
                        }
                        _ => {}
                    },
                    FormField::StartTime => match key.code {
                        KeyCode::Up | KeyCode::Char('+') => {
                            let m = (app.form_start_time.hour() * 60 + app.form_start_time.minute() + 15) % 1440;
                            app.form_start_time = NaiveTime::from_hms_opt(m / 60, m % 60, 0).unwrap();
                        }
                        KeyCode::Down | KeyCode::Char('-') => {
                            let curr = app.form_start_time.hour() * 60 + app.form_start_time.minute();
                            let m = if curr >= 15 { curr - 15 } else { 1440 - 15 };
                            app.form_start_time = NaiveTime::from_hms_opt(m / 60, m % 60, 0).unwrap();
                        }
                        _ => {}
                    },
                    FormField::EndTime => match key.code {
                        KeyCode::Up | KeyCode::Char('+') => {
                            let m = (app.form_end_time.hour() * 60 + app.form_end_time.minute() + 15) % 1440;
                            app.form_end_time = NaiveTime::from_hms_opt(m / 60, m % 60, 0).unwrap();
                        }
                        KeyCode::Down | KeyCode::Char('-') => {
                            let curr = app.form_end_time.hour() * 60 + app.form_end_time.minute();
                            let m = if curr >= 15 { curr - 15 } else { 1440 - 15 };
                            app.form_end_time = NaiveTime::from_hms_opt(m / 60, m % 60, 0).unwrap();
                        }
                        _ => {}
                    },
                    FormField::Description => match key.code {
                        KeyCode::Char(c) => app.form_description.push(c),
                        KeyCode::Backspace => {
                            app.form_description.pop();
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    fn handle_command_mode(&mut self, app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                app.mode = AppMode::Normal;
                app.command_buffer.clear();
            }
            KeyCode::Enter => {
                app.execute_command();
            }
            KeyCode::Backspace => {
                app.command_buffer.pop();
            }
            KeyCode::Char(c) => {
                app.command_buffer.push(c);
            }
            _ => {}
        }
    }

    fn handle_wizard_mode(&mut self, app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Char('1') => app.wizard_selected_option = 0,
            KeyCode::Char('2') => app.wizard_selected_option = 1,
            KeyCode::Char('3') => app.wizard_selected_option = 2,
            KeyCode::Char('4') => app.wizard_selected_option = 3,
            KeyCode::Char('j') | KeyCode::Down => {
                app.wizard_selected_option = (app.wizard_selected_option + 1) % 4;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.wizard_selected_option == 0 {
                    app.wizard_selected_option = 3;
                } else {
                    app.wizard_selected_option -= 1;
                }
            }
            KeyCode::Enter => {
                app.complete_wizard();
            }
            _ => {
                if app.wizard_selected_option == 2 {
                    match key.code {
                        KeyCode::Char(c) => app.wizard_custom_path.push(c),
                        KeyCode::Backspace => {
                            app.wizard_custom_path.pop();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_zoom_picker_mode(&mut self, app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Char('1') => {
                app.granularity = Granularity::FifteenMinutes;
                app.mode = AppMode::Normal;
            }
            KeyCode::Char('2') => {
                app.granularity = Granularity::ThirtyMinutes;
                app.mode = AppMode::Normal;
            }
            KeyCode::Char('3') => {
                app.granularity = Granularity::SixtyMinutes;
                app.mode = AppMode::Normal;
            }
            KeyCode::Esc | KeyCode::Enter => {
                app.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_help_mode(&mut self, app: &mut App, _key: KeyEvent) {
        app.mode = AppMode::Normal;
    }
}
