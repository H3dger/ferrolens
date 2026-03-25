#[test]
fn cli_requires_an_input_path() {
    let result = ferrolens::cli::parse_from(["ferrolens"]);
    assert!(result.is_err());
}

#[test]
fn cli_accepts_theme_flag() {
    let result = ferrolens::cli::parse_from(["ferrolens", "--theme", "catppuccin", "results.tsv"]);
    assert!(result.is_ok());
}

#[test]
fn cli_exposes_ferrolens_command_name() {
    use clap::CommandFactory;

    let command = ferrolens::cli::Cli::command();
    assert_eq!(command.get_name(), "ferrolens");
}

#[test]
fn help_flag_is_not_treated_as_runtime_error() {
    let result = ferrolens::run_with_args(["ferrolens", "--help"]);
    assert!(result.is_ok());
}

#[test]
fn moving_selection_updates_current_row() {
    use ferrolens::app::App;
    use ferrolens::data::types::{CellValue, ColumnDef, ColumnType, Dataset};

    let dataset = Dataset::new(
        vec![ColumnDef::new("gene", ColumnType::String)],
        vec![
            vec![CellValue::String("TP53".into())],
            vec![CellValue::String("EGFR".into())],
        ],
    );

    let mut app = App::new(dataset);
    app.select_next();

    assert_eq!(app.selected_row(), Some(1));
}

#[test]
fn current_row_details_follow_selection() {
    use ferrolens::app::App;
    use ferrolens::data::types::{CellValue, ColumnDef, ColumnType, Dataset};

    let dataset = Dataset::new(
        vec![ColumnDef::new("gene", ColumnType::String)],
        vec![
            vec![CellValue::String("TP53".into())],
            vec![CellValue::String("EGFR".into())],
        ],
    );

    let mut app = App::new(dataset);
    app.select_next();

    assert_eq!(
        app.current_row(),
        Some(&vec![CellValue::String("EGFR".into())])
    );
}

#[test]
fn render_state_reuses_cached_table_rows_until_state_changes() {
    use ferrolens::app::App;
    use ferrolens::data::types::{CellValue, ColumnDef, ColumnType, Dataset};

    let dataset = Dataset::new(
        vec![
            ColumnDef::new("gene", ColumnType::String),
            ColumnDef::new("AF", ColumnType::Float),
        ],
        vec![
            vec![
                CellValue::String("TP53".into()),
                CellValue::String("0.5".into()),
            ],
            vec![
                CellValue::String("EGFR".into()),
                CellValue::String("0.1".into()),
            ],
        ],
    );

    let mut app = App::new(dataset);

    let _ = app.render_state();
    let first = app.debug_table_rows_build_count();
    let _ = app.render_state();
    let second = app.debug_table_rows_build_count();
    assert_eq!(first, second);

    app.sort_by_column("AF", true).unwrap();
    let _ = app.render_state();
    let third = app.debug_table_rows_build_count();
    assert!(third > second);
}

#[test]
fn default_layout_exposes_table_and_detail_panes() {
    use ferrolens::app::App;
    use ferrolens::data::types::{CellValue, ColumnDef, ColumnType, Dataset};
    use ferrolens::ui::PaneId;

    let dataset = Dataset::new(
        vec![ColumnDef::new("gene", ColumnType::String)],
        vec![vec![CellValue::String("TP53".into())]],
    );

    let app = App::new(dataset);
    let layout = app.render_state();

    assert_eq!(
        layout.panes,
        vec![
            PaneId::TopBar,
            PaneId::LeftSidebar,
            PaneId::Table,
            PaneId::Detail,
            PaneId::BottomBar,
        ]
    );
}

#[test]
fn selected_theme_changes_render_palette() {
    use ferrolens::theme::{ThemeName, ThemePalette};

    let default_palette = ThemePalette::from_theme(ThemeName::Default);
    let catppuccin_palette = ThemePalette::from_theme(ThemeName::Catppuccin);

    assert_ne!(
        default_palette.table_header,
        catppuccin_palette.table_header
    );
    assert_ne!(default_palette.status_bar, catppuccin_palette.status_bar);
}

#[test]
fn sorting_changes_visible_row_order() {
    use ferrolens::app::App;
    use ferrolens::data::types::{CellValue, ColumnDef, ColumnType, Dataset};

    let dataset = Dataset::new(
        vec![
            ColumnDef::new("gene", ColumnType::String),
            ColumnDef::new("AF", ColumnType::Float),
        ],
        vec![
            vec![
                CellValue::String("EGFR".into()),
                CellValue::String("0.120".into()),
            ],
            vec![
                CellValue::String("TP53".into()),
                CellValue::String("0.005".into()),
            ],
        ],
    );

    let mut app = App::new(dataset);
    app.sort_by_column("AF", true).unwrap();

    assert_eq!(
        app.current_row(),
        Some(&vec![
            CellValue::String("TP53".into()),
            CellValue::String("0.005".into()),
        ])
    );
}

#[test]
fn hidden_columns_are_removed_from_render_state() {
    use ferrolens::app::App;
    use ferrolens::data::types::{CellValue, ColumnDef, ColumnType, Dataset};

    let dataset = Dataset::new(
        vec![
            ColumnDef::new("gene", ColumnType::String),
            ColumnDef::new("impact", ColumnType::Categorical),
        ],
        vec![vec![
            CellValue::String("TP53".into()),
            CellValue::String("HIGH".into()),
        ]],
    );

    let mut app = App::new(dataset);
    app.hide_column("impact").unwrap();
    let render_state = app.render_state();

    assert_eq!(render_state.visible_columns, vec!["gene".to_string()]);
}

#[test]
fn detail_pane_prioritizes_variant_review_fields() {
    use std::path::PathBuf;

    use ferrolens::app::App;

    let dataset =
        ferrolens::data::loader::vcf::load_dataset(&PathBuf::from("tests/fixtures/example.vcf"))
            .unwrap();
    let app = App::new(dataset);
    let render_state = app.render_state();

    let labels = render_state
        .detail_fields
        .iter()
        .map(|field| field.label.as_str())
        .collect::<Vec<_>>();

    let filter_idx = labels.iter().position(|label| *label == "FILTER").unwrap();
    let dp_idx = labels.iter().position(|label| *label == "DP").unwrap();
    let af_idx = labels.iter().position(|label| *label == "AF").unwrap();
    let info_idx = labels.iter().position(|label| *label == "INFO").unwrap();

    assert!(filter_idx < info_idx);
    assert!(dp_idx < info_idx);
    assert!(af_idx < info_idx);
}

#[test]
fn warnings_are_visible_in_status_ui() {
    use std::path::PathBuf;

    use ferrolens::app::App;

    let dataset = ferrolens::data::loader::delimited::load_dataset(&PathBuf::from(
        "tests/fixtures/dirty.txt",
    ))
    .unwrap();
    let app = App::new(dataset);
    let render_state = app.render_state();

    assert!(render_state.status_message.contains("warning"));
}

#[test]
fn unknown_vcf_info_is_preserved_raw() {
    use std::path::PathBuf;

    use ferrolens::app::App;

    let dataset =
        ferrolens::data::loader::vcf::load_dataset(&PathBuf::from("tests/fixtures/example.vcf"))
            .unwrap();
    let app = App::new(dataset);
    let render_state = app.render_state();

    let info_raw = render_state
        .detail_fields
        .iter()
        .find(|field| field.label == "INFO_RAW")
        .map(|field| field.value.clone())
        .unwrap();

    assert!(info_raw.contains("ZZ=note"));
}
