use crate::theme::{ThemeName, ThemePalette};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Alignment, Frame, Line, Style, Stylize};
use ratatui::style::Styled;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap};
use std::sync::Arc;

const TARGET_TABLE_COLUMN_WIDTH: u16 = 16;
const MIN_TABLE_COLUMN_WIDTH: u16 = 12;
const MAX_TABLE_COLUMN_WIDTH: u16 = 22;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaneId {
    TopBar,
    LeftSidebar,
    Table,
    Detail,
    BottomBar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetailField {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderState {
    pub panes: Vec<PaneId>,
    pub palette: ThemePalette,
    pub source_label: String,
    pub sidebar_fields: Vec<DetailField>,
    pub visible_columns: Vec<String>,
    pub table_rows: Arc<Vec<Vec<String>>>,
    pub selected_row: Option<usize>,
    pub focused_column: Option<usize>,
    pub horizontal_offset: usize,
    pub detail_fields: Vec<DetailField>,
    pub detail_scroll: u16,
    pub filtered_columns: Vec<String>,
    pub sort_state: Option<(String, bool)>,
    pub status_message: String,
    pub mode_label: String,
    pub focused_pane: PaneId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutSections {
    pub top_bar: Rect,
    pub left_sidebar: Rect,
    pub table: Rect,
    pub detail: Rect,
    pub bottom_bar: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RowWindow {
    pub start: usize,
    pub end: usize,
}

impl RenderState {
    pub fn for_app(
        theme: ThemeName,
        visible_columns: Vec<String>,
        table_rows: Arc<Vec<Vec<String>>>,
        selected_row: Option<usize>,
        focused_column: Option<usize>,
        horizontal_offset: usize,
        detail_fields: Vec<DetailField>,
        filtered_columns: Vec<String>,
        sort_state: Option<(String, bool)>,
        status_message: String,
    ) -> Self {
        Self {
            panes: vec![
                PaneId::TopBar,
                PaneId::LeftSidebar,
                PaneId::Table,
                PaneId::Detail,
                PaneId::BottomBar,
            ],
            palette: ThemePalette::from_theme(theme),
            source_label: String::new(),
            sidebar_fields: Vec::new(),
            visible_columns,
            table_rows,
            selected_row,
            focused_column,
            horizontal_offset,
            detail_fields,
            detail_scroll: 0,
            filtered_columns,
            sort_state,
            status_message,
            mode_label: "NORMAL".to_string(),
            focused_pane: PaneId::Table,
        }
    }
}

pub fn layout_sections(area: Rect) -> LayoutSections {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(80), Constraint::Fill(20)])
        .split(outer[1]);

    LayoutSections {
        top_bar: outer[0],
        left_sidebar: Rect::new(area.x, outer[1].y, 0, outer[1].height),
        table: body[0],
        detail: body[1],
        bottom_bar: outer[2],
    }
}

pub fn render(frame: &mut Frame, state: &RenderState) {
    let sections = layout_sections(frame.area());

    render_top_bar(frame, sections.top_bar, state);
    render_table(frame, sections.table, state);
    render_detail(frame, sections.detail, state);
    render_bottom_bar(frame, sections.bottom_bar, state);
}

pub fn visible_row_window(
    total_rows: usize,
    selected_row: Option<usize>,
    viewport_height: u16,
) -> RowWindow {
    if total_rows == 0 {
        return RowWindow { start: 0, end: 0 };
    }

    let height = usize::max(1, viewport_height.saturating_sub(4) as usize);
    let selected = selected_row.unwrap_or(0).min(total_rows - 1);
    let half = height / 2;
    let mut start = selected.saturating_sub(half);
    let end = usize::min(total_rows, start + height);
    if end - start < height {
        start = end.saturating_sub(height);
    }

    RowWindow { start, end }
}

fn render_top_bar(frame: &mut Frame, area: ratatui::layout::Rect, state: &RenderState) {
    let title = if state.source_label.is_empty() {
        "ferrolens".to_string()
    } else {
        state.source_label.clone()
    };
    let mut spans = vec!["ferrolens".fg(state.palette.text).bold().into()];
    if !title.is_empty() {
        spans.push(" · ".fg(state.palette.muted).into());
        spans.push(title.fg(state.palette.accent));
    }
    for field in state
        .sidebar_fields
        .iter()
        .filter(|field| field.label != "File")
    {
        spans.push(" · ".fg(state.palette.muted).into());
        spans.push(format!("{} ", field.label).fg(state.palette.muted).into());
        let value_color = match field.label.as_str() {
            "Theme" => state.palette.focused_column_fg,
            "Filters" => state.palette.filtered_header_fg,
            "Hidden" => state.palette.sorted_header_fg,
            _ => state.palette.status_bar,
        };
        spans.push(field.value.clone().fg(value_color));
    }

    let block = Block::default()
        .title(" TopBar ")
        .borders(Borders::ALL)
        .border_style(base_border_style(state));

    let paragraph = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left)
        .style(
            Style::default()
                .fg(state.palette.text)
                .bg(state.palette.top_bar_bg),
        )
        .block(block);

    frame.render_widget(paragraph, area);
}
fn render_table(frame: &mut Frame, area: ratatui::layout::Rect, state: &RenderState) {
    let viewport = table_viewport(state, area.width);

    if viewport.columns.is_empty() {
        let empty = Paragraph::new("No visible columns")
            .block(
                Block::default()
                    .title(" Table ")
                    .borders(Borders::ALL)
                    .border_style(focused_border_style(state, PaneId::Table)),
            )
            .style(
                Style::default()
                    .fg(state.palette.text)
                    .bg(state.palette.background),
            );
        frame.render_widget(empty, area);
        return;
    }

    let header =
        Row::new(viewport.columns.iter().map(|(index, column)| {
            build_header_cell(*index, column, state, viewport.column_width)
        }));
    let row_window = visible_row_window(state.table_rows.len(), state.selected_row, area.height);
    let rows = state.table_rows[row_window.start..row_window.end]
        .iter()
        .enumerate()
        .map(|(relative_index, row)| {
            let row_index = row_window.start + relative_index;
            Row::new(
                row.iter()
                    .enumerate()
                    .skip(viewport.start)
                    .take(viewport.columns.len())
                    .map(|(column_index, cell)| {
                        build_table_cell(
                            row_index,
                            column_index,
                            cell,
                            state,
                            viewport.column_width,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        });
    let widths = viewport
        .columns
        .iter()
        .map(|_| Constraint::Length(viewport.column_width))
        .collect::<Vec<_>>();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(" Table ")
                .borders(Borders::ALL)
                .border_style(focused_border_style(state, PaneId::Table)),
        )
        .row_highlight_style(Style::default().bg(state.palette.selected_row_bg))
        .highlight_symbol("▎ ");

    let mut table_state = TableState::default();
    table_state.select(
        state
            .selected_row
            .map(|index| index.saturating_sub(row_window.start)),
    );
    frame.render_stateful_widget(table, area, &mut table_state);
}

fn render_detail(frame: &mut Frame, area: ratatui::layout::Rect, state: &RenderState) {
    let lines = state
        .detail_fields
        .iter()
        .map(|field| {
            Line::from(vec![
                format!("{}: ", field.label).fg(state.palette.muted).into(),
                field.value.clone().fg(state.palette.text),
            ])
        })
        .collect::<Vec<_>>();

    let paragraph = Paragraph::new(lines)
        .scroll((state.detail_scroll, 0))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .title(" Detail ")
                .borders(Borders::ALL)
                .border_style(focused_border_style(state, PaneId::Detail)),
        )
        .style(
            Style::default()
                .fg(state.palette.text)
                .bg(state.palette.detail_bg),
        );

    frame.render_widget(paragraph, area);
}

fn render_bottom_bar(frame: &mut Frame, area: ratatui::layout::Rect, state: &RenderState) {
    let mut spans = vec![format!("{}", state.mode_label)
        .fg(state.palette.accent)
        .bold()
        .into()];
    for segment in state.status_message.split(" | ") {
        spans.push(" | ".fg(state.palette.muted).into());
        spans.extend(status_segment_spans(segment, state));
    }
    let paragraph = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .title(" BottomBar ")
                .borders(Borders::ALL)
                .border_style(base_border_style(state)),
        )
        .style(
            Style::default()
                .fg(state.palette.status_bar)
                .bg(state.palette.bottom_bar_bg),
        );

    frame.render_widget(paragraph, area);
}

fn base_border_style(state: &RenderState) -> Style {
    Style::default().fg(state.palette.border)
}

fn focused_border_style(state: &RenderState, pane: PaneId) -> Style {
    if state.focused_pane == pane {
        Style::default().fg(state.palette.accent).bold()
    } else {
        base_border_style(state)
    }
}

fn build_header_cell(
    index: usize,
    column: &str,
    state: &RenderState,
    column_width: u16,
) -> Cell<'static> {
    let is_filtered = state.filtered_columns.iter().any(|entry| entry == column);
    let sort_marker = state
        .sort_state
        .as_ref()
        .and_then(|(sorted_column, ascending)| {
            if sorted_column == column {
                Some(if *ascending { '↑' } else { '↓' })
            } else {
                None
            }
        });

    let mut label = column.to_string();
    if is_filtered {
        label.push('*');
    }
    if let Some(marker) = sort_marker {
        label.push(marker);
    }
    if state.focused_column == Some(index) {
        label = format!("[{label}]");
    }
    label = truncate_for_width(&label, column_width as usize);

    let mut style = Style::default()
        .fg(state.palette.header_hues[index % state.palette.header_hues.len()])
        .bold();
    if is_filtered {
        style = style.fg(state.palette.filtered_header_fg);
    }
    if sort_marker.is_some() {
        style = style.fg(state.palette.sorted_header_fg);
    }
    if state.focused_column == Some(index) {
        style = style
            .fg(state.palette.focused_header_fg)
            .bg(state.palette.focused_header_bg)
            .bold();
    }

    Cell::from(label).style(style)
}

fn build_table_cell(
    row_index: usize,
    column_index: usize,
    value: &str,
    state: &RenderState,
    column_width: u16,
) -> Cell<'static> {
    let mut style = Style::default().fg(state.palette.text);
    if state.selected_row == Some(row_index) {
        style = style
            .fg(state.palette.selected_row_fg)
            .bg(state.palette.selected_row_bg);
    }
    if state.focused_column == Some(column_index) {
        style = style.fg(state.palette.focused_column_fg);
    }
    if state.selected_row == Some(row_index) && state.focused_column == Some(column_index) {
        style = style.fg(state.palette.current_cell_fg).bold();
    }

    Cell::from(truncate_for_width(value, column_width as usize)).style(style)
}

fn truncate_for_width(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= width {
        return value.to_string();
    }

    if width == 1 {
        return "…".to_string();
    }

    chars.into_iter().take(width - 1).collect::<String>() + "…"
}

struct TableViewport {
    start: usize,
    columns: Vec<(usize, String)>,
    column_width: u16,
}

fn table_viewport(state: &RenderState, area_width: u16) -> TableViewport {
    let inner_width = area_width.saturating_sub(4);
    let visible_capacity = (inner_width / TARGET_TABLE_COLUMN_WIDTH).max(1) as usize;
    let columns = state
        .visible_columns
        .iter()
        .enumerate()
        .skip(state.horizontal_offset)
        .take(visible_capacity)
        .map(|(index, column)| (index, column.clone()))
        .collect::<Vec<_>>();
    let per_column_width = if columns.is_empty() {
        MIN_TABLE_COLUMN_WIDTH
    } else {
        (inner_width / columns.len() as u16).clamp(MIN_TABLE_COLUMN_WIDTH, MAX_TABLE_COLUMN_WIDTH)
    };

    TableViewport {
        start: state.horizontal_offset,
        columns,
        column_width: per_column_width,
    }
}

fn status_style(state: &RenderState) -> Style {
    let lower = state.status_message.to_ascii_lowercase();
    if lower.contains("error") {
        Style::default().fg(state.palette.error)
    } else if lower.contains("exported") {
        Style::default().fg(state.palette.success)
    } else if lower.contains("warning") {
        Style::default().fg(state.palette.warning)
    } else {
        Style::default().fg(state.palette.status_bar)
    }
}

fn status_segment_spans(segment: &str, state: &RenderState) -> Vec<ratatui::text::Span<'static>> {
    if let Some(value) = segment.strip_prefix("Sort: ") {
        return vec![
            "Sort: ".fg(state.palette.muted).into(),
            value.to_string().fg(state.palette.sorted_header_fg).into(),
        ];
    }
    if let Some(value) = segment.strip_prefix("Filters: ") {
        return vec![
            "Filters: ".fg(state.palette.muted).into(),
            value
                .to_string()
                .fg(state.palette.filtered_header_fg)
                .into(),
        ];
    }

    vec![segment.to_string().set_style(status_style(state)).into()]
}
