use chrono::{DateTime, Local, NaiveDate, Timelike};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Granularity {
    FifteenMinutes = 15,
    ThirtyMinutes = 30,
    SixtyMinutes = 60,
}

impl Granularity {
    pub fn minutes(self) -> u32 {
        self as u32
    }

    pub fn label(self) -> &'static str {
        match self {
            Granularity::FifteenMinutes => "15m",
            Granularity::ThirtyMinutes => "30m",
            Granularity::SixtyMinutes => "60m",
        }
    }

    #[allow(dead_code)]
    pub fn cycle_next(self) -> Self {
        match self {
            Granularity::FifteenMinutes => Granularity::ThirtyMinutes,
            Granularity::ThirtyMinutes => Granularity::SixtyMinutes,
            Granularity::SixtyMinutes => Granularity::FifteenMinutes,
        }
    }

    pub fn zoom_in(self) -> Self {
        match self {
            Granularity::SixtyMinutes => Granularity::ThirtyMinutes,
            Granularity::ThirtyMinutes => Granularity::FifteenMinutes,
            Granularity::FifteenMinutes => Granularity::FifteenMinutes,
        }
    }

    pub fn zoom_out(self) -> Self {
        match self {
            Granularity::FifteenMinutes => Granularity::ThirtyMinutes,
            Granularity::ThirtyMinutes => Granularity::SixtyMinutes,
            Granularity::SixtyMinutes => Granularity::SixtyMinutes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    Day,
    ThreeDay,
    Week,
}

impl ViewMode {
    pub fn days_count(self) -> usize {
        match self {
            ViewMode::Day => 1,
            ViewMode::ThreeDay => 3,
            ViewMode::Week => 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Visual,
    Insert,
    Command,
    FirstRunWizard,
    GranularityPicker,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub color_name: String,
}

impl Category {
    pub fn ratatui_color(&self) -> Color {
        match self.color_name.to_lowercase().as_str() {
            "cyan" | "blue" => Color::Cyan,
            "yellow" | "amber" => Color::Yellow,
            "green" => Color::Green,
            "magenta" | "purple" => Color::Magenta,
            "red" => Color::Red,
            "white" => Color::White,
            _ => Color::LightBlue,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeBlock {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category_id: String,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub all_day: bool,
}

impl TimeBlock {
    pub fn duration_minutes(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }

    pub fn overlaps_day(&self, date: NaiveDate) -> bool {
        let block_start_date = self.start_time.date_naive();
        let block_end_date = self.end_time.date_naive();
        block_start_date <= date && date <= block_end_date
    }

    pub fn start_minute_of_day(&self, date: NaiveDate) -> u32 {
        if self.start_time.date_naive() < date {
            0
        } else {
            self.start_time.hour() * 60 + self.start_time.minute()
        }
    }

    pub fn end_minute_of_day(&self, date: NaiveDate) -> u32 {
        if self.end_time.date_naive() > date {
            24 * 60
        } else {
            self.end_time.hour() * 60 + self.end_time.minute()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Title,
    Category,
    Date,
    StartTime,
    EndTime,
    Description,
}

impl FormField {
    pub fn next(self) -> Self {
        match self {
            FormField::Title => FormField::Category,
            FormField::Category => FormField::Date,
            FormField::Date => FormField::StartTime,
            FormField::StartTime => FormField::EndTime,
            FormField::EndTime => FormField::Description,
            FormField::Description => FormField::Title,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            FormField::Title => FormField::Description,
            FormField::Category => FormField::Title,
            FormField::Date => FormField::Category,
            FormField::StartTime => FormField::Date,
            FormField::EndTime => FormField::StartTime,
            FormField::Description => FormField::EndTime,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_granularity_transitions() {
        let g = Granularity::FifteenMinutes;
        assert_eq!(g.zoom_out(), Granularity::ThirtyMinutes);
        assert_eq!(g.zoom_out().zoom_out(), Granularity::SixtyMinutes);
        assert_eq!(g.zoom_out().zoom_out().zoom_out(), Granularity::SixtyMinutes);
        assert_eq!(g.zoom_out().zoom_in(), Granularity::FifteenMinutes);
    }

    #[test]
    fn test_view_mode_days() {
        assert_eq!(ViewMode::Day.days_count(), 1);
        assert_eq!(ViewMode::ThreeDay.days_count(), 3);
        assert_eq!(ViewMode::Week.days_count(), 7);
    }

    #[test]
    fn test_form_field_cycle() {
        let f = FormField::Title;
        assert_eq!(f.next(), FormField::Category);
        assert_eq!(f.next().prev(), FormField::Title);
    }
}
