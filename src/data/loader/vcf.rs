use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::Path;

use flate2::read::MultiGzDecoder;

use crate::data::types::{CellValue, ColumnDef, ColumnType, Dataset};
use crate::error::Result;

pub fn load_dataset(path: &Path) -> Result<Dataset> {
    let content = read_content(path)?;
    let mut columns = Vec::new();
    let mut rows = Vec::new();
    let mut info_keys = HashSet::new();

    for line in content.lines() {
        if line.starts_with("##") {
            continue;
        }

        if let Some(header) = line.strip_prefix('#') {
            columns = header
                .split('\t')
                .map(|name| {
                    let column_type = if name == "POS" {
                        ColumnType::GenomicLocus
                    } else {
                        ColumnType::String
                    };
                    ColumnDef::new(name, column_type)
                })
                .collect();
            continue;
        }

        if line.trim().is_empty() {
            continue;
        }

        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() >= 8 {
            for entry in fields[7].split(';') {
                if let Some((key, _)) = entry.split_once('=') {
                    info_keys.insert(key.to_string());
                }
            }
        }

        rows.push(
            fields
                .into_iter()
                .map(|value| CellValue::String(value.to_string()))
                .collect(),
        );
    }

    let mut dataset = Dataset::new(columns, rows);
    dataset
        .metadata
        .insert("dataset_kind".to_string(), "vcf".to_string());
    if !info_keys.is_empty() {
        let mut keys = info_keys.into_iter().collect::<Vec<_>>();
        keys.sort();
        dataset
            .metadata
            .insert("info_keys".to_string(), keys.join(","));
    }
    Ok(dataset)
}

fn read_content(path: &Path) -> Result<String> {
    if path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
        let file = fs::File::open(path)?;
        let mut decoder = MultiGzDecoder::new(file);
        let mut content = String::new();
        decoder.read_to_string(&mut content)?;
        Ok(content)
    } else {
        Ok(fs::read_to_string(path)?)
    }
}
