use ferrolens::data::types::{CellValue, ColumnDef, ColumnType, Dataset};
use std::path::PathBuf;

#[test]
fn dataset_tracks_columns_rows_and_warnings() {
    let dataset = Dataset::new(
        vec![ColumnDef::new("gene", ColumnType::String)],
        vec![vec![CellValue::String("TP53".into())]],
    );

    assert_eq!(dataset.columns.len(), 1);
    assert_eq!(dataset.rows.len(), 1);
    assert!(dataset.warnings.is_empty());
}

#[test]
fn dataset_supports_genomic_locus_column_type() {
    let column = ColumnDef::new("locus", ColumnType::GenomicLocus);
    assert_eq!(column.column_type, ColumnType::GenomicLocus);
}

#[test]
fn detects_tab_delimited_files() {
    let path = PathBuf::from("tests/fixtures/example.tsv");
    let dataset = ferrolens::data::loader::delimited::load_dataset(&path).unwrap();

    assert_eq!(dataset.columns.len(), 3);
    assert_eq!(dataset.rows.len(), 2);
}

#[test]
fn detects_comma_delimited_files() {
    let path = PathBuf::from("tests/fixtures/example.csv");
    let dataset = ferrolens::data::loader::delimited::load_dataset(&path).unwrap();

    assert_eq!(dataset.columns.len(), 3);
    assert_eq!(dataset.rows.len(), 2);
}

#[test]
fn records_warnings_for_malformed_rows() {
    let path = PathBuf::from("tests/fixtures/dirty.txt");
    let dataset = ferrolens::data::loader::delimited::load_dataset(&path).unwrap();

    assert!(!dataset.warnings.is_empty());
}

#[test]
fn loader_dispatches_delimited_files_by_extension() {
    let path = PathBuf::from("tests/fixtures/example.tsv");
    let dataset = ferrolens::data::loader::load_dataset(&path).unwrap();

    assert_eq!(dataset.rows.len(), 2);
}
