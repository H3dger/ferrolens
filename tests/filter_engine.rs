use ferrolens::app::App;
use ferrolens::data::types::{CellValue, ColumnDef, ColumnType, Dataset};
use std::fs;
use std::path::PathBuf;

fn variant_dataset() -> Dataset {
    Dataset::new(
        vec![
            ColumnDef::new("gene", ColumnType::String),
            ColumnDef::new("AF", ColumnType::Float),
            ColumnDef::new("FILTER", ColumnType::Categorical),
        ],
        vec![
            vec![
                CellValue::String("TP53".into()),
                CellValue::String("0.005".into()),
                CellValue::String("PASS".into()),
            ],
            vec![
                CellValue::String("EGFR".into()),
                CellValue::String("0.120".into()),
                CellValue::String("LowQual".into()),
            ],
            vec![
                CellValue::String("KRAS".into()),
                CellValue::String("0.009".into()),
                CellValue::String("PASS".into()),
            ],
        ],
    )
}

#[test]
fn numeric_filters_reduce_visible_rows() {
    let mut app = App::new(variant_dataset());
    app.apply_filter_str("AF < 0.01").unwrap();

    assert_eq!(app.visible_row_count(), 2);
}

#[test]
fn categorical_filters_match_expected_rows() {
    let mut app = App::new(variant_dataset());
    app.apply_filter_str("FILTER == PASS").unwrap();

    let genes = app
        .visible_rows()
        .into_iter()
        .map(|row| match &row[0] {
            CellValue::String(value) => value.clone(),
            _ => String::new(),
        })
        .collect::<Vec<_>>();

    assert_eq!(genes, vec!["TP53".to_string(), "KRAS".to_string()]);
}

#[test]
fn global_search_finds_gene_symbols() {
    let mut app = App::new(variant_dataset());
    app.set_search_query("tp53");

    assert_eq!(app.visible_row_count(), 1);
    assert_eq!(
        app.current_row(),
        Some(&vec![
            CellValue::String("TP53".into()),
            CellValue::String("0.005".into()),
            CellValue::String("PASS".into()),
        ])
    );
}

#[test]
fn exports_filtered_rows_to_a_new_file() {
    let mut app = App::new(variant_dataset());
    app.apply_filter_str("AF < 0.01").unwrap();

    let output = unique_temp_path("filtered-export.tsv");
    app.export_visible_rows(&output).unwrap();

    let content = fs::read_to_string(&output).unwrap();
    assert!(content.contains("TP53"));
    assert!(content.contains("KRAS"));
    assert!(!content.contains("EGFR"));

    let _ = fs::remove_file(output);
}

#[test]
fn export_never_mutates_input_file() {
    let input = unique_temp_path("source-input.tsv");
    let output = unique_temp_path("filtered-output.tsv");
    let source = "gene\taf\nTP53\t0.005\nEGFR\t0.120\n";
    fs::write(&input, source).unwrap();

    let original_bytes = fs::read(&input).unwrap();
    let dataset = ferrolens::data::loader::delimited::load_dataset(&input).unwrap();
    let mut app = App::new(dataset);
    app.apply_filter_str("af < 0.01").unwrap();
    app.export_visible_rows(&output).unwrap();

    assert_eq!(fs::read(&input).unwrap(), original_bytes);

    let _ = fs::remove_file(input);
    let _ = fs::remove_file(output);
}

fn unique_temp_path(filename: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("ferrolens-{nanos}-{filename}"))
}
