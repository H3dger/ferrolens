use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ferrolens::app::App;
use ferrolens::data::types::{CellValue, ColumnDef, ColumnType, Dataset};
use ferrolens::input::{InputMode, Session};
use ferrolens::theme::{ThemeName, ThemePalette};
use ferrolens::ui::{self, DetailField, PaneId};
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::{backend::TestBackend, Terminal};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

fn runtime_dataset() -> Dataset {
    let mut rows = Vec::new();
    for index in 0..15 {
        let gene = match index {
            0 => "TP53".to_string(),
            1 => "EGFR".to_string(),
            _ => format!("GENE{index}"),
        };
        let af = format!("0.{index:03}");
        let filter = if index % 2 == 0 { "PASS" } else { "LowQual" };
        rows.push(vec![
            CellValue::String(gene),
            CellValue::String(af),
            CellValue::String(filter.to_string()),
        ]);
    }

    Dataset::new(
        vec![
            ColumnDef::new("gene", ColumnType::String),
            ColumnDef::new("AF", ColumnType::Float),
            ColumnDef::new("FILTER", ColumnType::Categorical),
        ],
        rows,
    )
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn type_text(session: &mut Session, value: &str) {
    for ch in value.chars() {
        session.handle_key(key(KeyCode::Char(ch)));
    }
}

fn backspace_times(session: &mut Session, count: usize) {
    for _ in 0..count {
        session.handle_key(key(KeyCode::Backspace));
    }
}

fn selected_gene(session: &Session) -> String {
    match &session.app().current_row().unwrap()[0] {
        CellValue::String(value) => value.clone(),
        _ => String::new(),
    }
}

fn render_to_string(render_state: &ui::RenderState, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, render_state))
        .unwrap();

    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
}

fn detail_heavy_dataset() -> Dataset {
    let columns = (0..12)
        .map(|index| ColumnDef::new(format!("field_{index}"), ColumnType::String))
        .collect::<Vec<_>>();
    let rows = vec![
        (0..12)
            .map(|index| CellValue::String(format!("row0_value_{index}")))
            .collect::<Vec<_>>(),
        (0..12)
            .map(|index| CellValue::String(format!("row1_value_{index}")))
            .collect::<Vec<_>>(),
    ];

    Dataset::new(columns, rows)
}

fn sort_hotkey_dataset() -> Dataset {
    Dataset::new(
        vec![
            ColumnDef::new("gene", ColumnType::String),
            ColumnDef::new("AF", ColumnType::Float),
            ColumnDef::new("FILTER", ColumnType::Categorical),
        ],
        vec![
            vec![
                CellValue::String("EGFR".into()),
                CellValue::String("0.120".into()),
                CellValue::String("LowQual".into()),
            ],
            vec![
                CellValue::String("TP53".into()),
                CellValue::String("0.005".into()),
                CellValue::String("PASS".into()),
            ],
        ],
    )
}

fn categorical_hint_dataset() -> Dataset {
    Dataset::new(
        vec![ColumnDef::new("impact", ColumnType::Categorical)],
        vec![
            vec![CellValue::String("Alpha".into())],
            vec![CellValue::String("Beta".into())],
            vec![CellValue::String("Gamma".into())],
            vec![CellValue::String("Delta".into())],
            vec![CellValue::String("Epsilon".into())],
        ],
    )
}

fn wide_table_dataset() -> Dataset {
    Dataset::new(
        (0..12)
            .map(|index| ColumnDef::new(format!("COL{index}"), ColumnType::String))
            .collect(),
        vec![(0..12)
            .map(|index| CellValue::String(format!("value{index}")))
            .collect()],
    )
}

fn long_content_dataset() -> Dataset {
    Dataset::new(
        vec![
            ColumnDef::new("LONG_ANNOTATION_FIELD_NAME", ColumnType::String),
            ColumnDef::new("SECOND_COLUMN", ColumnType::String),
        ],
        vec![vec![
            CellValue::String(
                "EXTREMELY_LONG_ANNOTATION_VALUE_THAT_SHOULD_NOT_FIT_IN_THE_TABLE_VIEW".into(),
            ),
            CellValue::String("short".into()),
        ]],
    )
}

fn exported_path_from_status(status: &str) -> PathBuf {
    PathBuf::from(
        status
            .strip_prefix("Exported visible rows to ")
            .expect("status should expose export path"),
    )
}

fn catppuccin_color(color: catppuccin::Color) -> Color {
    Color::Rgb(color.rgb.r, color.rgb.g, color.rgb.b)
}

#[test]
fn normal_mode_keys_drive_navigation_scroll_and_focus() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('j')));
    session.handle_key(key(KeyCode::PageDown));
    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Char(']')));
    session.handle_key(key(KeyCode::Tab));

    assert_eq!(session.app().selected_row(), Some(11));

    let render_state = session.render_state();
    assert_eq!(render_state.focused_column, Some(1));
    assert_eq!(render_state.horizontal_offset, 1);
    assert_eq!(render_state.focused_pane, PaneId::Detail);
}

#[test]
fn column_focus_moves_separately_from_horizontal_scroll() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    assert_eq!(session.render_state().focused_column, Some(0));
    assert_eq!(session.render_state().horizontal_offset, 0);

    session.handle_key(key(KeyCode::Char('l')));

    let focused = session.render_state();
    assert_eq!(focused.focused_column, Some(1));
    assert_eq!(focused.horizontal_offset, 0);

    session.handle_key(key(KeyCode::Char(']')));

    let scrolled = session.render_state();
    assert_eq!(scrolled.focused_column, Some(1));
    assert_eq!(scrolled.horizontal_offset, 1);
}

#[test]
fn search_mode_applies_query_on_enter() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('/')));
    type_text(&mut session, "egfr");

    assert_eq!(session.input_mode(), InputMode::Search);
    assert!(session
        .render_state()
        .status_message
        .contains("Search: egfr"));

    session.handle_key(key(KeyCode::Enter));

    assert_eq!(session.input_mode(), InputMode::Normal);
    assert_eq!(session.app().visible_row_count(), 1);
    assert_eq!(selected_gene(&session), "EGFR".to_string());
}

#[test]
fn filter_hotkey_prefills_string_column_template() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('f')));

    assert_eq!(session.input_mode(), InputMode::Filter);
    assert!(session
        .render_state()
        .status_message
        .contains("Filter: gene == "));
}

#[test]
fn filter_hotkey_prefills_numeric_column_template() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Char('f')));

    assert_eq!(session.input_mode(), InputMode::Filter);
    assert!(session
        .render_state()
        .status_message
        .contains("Filter: AF < "));
}

#[test]
fn filter_hotkey_prefills_categorical_template_and_shows_values_hint() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Char('f')));

    let status = session.render_state().status_message;
    assert_eq!(session.input_mode(), InputMode::Filter);
    assert!(status.contains("Filter: FILTER in [ "));
    assert!(status.contains("Values: PASS, LowQual"));
}

#[test]
fn categorical_values_hint_stays_concise_when_many_unique_values_exist() {
    let mut session = Session::new(App::new(categorical_hint_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('f')));

    let status = session.render_state().status_message;
    assert!(status.contains("Values: Alpha, Beta, Gamma, …"));
}

#[test]
fn escape_in_search_mode_clears_active_search_and_restores_full_row_set() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('/')));
    type_text(&mut session, "egfr");
    session.handle_key(key(KeyCode::Enter));

    assert_eq!(session.app().visible_row_count(), 1);
    assert_eq!(selected_gene(&session), "EGFR".to_string());

    session.handle_key(key(KeyCode::Char('/')));
    session.handle_key(key(KeyCode::Esc));

    assert_eq!(session.input_mode(), InputMode::Normal);
    assert_eq!(session.app().visible_row_count(), 15);
    assert_eq!(selected_gene(&session), "TP53".to_string());
}

#[test]
fn reset_hotkey_restores_full_view_baseline() {
    let mut app = App::new(runtime_dataset());
    app.set_search_query("egfr");
    app.apply_filter_str("FILTER == LowQual").unwrap();
    app.sort_by_column("AF", false).unwrap();
    app.hide_column("FILTER").unwrap();

    let mut session = Session::new(app, "results.tsv");
    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Tab));
    session.handle_key(key(KeyCode::Char('j')));

    session.handle_key(key(KeyCode::Char('r')));

    let render_state = session.render_state();
    assert_eq!(session.input_mode(), InputMode::Normal);
    assert_eq!(session.app().visible_row_count(), 15);
    assert_eq!(selected_gene(&session), "TP53".to_string());
    assert_eq!(render_state.visible_columns, vec!["gene", "AF", "FILTER"]);
    assert_eq!(render_state.horizontal_offset, 0);
    assert_eq!(render_state.focused_pane, PaneId::Table);
    assert_eq!(render_state.detail_scroll, 0);
    assert!(render_state.status_message.contains("reset"));
}

#[test]
fn invalid_filter_is_reported_in_bottom_status_area() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('f')));
    backspace_times(&mut session, "gene == ".len());
    type_text(&mut session, "AF ~= 0.01");
    session.handle_key(key(KeyCode::Enter));

    assert_eq!(session.input_mode(), InputMode::Normal);
    assert_eq!(session.app().visible_row_count(), 15);
    assert!(session
        .render_state()
        .status_message
        .contains("unsupported filter expression"));
}

#[test]
fn normal_mode_status_surfaces_active_sort_and_filter_summary() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('f')));
    type_text(&mut session, "TP53");
    session.handle_key(key(KeyCode::Enter));
    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Char('S')));

    let status_message = session.render_state().status_message;
    assert!(status_message.contains("Sort: AF asc"));
    assert!(status_message.contains("Filters: gene == TP53"));
}

#[test]
fn sort_mode_applies_column_sorting() {
    let mut session = Session::new(
        App::with_theme(runtime_dataset(), ThemeName::Catppuccin),
        "results.tsv",
    );

    session.handle_key(key(KeyCode::Char('s')));
    type_text(&mut session, "AF desc");
    session.handle_key(key(KeyCode::Enter));

    assert_eq!(session.input_mode(), InputMode::Normal);
    assert_eq!(selected_gene(&session), "GENE14".to_string());
    assert_eq!(
        session.render_state().palette,
        ThemePalette::from_theme(ThemeName::Catppuccin)
    );
}

#[test]
fn catppuccin_palette_maps_mocha_roles_semantically() {
    let palette = ThemePalette::from_theme(ThemeName::Catppuccin);
    let mocha = catppuccin::PALETTE.mocha.colors;

    assert_eq!(palette.accent, catppuccin_color(mocha.mauve));
    assert_eq!(palette.focused_column_fg, catppuccin_color(mocha.lavender));
    assert_eq!(palette.current_cell_fg, catppuccin_color(mocha.lavender));
    assert_eq!(palette.sorted_header_fg, catppuccin_color(mocha.teal));
    assert_eq!(palette.filtered_header_fg, catppuccin_color(mocha.yellow));
    assert_eq!(palette.warning, catppuccin_color(mocha.peach));
    assert_eq!(palette.error, catppuccin_color(mocha.red));
    assert_eq!(palette.background, catppuccin_color(mocha.base));
    assert_eq!(palette.top_bar_bg, Color::Reset);
    assert_eq!(palette.bottom_bar_bg, Color::Reset);
    assert_eq!(palette.detail_bg, Color::Reset);
    assert_eq!(palette.current_cell_bg, catppuccin_color(mocha.surface1));
    assert_eq!(palette.focused_header_bg, catppuccin_color(mocha.surface0));
    assert_eq!(palette.success, catppuccin_color(mocha.green));
}

#[test]
fn sort_hotkey_operates_on_current_focused_column() {
    let mut session = Session::new(App::new(sort_hotkey_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Char('S')));

    assert_eq!(selected_gene(&session), "TP53".to_string());
    assert!(session.render_state().status_message.contains("AF"));
}

#[test]
fn hide_current_column_hotkey_uses_focused_column() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Char('H')));

    let render_state = session.render_state();
    assert_eq!(render_state.visible_columns, vec!["gene", "FILTER"]);
    assert!(render_state.status_message.contains("AF"));
}

#[test]
fn table_header_marks_focused_sorted_and_filtered_columns() {
    let mut session = Session::new(App::new(sort_hotkey_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Char('f')));
    type_text(&mut session, "PASS");
    session.handle_key(key(KeyCode::Enter));
    session.handle_key(key(KeyCode::Char('h')));
    session.handle_key(key(KeyCode::Char('S')));
    session.handle_key(key(KeyCode::Char('h')));

    let rendered = render_to_string(&session.render_state(), 120, 24);

    assert!(rendered.contains("[gene]"));
    assert!(rendered.contains("AF↑"));
    assert!(rendered.contains("FILTER*"));
}

#[test]
fn export_hotkey_writes_visible_rows_to_a_new_temp_file() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Char('/')));
    type_text(&mut session, "egfr");
    session.handle_key(key(KeyCode::Enter));
    session.handle_key(key(KeyCode::Char('e')));

    let status_message = session.render_state().status_message;
    let output = exported_path_from_status(&status_message);
    let content = fs::read_to_string(&output).unwrap();

    assert!(content.contains("EGFR"));
    assert!(!content.contains("TP53"));
    assert!(status_message.contains(output.to_string_lossy().as_ref()));

    let _ = fs::remove_file(output);
}

#[test]
fn catppuccin_render_blends_large_panes_with_terminal_background_and_keeps_green_success() {
    let mut session = Session::new(
        App::with_theme(runtime_dataset(), ThemeName::Catppuccin),
        "results.tsv",
    );
    session.handle_key(key(KeyCode::Char('e')));

    let render_state = session.render_state();
    let backend = TestBackend::new(100, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &render_state))
        .unwrap();

    let sections = ui::layout_sections(Rect::new(0, 0, 100, 24));
    let buffer = terminal.backend().buffer();
    let top_cell = buffer
        .cell((sections.top_bar.x + 2, sections.top_bar.y + 1))
        .unwrap();
    let detail_cell = buffer
        .cell((sections.detail.x + 2, sections.detail.y + 1))
        .unwrap();
    let bottom_cell = buffer
        .cell((sections.bottom_bar.x + 2, sections.bottom_bar.y + 1))
        .unwrap();
    let export_cell = (sections.bottom_bar.x..sections.bottom_bar.x + sections.bottom_bar.width)
        .filter_map(|x| buffer.cell((x, sections.bottom_bar.y + 1)))
        .find(|cell| cell.symbol() == "E")
        .unwrap();

    assert_eq!(top_cell.bg, Color::Reset);
    assert_eq!(detail_cell.bg, Color::Reset);
    assert_eq!(bottom_cell.bg, Color::Reset);
    assert_eq!(export_cell.fg, render_state.palette.success);
}

#[test]
fn selected_row_band_extends_through_highlight_gutter() {
    let session = Session::new(
        App::with_theme(runtime_dataset(), ThemeName::Catppuccin),
        "results.tsv",
    );
    let render_state = session.render_state();
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &render_state))
        .unwrap();

    let sections = ui::layout_sections(Rect::new(0, 0, 120, 24));
    let buffer = terminal.backend().buffer();
    let gutter_cell = buffer
        .cell((sections.table.x + 1, sections.table.y + 2))
        .unwrap();
    let row_cell = (sections.table.x..sections.table.x + sections.table.width)
        .filter_map(|x| buffer.cell((x, sections.table.y + 2)))
        .find(|cell| cell.symbol() == "0")
        .unwrap();

    assert_eq!(gutter_cell.symbol(), "▎");
    assert_eq!(gutter_cell.bg, render_state.palette.selected_row_bg);
    assert_eq!(row_cell.bg, render_state.palette.selected_row_bg);
}

#[test]
fn catppuccin_top_and_bottom_bars_use_layered_information_colors() {
    let mut session = Session::new(
        App::with_theme(runtime_dataset(), ThemeName::Catppuccin),
        "results.tsv",
    );
    session.handle_key(key(KeyCode::Char('f')));
    type_text(&mut session, "TP53");
    session.handle_key(key(KeyCode::Enter));
    session.handle_key(key(KeyCode::Char('l')));
    session.handle_key(key(KeyCode::Char('S')));

    let render_state = session.render_state();
    let backend = TestBackend::new(140, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &render_state))
        .unwrap();

    let sections = ui::layout_sections(Rect::new(0, 0, 140, 24));
    let buffer = terminal.backend().buffer();
    let top_row = (sections.top_bar.x..sections.top_bar.x + sections.top_bar.width)
        .filter_map(|x| buffer.cell((x, sections.top_bar.y + 1)))
        .collect::<Vec<_>>();
    let bottom_row = (sections.bottom_bar.x..sections.bottom_bar.x + sections.bottom_bar.width)
        .filter_map(|x| buffer.cell((x, sections.bottom_bar.y + 1)))
        .collect::<Vec<_>>();

    assert!(top_row
        .iter()
        .any(|cell| cell.symbol() == "·" && cell.fg == render_state.palette.muted));
    assert!(top_row
        .iter()
        .any(|cell| cell.symbol() == "0" && cell.fg != render_state.palette.text));
    assert!(bottom_row
        .iter()
        .any(|cell| cell.fg == render_state.palette.accent));
    assert!(bottom_row
        .iter()
        .any(|cell| cell.symbol() == "A" && cell.fg == render_state.palette.sorted_header_fg));
    assert!(bottom_row
        .iter()
        .any(|cell| cell.symbol() == "g" && cell.fg == render_state.palette.filtered_header_fg));
}

#[test]
fn catppuccin_top_bar_uses_multiple_information_hues() {
    let session = Session::new(
        App::with_theme(runtime_dataset(), ThemeName::Catppuccin),
        "results.tsv",
    );
    let render_state = session.render_state();
    let backend = TestBackend::new(140, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &render_state))
        .unwrap();

    let sections = ui::layout_sections(Rect::new(0, 0, 140, 24));
    let colors = (sections.top_bar.x..sections.top_bar.x + sections.top_bar.width)
        .filter_map(|x| {
            terminal
                .backend()
                .buffer()
                .cell((x, sections.top_bar.y + 1))
        })
        .filter(|cell| cell.symbol().chars().any(|ch| ch.is_alphanumeric()))
        .map(|cell| cell.fg)
        .collect::<HashSet<_>>();

    assert!(colors.contains(&render_state.palette.accent));
    assert!(colors.contains(&render_state.palette.focused_column_fg));
    assert!(colors.contains(&render_state.palette.filtered_header_fg));
    assert!(colors.contains(&render_state.palette.sorted_header_fg));
}

#[test]
fn widget_rendering_includes_sidebar_and_prompt_text() {
    let mut session = Session::new(App::new(runtime_dataset()), "results.tsv");
    session.handle_key(key(KeyCode::Char('/')));
    type_text(&mut session, "tp53");

    let render_state = session.render_state();
    let rendered = render_to_string(&render_state, 100, 30);

    assert!(rendered.contains("results.tsv"));
    assert!(rendered.contains("SEARCH"));
    assert!(rendered.contains("Search: tp53"));
    assert!(!rendered.contains("LeftSidebar"));
}

#[test]
fn detail_scroll_changes_visible_detail_pane_content() {
    let detail_fields = (0..12)
        .map(|index| DetailField {
            label: format!("DETAIL_{index}"),
            value: format!("detail-only-{index}"),
        })
        .collect::<Vec<_>>();

    let mut base_state = ui::RenderState::for_app(
        ThemeName::Default,
        vec!["gene".to_string()],
        Arc::new(vec![vec!["table-row".to_string()]]),
        Some(0),
        Some(0),
        0,
        detail_fields,
        Vec::new(),
        None,
        "Ready".to_string(),
    );
    base_state.sidebar_fields = vec![DetailField {
        label: "File".to_string(),
        value: "results.tsv".to_string(),
    }];
    base_state.focused_pane = PaneId::Detail;

    let unscrolled = render_to_string(&base_state, 140, 18);

    let mut scrolled_state = base_state.clone();
    scrolled_state.detail_scroll = 1;
    let scrolled = render_to_string(&scrolled_state, 140, 18);

    assert!(unscrolled.contains("DETAIL_0"));
    assert!(unscrolled.contains("detail-only-0"));
    assert!(!scrolled.contains("DETAIL_0"));
    assert!(!scrolled.contains("detail-only-0"));
    assert!(scrolled.contains("DETAIL_1"));
    assert!(scrolled.contains("detail-only-1"));
}

#[test]
fn detail_focus_scrolls_detail_instead_of_moving_table_selection() {
    let mut session = Session::new(App::new(detail_heavy_dataset()), "results.tsv");

    session.handle_key(key(KeyCode::Tab));
    session.handle_key(key(KeyCode::Char('j')));

    assert_eq!(session.app().selected_row(), Some(0));
    assert_eq!(session.render_state().focused_pane, PaneId::Detail);
    assert_eq!(session.render_state().detail_scroll, 1);
}

#[test]
fn layout_has_no_sidebar_and_uses_table_first_split() {
    let sections = ui::layout_sections(Rect::new(0, 0, 120, 30));
    let body_width = sections.table.width + sections.detail.width;

    assert_eq!(sections.left_sidebar.width, 0);
    assert_eq!(sections.table.x, 0);
    assert!(sections.table.width > sections.detail.width);
    assert!(sections.table.width >= sections.detail.width + 10);
    assert!(sections.table.width * 100 >= body_width * 78);
    assert!(sections.detail.width * 100 <= body_width * 22);
}

#[test]
fn wide_table_viewport_fits_more_columns_when_space_allows() {
    let session = Session::new(App::new(wide_table_dataset()), "results.tsv");
    let render_state = session.render_state();
    let backend = TestBackend::new(220, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &render_state))
        .unwrap();
    let sections = ui::layout_sections(Rect::new(0, 0, 220, 24));
    let header_row = (sections.table.x..sections.table.x + sections.table.width)
        .filter_map(|x| terminal.backend().buffer().cell((x, sections.table.y + 1)))
        .map(|cell| cell.symbol())
        .collect::<String>();

    for index in 0..10 {
        assert!(header_row.contains(&format!("COL{index}")));
    }
    assert!(!header_row.contains("COL10"));
    assert!(!header_row.contains("COL11"));
}

#[test]
fn visible_row_window_only_covers_viewport_slice() {
    let window = ui::visible_row_window(12_542, Some(6000), 20);

    assert!(window.start < 6000);
    assert!(window.end > 6000);
    assert!(window.end - window.start <= 20);
}

#[test]
fn catppuccin_plain_headers_use_differentiated_text_hues() {
    let session = Session::new(
        App::with_theme(wide_table_dataset(), ThemeName::Catppuccin),
        "results.tsv",
    );
    let render_state = session.render_state();
    let backend = TestBackend::new(220, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &render_state))
        .unwrap();

    let sections = ui::layout_sections(Rect::new(0, 0, 220, 24));
    let header_colors = (sections.table.x..sections.table.x + sections.table.width)
        .filter_map(|x| terminal.backend().buffer().cell((x, sections.table.y + 1)))
        .filter(|cell| cell.symbol().chars().any(|ch| ch.is_alphanumeric()))
        .map(|cell| cell.fg)
        .collect::<HashSet<_>>();

    assert!(header_colors.len() >= 4);
}

#[test]
fn table_render_truncates_long_headers_and_cells() {
    let session = Session::new(App::new(long_content_dataset()), "results.tsv");
    let rendered = render_to_string(&session.render_state(), 120, 20);

    assert!(rendered.contains("…"));
    assert!(!rendered.contains("LONG_ANNOTATION_FIELD_NAME"));
    assert!(
        !rendered.contains("EXTREMELY_LONG_ANNOTATION_VALUE_THAT_SHOULD_NOT_FIT_IN_THE_TABLE_VIEW")
    );
}

#[test]
fn focused_header_and_current_cell_use_distinct_render_styles() {
    let session = Session::new(
        App::with_theme(runtime_dataset(), ThemeName::Catppuccin),
        "results.tsv",
    );
    let render_state = session.render_state();
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| ui::render(frame, &render_state))
        .unwrap();

    let sections = ui::layout_sections(Rect::new(0, 0, 120, 24));
    let buffer = terminal.backend().buffer();
    let focused_header = (sections.table.x..sections.table.x + sections.table.width)
        .filter_map(|x| buffer.cell((x, sections.table.y + 1)))
        .find(|cell| cell.symbol() == "[")
        .unwrap();
    let unfocused_header = (sections.table.x..sections.table.x + sections.table.width)
        .filter_map(|x| buffer.cell((x, sections.table.y + 1)))
        .find(|cell| cell.symbol() == "A" && cell.fg != focused_header.fg)
        .unwrap();
    let selected_row_cell = (sections.table.x..sections.table.x + sections.table.width)
        .filter_map(|x| buffer.cell((x, sections.table.y + 2)))
        .find(|cell| cell.symbol() == "0")
        .unwrap();
    let focused_data_cell = (sections.table.x..sections.table.x + sections.table.width)
        .filter_map(|x| buffer.cell((x, sections.table.y + 2)))
        .find(|cell| cell.symbol() == "T")
        .unwrap();

    assert_eq!(focused_header.symbol(), "[");
    assert_ne!(focused_header.bg, unfocused_header.bg);
    assert_eq!(focused_data_cell.bg, render_state.palette.selected_row_bg);
    assert_eq!(selected_row_cell.bg, render_state.palette.selected_row_bg);
    assert_ne!(focused_data_cell.fg, selected_row_cell.fg);
}

#[test]
fn default_theme_uses_terminal_background_for_non_table_panes() {
    let session = Session::new(App::new(runtime_dataset()), "results.tsv");
    let render_state = session.render_state();
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| ui::render(frame, &render_state))
        .unwrap();

    let sections = ui::layout_sections(Rect::new(0, 0, 100, 30));
    let top_cell = terminal
        .backend()
        .buffer()
        .cell((sections.top_bar.x + 2, sections.top_bar.y + 1))
        .unwrap();
    let detail_cell = terminal
        .backend()
        .buffer()
        .cell((sections.detail.x + 2, sections.detail.y + 1))
        .unwrap();
    let bottom_cell = terminal
        .backend()
        .buffer()
        .cell((sections.bottom_bar.x + 2, sections.bottom_bar.y + 1))
        .unwrap();

    assert_eq!(top_cell.bg, Color::Reset);
    assert_eq!(detail_cell.bg, Color::Reset);
    assert_eq!(bottom_cell.bg, Color::Reset);
}
