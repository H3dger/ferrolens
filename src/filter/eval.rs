use crate::data::types::{CellValue, Dataset};
use crate::filter::{ComparisonOp, FilterExpr};

pub fn matches_filter(dataset: &Dataset, row_index: usize, filter: &FilterExpr) -> bool {
    let Some(row) = dataset.rows.get(row_index) else {
        return false;
    };

    match filter {
        FilterExpr::Comparison { column, op, value } => {
            let Some(cell) = lookup_cell(dataset, row, column) else {
                return false;
            };

            match op {
                ComparisonOp::Eq => {
                    if let (Some(left), Some(right)) =
                        (cell_as_f64(cell), value.parse::<f64>().ok())
                    {
                        (left - right).abs() < f64::EPSILON
                    } else {
                        cell_as_string(cell) == *value
                    }
                }
                ComparisonOp::Lt => compare_numeric(cell, value, |left, right| left < right),
                ComparisonOp::Lte => compare_numeric(cell, value, |left, right| left <= right),
                ComparisonOp::Gt => compare_numeric(cell, value, |left, right| left > right),
                ComparisonOp::Gte => compare_numeric(cell, value, |left, right| left >= right),
            }
        }
        FilterExpr::InList { column, values } => {
            let Some(cell) = lookup_cell(dataset, row, column) else {
                return false;
            };
            let rendered = cell_as_string(cell);
            values.iter().any(|value| rendered == *value)
        }
    }
}

pub fn matches_search(dataset: &Dataset, row_index: usize, query: &str) -> bool {
    let Some(row) = dataset.rows.get(row_index) else {
        return false;
    };

    let query = query.to_ascii_lowercase();
    row.iter()
        .map(cell_as_string)
        .any(|value| value.to_ascii_lowercase().contains(&query))
}

fn lookup_cell<'a>(
    dataset: &'a Dataset,
    row: &'a [CellValue],
    column: &str,
) -> Option<&'a CellValue> {
    let index = dataset
        .columns
        .iter()
        .position(|entry| entry.name == column)?;
    row.get(index)
}

fn compare_numeric(cell: &CellValue, value: &str, cmp: impl Fn(f64, f64) -> bool) -> bool {
    let Some(left) = cell_as_f64(cell) else {
        return false;
    };
    let Some(right) = value.parse::<f64>().ok() else {
        return false;
    };
    cmp(left, right)
}

fn cell_as_f64(cell: &CellValue) -> Option<f64> {
    match cell {
        CellValue::String(value) => value.parse::<f64>().ok(),
        CellValue::Integer(value) => Some(*value as f64),
        CellValue::Float(value) => value.parse::<f64>().ok(),
        CellValue::Boolean(value) => Some(if *value { 1.0 } else { 0.0 }),
        CellValue::Empty => None,
    }
}

fn cell_as_string(cell: &CellValue) -> String {
    match cell {
        CellValue::String(value) => value.clone(),
        CellValue::Integer(value) => value.to_string(),
        CellValue::Float(value) => value.clone(),
        CellValue::Boolean(value) => value.to_string(),
        CellValue::Empty => String::new(),
    }
}
