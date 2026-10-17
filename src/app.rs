use std::path::{Path, PathBuf};
use std::time::Instant;
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveTime, Timelike};
use uuid::Uuid;

use crate::config::{AppConfig, ConfigManager};
use crate::db::Database;
use crate::model::{AppMode, Category, FormField, Granularity, TimeBlock, ViewMode};

pub struct App {
    pub db: Option<Database>,
    pub config: AppConfig,
    pub mode: AppMode,
    pub view_mode: ViewMode,
    pub granularity: Granularity,

    // Time & Date Navigation
    pub current_date: NaiveDate,
    pub cursor_day_offset: usize, // 0..days_count
    pub cursor_minute: u32,       // 0..1439 (minute of the day)
    pub scroll_offset_minutes: u32, // Top of the visible 24h viewport (in minutes)

    // Visual Mode selection
    pub visual_start_minute: Option<u32>,

    // Side panel toggle
    pub show_inspector: bool,

    // Loaded Data
    pub categories: Vec<Category>,
    pub blocks: Vec<TimeBlock>,
    pub selected_block_id: Option<String>,
    pub yanked_block: Option<TimeBlock>,

    // Form Modal State (Insert / Edit)
    pub form_block_id: Option<String>,
    pub form_title: String,
    pub form_category_idx: usize,
    pub form_date: NaiveDate,
    pub form_start_time: NaiveTime,
    pub form_end_time: NaiveTime,
    pub form_description: String,
    pub form_field: FormField,

    // First-Run Wizard State
    pub wizard_selected_option: usize,
    pub wizard_custom_path: String,

    // Command Mode State
    pub command_buffer: String,

    // Status / Alert Notification
    pub status_message: Option<(String, Instant)>,
    pub should_quit: bool,
}

impl App {
    pub fn new(cli_db: Option<PathBuf>) -> Self {
        let (config, config_exists) = ConfigManager::load_config();
        let now = Local::now();
        let today = now.date_naive();
        let current_minute = (now.hour() * 60 + now.minute()) / 15 * 15; // round to 15m
        let scroll_start = current_minute.saturating_sub(120); // scroll so current hour is visible

        let mut app = Self {
            db: None,
            config: config.clone(),
            mode: AppMode::Normal,
            view_mode: ViewMode::Week,
            granularity: config.default_granularity.unwrap_or(Granularity::FifteenMinutes),
            current_date: today,
            cursor_day_offset: today.weekday().num_days_from_monday() as usize,
            cursor_minute: current_minute,
            scroll_offset_minutes: scroll_start,
            visual_start_minute: None,
            show_inspector: true,
            categories: Vec::new(),
            blocks: Vec::new(),
            selected_block_id: None,
            yanked_block: None,

            form_block_id: None,
            form_title: String::new(),
            form_category_idx: 0,
            form_date: today,
            form_start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            form_end_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            form_description: String::new(),
            form_field: FormField::Title,

            wizard_selected_option: 0,
            wizard_custom_path: String::new(),

            command_buffer: String::new(),
            status_message: None,
            should_quit: false,
        };

        // Determine DB path
        let resolved_path = if let Some(path) = cli_db {
            Some(path)
        } else if let Ok(env_path) = std::env::var("DAYPLANNER_DB") {
            Some(PathBuf::from(env_path))
        } else if let Some(cfg_path) = &config.db_path {
            Some(PathBuf::from(cfg_path))
        } else {
            None
        };

        if let Some(path) = resolved_path {
            app.connect_db(&path);
        } else if !config_exists {
            // First run: show onboarding wizard
            app.mode = AppMode::FirstRunWizard;
        } else {
            // Fallback
            let fallback = ConfigManager::temp_fallback_path();
            app.connect_db(&fallback);
        }

        app
    }

    pub fn connect_db(&mut self, path: &Path) {
        match Database::new(path) {
            Ok(database) => {
                let categories = database.get_categories().unwrap_or_default();
                self.categories = categories;
                self.db = Some(database);
                self.refresh_blocks();
                self.set_status(format!("Connected to database: {}", path.display()));
            }
            Err(e) => {
                self.set_status(format!("Error opening DB: {}", e));
            }
        }
    }

    pub fn set_status<S: Into<String>>(&mut self, msg: S) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn view_start_date(&self) -> NaiveDate {
        match self.view_mode {
            ViewMode::Day => self.current_date,
            ViewMode::ThreeDay => self.current_date,
            ViewMode::Week => {
                let days_from_monday = self.current_date.weekday().num_days_from_monday() as i64;
                self.current_date - Duration::days(days_from_monday)
            }
        }
    }

    pub fn view_end_date(&self) -> NaiveDate {
        let count = self.view_mode.days_count() as i64;
        self.view_start_date() + Duration::days(count - 1)
    }

    pub fn cursor_date(&self) -> NaiveDate {
        self.view_start_date() + Duration::days(self.cursor_day_offset as i64)
    }

    pub fn refresh_blocks(&mut self) {
        if let Some(db) = &self.db {
            let start = self
                .view_start_date()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap();
            let end = self
                .view_end_date()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap();

            if let Ok(blocks) = db.get_blocks_for_range(start, end) {
                self.blocks = blocks;
                self.update_selected_block();
            }
        }
    }

    pub fn update_selected_block(&mut self) {
        let c_date = self.cursor_date();
        let c_min = self.cursor_minute;

        // Check if cursor lands inside any block
        let found = self.blocks.iter().find(|b| {
            if b.overlaps_day(c_date) {
                let start_min = b.start_minute_of_day(c_date);
                let end_min = b.end_minute_of_day(c_date);
                start_min <= c_min && c_min < end_min
            } else {
                false
            }
        });

        self.selected_block_id = found.map(|b| b.id.clone());
    }

    pub fn get_selected_block(&self) -> Option<&TimeBlock> {
        if let Some(id) = &self.selected_block_id {
            self.blocks.iter().find(|b| &b.id == id)
        } else {
            None
        }
    }

    // Navigation Motions
    pub fn move_cursor_left(&mut self) {
        if self.cursor_day_offset > 0 {
            self.cursor_day_offset -= 1;
        } else {
            // Move week/view backward
            let count = self.view_mode.days_count() as i64;
            self.current_date = self.current_date - Duration::days(count);
            self.cursor_day_offset = self.view_mode.days_count() - 1;
            self.refresh_blocks();
        }
        self.update_selected_block();
    }

    pub fn move_cursor_right(&mut self) {
        let max_days = self.view_mode.days_count();
        if self.cursor_day_offset + 1 < max_days {
            self.cursor_day_offset += 1;
        } else {
            // Move week/view forward
            let count = self.view_mode.days_count() as i64;
            self.current_date = self.current_date + Duration::days(count);
            self.cursor_day_offset = 0;
            self.refresh_blocks();
        }
        self.update_selected_block();
    }

    pub fn move_cursor_up(&mut self) {
        let step = self.granularity.minutes();
        if self.cursor_minute >= step {
            self.cursor_minute -= step;
        } else {
            self.cursor_minute = 0;
        }
        self.keep_cursor_in_view();
        self.update_selected_block();
    }

    pub fn move_cursor_down(&mut self) {
        let step = self.granularity.minutes();
        if self.cursor_minute + step < 1440 {
            self.cursor_minute += step;
        } else {
            self.cursor_minute = 1440 - step;
        }
        self.keep_cursor_in_view();
        self.update_selected_block();
    }

    pub fn jump_hours(&mut self, hours: i32) {
        let minutes = hours * 60;
        if minutes < 0 {
            let abs_m = (-minutes) as u32;
            self.cursor_minute = self.cursor_minute.saturating_sub(abs_m);
        } else {
            let m = minutes as u32;
            self.cursor_minute = (self.cursor_minute + m).min(1440 - self.granularity.minutes());
        }
        self.keep_cursor_in_view();
        self.update_selected_block();
    }

    pub fn jump_to_start_of_day(&mut self) {
        self.cursor_minute = 0;
        self.scroll_offset_minutes = 0;
        self.update_selected_block();
    }

    pub fn jump_to_end_of_day(&mut self) {
        self.cursor_minute = 1440 - self.granularity.minutes();
        self.scroll_offset_minutes = 1440_u32.saturating_sub(480);
        self.update_selected_block();
    }

    pub fn jump_to_now(&mut self) {
        let now = Local::now();
        self.current_date = now.date_naive();
        let step = self.granularity.minutes();
        self.cursor_minute = (now.hour() * 60 + now.minute()) / step * step;
        self.cursor_day_offset = match self.view_mode {
            ViewMode::Day => 0,
            ViewMode::ThreeDay => 0,
            ViewMode::Week => self.current_date.weekday().num_days_from_monday() as usize,
        };
        self.scroll_offset_minutes = self.cursor_minute.saturating_sub(120);
        self.refresh_blocks();
    }

    pub fn keep_cursor_in_view(&mut self) {
        // Assume default visible viewport height is approx 10 hours (600 mins)
        let visible_window = 600;
        if self.cursor_minute < self.scroll_offset_minutes {
            self.scroll_offset_minutes = self.cursor_minute;
        } else if self.cursor_minute >= self.scroll_offset_minutes + visible_window {
            self.scroll_offset_minutes = self.cursor_minute.saturating_sub(visible_window - 120);
        }
    }

    pub fn jump_next_block(&mut self) {
        let c_date = self.cursor_date();
        let c_min = self.cursor_minute;

        // Find next block on current day or subsequent days
        let next = self.blocks.iter().find(|b| {
            if b.overlaps_day(c_date) {
                b.start_minute_of_day(c_date) > c_min
            } else {
                b.start_time.date_naive() > c_date
            }
        });

        if let Some(b) = next {
            let b_date = b.start_time.date_naive();
            let view_start = self.view_start_date();
            let days_diff = (b_date - view_start).num_days();
            if days_diff >= 0 && (days_diff as usize) < self.view_mode.days_count() {
                self.cursor_day_offset = days_diff as usize;
            }
            self.cursor_minute = b.start_minute_of_day(b_date);
            self.keep_cursor_in_view();
            self.update_selected_block();
        }
    }

    pub fn jump_prev_block(&mut self) {
        let c_date = self.cursor_date();
        let c_min = self.cursor_minute;

        let prev = self.blocks.iter().rev().find(|b| {
            if b.overlaps_day(c_date) {
                b.start_minute_of_day(c_date) < c_min
            } else {
                b.start_time.date_naive() < c_date
            }
        });

        if let Some(b) = prev {
            let b_date = b.start_time.date_naive();
            let view_start = self.view_start_date();
            let days_diff = (b_date - view_start).num_days();
            if days_diff >= 0 && (days_diff as usize) < self.view_mode.days_count() {
                self.cursor_day_offset = days_diff as usize;
            }
            self.cursor_minute = b.start_minute_of_day(b_date);
            self.keep_cursor_in_view();
            self.update_selected_block();
        }
    }

    // Block Form Preparation
    pub fn open_create_modal(&mut self) {
        let date = self.cursor_date();
        let start_min = self.cursor_minute;
        let end_min = (start_min + 60).min(1439);

        let start_h = start_min / 60;
        let start_m = start_min % 60;
        let end_h = end_min / 60;
        let end_m = end_min % 60;

        self.form_block_id = None;
        self.form_title = String::new();
        self.form_category_idx = 0;
        self.form_date = date;
        self.form_start_time = NaiveTime::from_hms_opt(start_h, start_m, 0).unwrap();
        self.form_end_time = NaiveTime::from_hms_opt(end_h, end_m, 0).unwrap();
        self.form_description = String::new();
        self.form_field = FormField::Title;
        self.mode = AppMode::Insert;
    }

    pub fn open_edit_modal(&mut self) {
        if let Some(block) = self.get_selected_block().cloned() {
            self.form_block_id = Some(block.id);
            self.form_title = block.title;
            self.form_category_idx = self
                .categories
                .iter()
                .position(|c| c.id == block.category_id)
                .unwrap_or(0);
            self.form_date = block.start_time.date_naive();
            self.form_start_time = block.start_time.time();
            self.form_end_time = block.end_time.time();
            self.form_description = block.description;
            self.form_field = FormField::Title;
            self.mode = AppMode::Insert;
        } else {
            self.set_status("No block selected to edit. Press 'i' to create one.");
        }
    }

    pub fn open_create_from_visual(&mut self) {
        if let Some(start_min) = self.visual_start_minute {
            let end_min = self.cursor_minute;
            let (min_m, max_m) = if start_min <= end_min {
                (start_min, end_min + self.granularity.minutes())
            } else {
                (end_min, start_min + self.granularity.minutes())
            };

            let date = self.cursor_date();
            let start_h = (min_m / 60).min(23);
            let start_m = min_m % 60;
            let end_h = (max_m / 60).min(23);
            let end_m = if max_m >= 1440 { 59 } else { max_m % 60 };

            self.form_block_id = None;
            self.form_title = String::new();
            self.form_category_idx = 0;
            self.form_date = date;
            self.form_start_time = NaiveTime::from_hms_opt(start_h, start_m, 0).unwrap();
            self.form_end_time = NaiveTime::from_hms_opt(end_h, end_m, 0).unwrap();
            self.form_description = String::new();
            self.form_field = FormField::Title;
            self.visual_start_minute = None;
            self.mode = AppMode::Insert;
        }
    }

    pub fn save_form(&mut self) {
        let title = if self.form_title.trim().is_empty() {
            "Untitled Event".to_string()
        } else {
            self.form_title.trim().to_string()
        };

        let cat_id = if let Some(cat) = self.categories.get(self.form_category_idx) {
            cat.id.clone()
        } else {
            "cat_task".to_string()
        };

        let start_dt = self
            .form_date
            .and_time(self.form_start_time)
            .and_local_timezone(Local)
            .unwrap();

        let mut end_time = self.form_end_time;
        if end_time <= self.form_start_time {
            // Ensure end time is at least 15m after start time
            let plus_min = (self.form_start_time.hour() * 60 + self.form_start_time.minute() + 15).min(1439);
            end_time = NaiveTime::from_hms_opt(plus_min / 60, plus_min % 60, 0).unwrap();
        }

        let end_dt = self
            .form_date
            .and_time(end_time)
            .and_local_timezone(Local)
            .unwrap();

        let block = TimeBlock {
            id: self.form_block_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
            title,
            description: self.form_description.clone(),
            category_id: cat_id,
            start_time: start_dt,
            end_time: end_dt,
            all_day: false,
        };

        if let Some(db) = &self.db {
            if self.form_block_id.is_some() {
                let _ = db.update_block(&block);
                self.set_status("Block updated.");
            } else {
                let _ = db.insert_block(&block);
                self.set_status("Block created.");
            }
        }

        self.mode = AppMode::Normal;
        self.refresh_blocks();
    }

    pub fn delete_selected_block(&mut self) {
        if let Some(id) = self.selected_block_id.clone() {
            if let Some(db) = &self.db {
                let _ = db.delete_block(&id);
                self.set_status("Block deleted.");
                self.refresh_blocks();
            }
        } else {
            self.set_status("No block selected to delete.");
        }
    }

    pub fn nudge_block(&mut self, minutes: i32) {
        if let Some(mut block) = self.get_selected_block().cloned() {
            let dur = block.end_time - block.start_time;
            if minutes > 0 {
                block.start_time = block.start_time + Duration::minutes(minutes as i64);
                block.end_time = block.start_time + dur;
            } else {
                block.start_time = block.start_time - Duration::minutes((-minutes) as i64);
                block.end_time = block.start_time + dur;
            }

            if let Some(db) = &self.db {
                let _ = db.update_block(&block);
                self.refresh_blocks();
            }
        }
    }

    pub fn resize_block(&mut self, minutes: i32) {
        if let Some(mut block) = self.get_selected_block().cloned() {
            let new_end = if minutes > 0 {
                block.end_time + Duration::minutes(minutes as i64)
            } else {
                block.end_time - Duration::minutes((-minutes) as i64)
            };

            if (new_end - block.start_time).num_minutes() >= 15 {
                block.end_time = new_end;
                if let Some(db) = &self.db {
                    let _ = db.update_block(&block);
                    self.refresh_blocks();
                }
            }
        }
    }

    pub fn yank_selected_block(&mut self) {
        if let Some(b) = self.get_selected_block() {
            self.yanked_block = Some(b.clone());
            self.set_status("Block yanked (copied). Press 'p' to paste.");
        }
    }

    pub fn paste_yanked_block(&mut self) {
        if let Some(yanked) = self.yanked_block.clone() {
            let dur = yanked.end_time - yanked.start_time;
            let c_date = self.cursor_date();
            let c_min = self.cursor_minute;
            let start_dt = c_date
                .and_hms_opt(c_min / 60, c_min % 60, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap();
            let end_dt = start_dt + dur;

            let new_block = TimeBlock {
                id: Uuid::new_v4().to_string(),
                title: yanked.title,
                description: yanked.description,
                category_id: yanked.category_id,
                start_time: start_dt,
                end_time: end_dt,
                all_day: yanked.all_day,
            };

            if let Some(db) = &self.db {
                let _ = db.insert_block(&new_block);
                self.set_status("Block pasted.");
                self.refresh_blocks();
            }
        }
    }

    pub fn execute_command(&mut self) {
        let cmd = self.command_buffer.trim().to_string();
        self.command_buffer.clear();
        self.mode = AppMode::Normal;

        if cmd == "q" || cmd == "q!" || cmd == "quit" {
            self.should_quit = true;
        } else if cmd == "w" || cmd == "write" {
            self.set_status("Database synced.");
        } else if cmd.starts_with("db ") {
            let path_str = cmd[3..].trim();
            let path = PathBuf::from(path_str);
            self.connect_db(&path);
            let mut cfg = self.config.clone();
            cfg.db_path = Some(path_str.to_string());
            let _ = ConfigManager::save_config(&cfg);
            self.config = cfg;
        } else if cmd.starts_with("goto ") {
            let target = cmd[5..].trim();
            if target == "today" {
                self.jump_to_now();
            } else if let Ok(date) = NaiveDate::parse_from_str(target, "%Y-%m-%d") {
                self.current_date = date;
                self.cursor_day_offset = 0;
                self.refresh_blocks();
                self.set_status(format!("Navigated to {}", date));
            } else {
                self.set_status(format!("Invalid date format (use YYYY-MM-DD): {}", target));
            }
        } else if !cmd.is_empty() {
            self.set_status(format!("Unknown command: :{}", cmd));
        }
    }

    pub fn complete_wizard(&mut self) {
        let chosen_path = match self.wizard_selected_option {
            0 => ConfigManager::vault_preset_path(),
            1 => ConfigManager::xdg_data_default_path(),
            2 => {
                if self.wizard_custom_path.trim().is_empty() {
                    ConfigManager::vault_preset_path()
                } else {
                    PathBuf::from(self.wizard_custom_path.trim())
                }
            }
            _ => ConfigManager::temp_fallback_path(),
        };

        let _ = ConfigManager::ensure_parent_dirs(&chosen_path);
        let mut cfg = self.config.clone();
        cfg.db_path = Some(chosen_path.to_string_lossy().to_string());
        let _ = ConfigManager::save_config(&cfg);
        self.config = cfg;

        self.connect_db(&chosen_path);
        self.mode = AppMode::Normal;
    }
}
