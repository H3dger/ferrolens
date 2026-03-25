use std::fs;
use std::path::Path;

use crate::data::types::{CellValue, ColumnDef, ColumnType, Dataset};
use crate::error::Result;

pub fn load_dataset(path: &Path) -> Result<Dataset> {
    let content = fs::read_to_string(path)?;
    let delimiter = detect_delimiter(&content);

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .from_reader(content.as_bytes());

    let headers = reader.headers()?.clone();
    let columns = headers
        .iter()
        .map(|name| ColumnDef::new(name, ColumnType::String))
        .collect::<Vec<_>>();

    let expected_len = headers.len();
    let mut rows = Vec::new();
    let mut warnings = Vec::new();

    for (idx, record) in reader.records().enumerate() {
        match record {
            Ok(record) => {
                if record.len() != expected_len {
                    warnings.push(format!(
                        "row {} has {} fields; expected {}",
                        idx + 2,
                        record.len(),
                        expected_len
                    ));
                    continue;
                }

                rows.push(
                    record
                        .iter()
                        .map(|value| CellValue::String(value.to_string()))
                        .collect(),
                );
            }
            Err(error) => warnings.push(format!("row {} parse error: {error}", idx + 2)),
        }
    }

    Ok(Dataset {
        columns,
        rows,
        warnings,
        metadata: Default::default(),
    })
}

fn detect_delimiter(content: &str) -> u8 {
    let header = content.lines().next().unwrap_or_default();
    if header.matches('\t').count() >= header.matches(',').count() {
        b'\t'
    } else {
        b','
    }
}
