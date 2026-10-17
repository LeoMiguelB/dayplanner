pub mod grid;
pub mod header;
pub mod inspector;
pub mod modals;
pub mod status_bar;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;
use crate::model::AppMode;

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Split vertically: Header (length 1), Workspace (Min 10), Status Bar (length 1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top Header
            Constraint::Min(8),    // Calendar Grid & optional Inspector
            Constraint::Length(1), // Bottom Status Bar
        ])
        .split(size);

    header::render_header(f, app, chunks[0]);

    // Workspace: If inspector is shown, split horizontally into Grid + Inspector
    if app.show_inspector && chunks[1].width > 80 {
        let workspace_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(50),   // Grid
                Constraint::Length(34), // Inspector sidebar
            ])
            .split(chunks[1]);

        grid::render_grid(f, app, workspace_chunks[0]);
        inspector::render_inspector(f, app, workspace_chunks[1]);
    } else {
        grid::render_grid(f, app, chunks[1]);
    }

    status_bar::render_status_bar(f, app, chunks[2]);

    // Render active modal on top
    match app.mode {
        AppMode::Insert => {
            modals::render_form_modal(f, app, size);
        }
        AppMode::FirstRunWizard => {
            modals::render_wizard_modal(f, app, size);
        }
        AppMode::GranularityPicker => {
            modals::render_zoom_picker(f, app, size);
        }
        AppMode::Help => {
            modals::render_help_modal(f, size);
        }
        _ => {}
    }
}
