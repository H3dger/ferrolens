use crate::app::App;
use crate::ui::{DetailField, PaneId, RenderState};
use anyhow::{anyhow, Result};
use crossterm::event::{KeyCode, KeyEvent};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const PAGE_SIZE: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    Filter,
    Sort,
}

pub struct Session {
    app: App,
    source_label: String,
    input_mode: InputMode,
    focused_pane: PaneId,
    pending_input: String,
    detail_scroll: u16,
    status_override: Option<String>,
    should_quit: bool,
}

impl Session {
    pub fn new(app: App, source_label: impl Into<String>) -> Self {
        Self {
            app,
            source_label: source_label.into(),
            input_mode: InputMode::Normal,
            focused_pane: PaneId::Table,
            pending_input: String::new(),
            detail_scroll: 0,
            status_override: None,
            should_quit: false,
        }
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key),
            _ => self.handle_input_mode(key),
        }
    }

    pub fn render_state(&self) -> RenderState {
        let mut state = self.app.render_state();
        state.source_label = self.source_label.clone();
        state.sidebar_fields = vec![
            DetailField {
                label: "File".to_string(),
                value: self.source_label.clone(),
            },
            DetailField {
                label: "Theme".to_string(),
                value: self.app.theme().as_str().to_string(),
            },
            DetailField {
                label: "Filters".to_string(),
                value: self.app.filter_count().to_string(),
            },
            DetailField {
                label: "Hidden".to_string(),
                value: self.app.hidden_column_count().to_string(),
            },
        ];
        state.mode_label = self.input_mode.label().to_string();
        state.focused_pane = self.focused_pane;
        state.detail_scroll = self.detail_scroll;
        state.status_message = self.status_message(&state.status_message);
        state
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('h') | KeyCode::Left => self.move_column_focus_left(),
            KeyCode::Char('l') | KeyCode::Right => self.move_column_focus_right(),
            KeyCode::Char('[') => self.app.scroll_left(),
            KeyCode::Char(']') => self.app.scroll_right(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::PageUp => self.page_up(),
            KeyCode::Char('/') => self.enter_mode(InputMode::Search),
            KeyCode::Char('f') => self.enter_mode(InputMode::Filter),
            KeyCode::Char('s') => self.enter_mode(InputMode::Sort),
            KeyCode::Char('S') => self.sort_focused_column(),
            KeyCode::Char('r') => self.reset_view(),
            KeyCode::Char('H') => self.hide_current_column(),
            KeyCode::Char('e') => self.export_visible_rows(),
            KeyCode::Tab => self.toggle_focus(),
            _ => {}
        }
    }

    fn handle_input_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.apply_pending_input(),
            KeyCode::Esc => self.cancel_input(),
            KeyCode::Backspace => {
                self.pending_input.pop();
            }
            KeyCode::Char(ch) => {
                self.pending_input.push(ch);
            }
            _ => {}
        }
    }

    fn enter_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
        self.pending_input = match mode {
            InputMode::Filter => self.app.focused_filter_prefill().unwrap_or_default(),
            _ => String::new(),
        };
        self.status_override = None;
    }

    fn cancel_input(&mut self) {
        if self.input_mode == InputMode::Search {
            self.app.set_search_query("");
            self.detail_scroll = 0;
        }
        self.input_mode = InputMode::Normal;
        self.pending_input.clear();
        self.status_override = None;
    }

    fn apply_pending_input(&mut self) {
        let result = match self.input_mode {
            InputMode::Normal => Ok(()),
            InputMode::Search => {
                self.app.set_search_query(self.pending_input.clone());
                Ok(())
            }
            InputMode::Filter => self.app.apply_filter_str(&self.pending_input),
            InputMode::Sort => parse_sort_expression(&self.pending_input)
                .and_then(|(column, ascending)| self.app.sort_by_column(&column, ascending)),
        };

        self.input_mode = InputMode::Normal;
        self.pending_input.clear();

        match result {
            Ok(()) => {
                self.detail_scroll = 0;
                self.status_override = None;
            }
            Err(error) => self.status_override = Some(format!("Error: {error}")),
        }
    }

    fn status_message(&self, app_status: &str) -> String {
        if self.input_mode != InputMode::Normal {
            let mut prompt = format!("{}: {}", self.input_mode.prompt_label(), self.pending_input);
            if self.input_mode == InputMode::Filter {
                if let Some(hint) = self.app.focused_categorical_hint() {
                    prompt.push_str(" | ");
                    prompt.push_str(&hint);
                }
            }
            return prompt;
        }

        let mut parts = Vec::new();
        if let Some(status) = &self.status_override {
            parts.push(status.clone());
        } else if app_status != "Ready" {
            parts.push(app_status.to_string());
        }
        if let Some(sort_summary) = self.app.sort_summary() {
            parts.push(format!("Sort: {sort_summary}"));
        }
        if let Some(filter_summary) = self.app.filter_summary() {
            parts.push(format!("Filters: {filter_summary}"));
        }
        if parts.is_empty() {
            parts.push(app_status.to_string());
        }

        parts.join(" | ")
    }

    fn toggle_focus(&mut self) {
        self.focused_pane = match self.focused_pane {
            PaneId::Table => PaneId::Detail,
            _ => PaneId::Table,
        };
    }

    fn move_down(&mut self) {
        if self.focused_pane == PaneId::Detail {
            self.scroll_detail_down(1);
        } else {
            self.app.select_next();
            self.detail_scroll = 0;
        }
    }

    fn move_up(&mut self) {
        if self.focused_pane == PaneId::Detail {
            self.detail_scroll = self.detail_scroll.saturating_sub(1);
        } else {
            self.app.select_previous();
            self.detail_scroll = 0;
        }
    }

    fn page_down(&mut self) {
        if self.focused_pane == PaneId::Detail {
            self.scroll_detail_down(PAGE_SIZE as u16);
        } else {
            self.app.page_down(PAGE_SIZE);
            self.detail_scroll = 0;
        }
    }

    fn page_up(&mut self) {
        if self.focused_pane == PaneId::Detail {
            self.detail_scroll = self.detail_scroll.saturating_sub(PAGE_SIZE as u16);
        } else {
            self.app.page_up(PAGE_SIZE);
            self.detail_scroll = 0;
        }
    }

    fn scroll_detail_down(&mut self, amount: u16) {
        let max_scroll = self
            .app
            .render_state()
            .detail_fields
            .len()
            .saturating_sub(1)
            .min(u16::MAX as usize) as u16;
        self.detail_scroll = self.detail_scroll.saturating_add(amount).min(max_scroll);
    }

    fn move_column_focus_left(&mut self) {
        if self.focused_pane == PaneId::Table {
            self.app.focus_previous_column();
        }
    }

    fn move_column_focus_right(&mut self) {
        if self.focused_pane == PaneId::Table {
            self.app.focus_next_column();
        }
    }

    fn reset_view(&mut self) {
        self.app.reset_view();
        self.input_mode = InputMode::Normal;
        self.pending_input.clear();
        self.detail_scroll = 0;
        self.focused_pane = PaneId::Table;
        self.status_override = Some("View reset".to_string());
    }

    fn hide_current_column(&mut self) {
        match self.app.hide_current_visible_column() {
            Ok(column) => {
                self.detail_scroll = 0;
                self.status_override = Some(format!("Hidden column: {column}"));
            }
            Err(error) => self.status_override = Some(format!("Error: {error}")),
        }
    }

    fn sort_focused_column(&mut self) {
        match self.app.sort_focused_column_toggle() {
            Ok((column, ascending)) => {
                self.detail_scroll = 0;
                let direction = if ascending { "ascending" } else { "descending" };
                self.status_override = Some(format!("Sorted by {column} ({direction})"));
            }
            Err(error) => self.status_override = Some(format!("Error: {error}")),
        }
    }

    fn export_visible_rows(&mut self) {
        let output = self.build_export_path();
        match self.app.export_visible_rows(&output) {
            Ok(()) => {
                self.status_override =
                    Some(format!("Exported visible rows to {}", output.display()));
            }
            Err(error) => self.status_override = Some(format!("Error: {error}")),
        }
    }

    fn build_export_path(&self) -> PathBuf {
        let stem = Path::new(&self.source_label)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("export")
            .chars()
            .map(|ch| match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
                _ => '-',
            })
            .collect::<String>();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        std::env::temp_dir().join(format!("ferrolens-{stem}-{nanos}.tsv"))
    }
}

impl InputMode {
    fn label(self) -> &'static str {
        match self {
            InputMode::Normal => "NORMAL",
            InputMode::Search => "SEARCH",
            InputMode::Filter => "FILTER",
            InputMode::Sort => "SORT",
        }
    }

    fn prompt_label(self) -> &'static str {
        match self {
            InputMode::Normal => "Ready",
            InputMode::Search => "Search",
            InputMode::Filter => "Filter",
            InputMode::Sort => "Sort",
        }
    }
}

fn parse_sort_expression(input: &str) -> Result<(String, bool)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("sort expression cannot be empty"));
    }

    let lower = trimmed.to_ascii_lowercase();
    if let Some(column) = trimmed.strip_prefix('-') {
        return sort_column(column, false);
    }
    if lower.ends_with(" desc") {
        return sort_column(&trimmed[..trimmed.len() - 5], false);
    }
    if lower.ends_with(" asc") {
        return sort_column(&trimmed[..trimmed.len() - 4], true);
    }

    sort_column(trimmed, true)
}

fn sort_column(column: &str, ascending: bool) -> Result<(String, bool)> {
    let trimmed = column.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("sort column cannot be empty"));
    }
    Ok((trimmed.to_string(), ascending))
}
