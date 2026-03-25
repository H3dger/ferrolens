use std::path::Path;

use crate::data::types::{CellValue, Dataset};
use crate::error::Result;

pub fn write_visible_rows(
    dataset: &Dataset,
    row_indices: &[usize],
    visible_columns: &[String],
    output: &Path,
) -> Result<()> {
    let column_indices = dataset
        .columns
        .iter()
        .enumerate()
        .filter(|(_, column)| visible_columns.contains(&column.name))
        .collect::<Vec<_>>();

    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(output)?;

    writer.write_record(
        column_indices
            .iter()
            .map(|(_, column)| column.name.as_str()),
    )?;

    for row_index in row_indices {
        if let Some(row) = dataset.rows.get(*row_index) {
            let rendered = column_indices
                .iter()
                .filter_map(|(index, _)| row.get(*index))
                .map(render_cell)
                .collect::<Vec<_>>();
            writer.write_record(rendered)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn render_cell(cell: &CellValue) -> String {
    match cell {
        CellValue::String(value) => value.clone(),
        CellValue::Integer(value) => value.to_string(),
        CellValue::Float(value) => value.clone(),
        CellValue::Boolean(value) => value.to_string(),
        CellValue::Empty => String::new(),
    }
}
