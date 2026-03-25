use crate::data::types::{CellValue, ColumnDef, ColumnType, Dataset};
use crate::export::write_visible_rows;
use crate::filter::{matches_filter, matches_search, parse_filter, FilterExpr};
use crate::theme::ThemeName;
use crate::ui::{DetailField, RenderState};
use anyhow::Result;
#[cfg(test)]
use std::cell::Cell;
#[cfg(not(test))]
use std::cell::Cell as BuildCounterCell;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct App {
    dataset: Dataset,
    selected_row: Option<usize>,
    focused_column: Option<usize>,
    theme: ThemeName,
    filters: Vec<FilterExpr>,
    search_query: Option<String>,
    visible_row_indices: Vec<usize>,
    hidden_columns: HashSet<String>,
    sort_state: Option<(String, bool)>,
    horizontal_offset: usize,
    cached_visible_columns: RefCell<Option<Vec<String>>>,
    cached_table_rows: RefCell<Option<Arc<Vec<Vec<String>>>>>,
    table_rows_build_count: BuildCounter,
}

#[cfg(test)]
type BuildCounter = Cell<usize>;
#[cfg(not(test))]
type BuildCounter = BuildCounterCell<usize>;

impl App {
    pub fn new(dataset: Dataset) -> Self {
        Self::with_theme(dataset, ThemeName::Default)
    }

    pub fn with_theme(dataset: Dataset, theme: ThemeName) -> Self {
        let row_count = dataset.rows.len();
        let column_count = dataset.columns.len();
        let selected_row = if row_count == 0 { None } else { Some(0) };
        Self {
            dataset,
            selected_row,
            focused_column: if column_count == 0 { None } else { Some(0) },
            theme,
            filters: Vec::new(),
            search_query: None,
            visible_row_indices: (0..row_count).collect(),
            hidden_columns: HashSet::new(),
            sort_state: None,
            horizontal_offset: 0,
            cached_visible_columns: RefCell::new(None),
            cached_table_rows: RefCell::new(None),
            table_rows_build_count: BuildCounter::new(0),
        }
    }

    pub fn select_next(&mut self) {
        let Some(current) = self.selected_row else {
            return;
        };

        if current + 1 < self.visible_row_indices.len() {
            self.selected_row = Some(current + 1);
        }
    }

    pub fn select_previous(&mut self) {
        let Some(current) = self.selected_row else {
            return;
        };

        if current > 0 {
            self.selected_row = Some(current - 1);
        }
    }

    pub fn page_down(&mut self, page_size: usize) {
        self.move_selection_by(page_size as isize);
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.move_selection_by(-(page_size as isize));
    }

    pub fn scroll_left(&mut self) {
        self.horizontal_offset = self.horizontal_offset.saturating_sub(1);
    }

    pub fn scroll_right(&mut self) {
        let visible_columns = self.visible_columns();
        if self.horizontal_offset + 1 < visible_columns.len() {
            self.horizontal_offset += 1;
        }
    }

    pub fn focus_previous_column(&mut self) {
        let Some(current) = self.focused_column else {
            return;
        };

        if current > 0 {
            self.focused_column = Some(current - 1);
        }
    }

    pub fn focus_next_column(&mut self) {
        let Some(current) = self.focused_column else {
            return;
        };

        if current + 1 < self.visible_columns().len() {
            self.focused_column = Some(current + 1);
        }
    }

    pub fn selected_row(&self) -> Option<usize> {
        self.selected_row
    }

    pub fn current_row(&self) -> Option<&Vec<CellValue>> {
        self.selected_row
            .and_then(|visible_idx| self.visible_row_indices.get(visible_idx).copied())
            .and_then(|row_idx| self.dataset.rows.get(row_idx))
    }

    pub fn theme(&self) -> ThemeName {
        self.theme
    }

    pub fn focused_column_name(&self) -> Option<String> {
        self.focused_column
            .and_then(|index| self.visible_columns().get(index).cloned())
    }

    pub fn focused_filter_prefill(&self) -> Option<String> {
        let (_, column) = self.focused_visible_column()?;
        let operator = match column.column_type {
            ColumnType::Integer | ColumnType::Float => "<",
            ColumnType::Categorical => "in [",
            ColumnType::String | ColumnType::GenomicLocus | ColumnType::Boolean => "==",
        };

        Some(format!("{} {} ", column.name, operator))
    }

    pub fn focused_categorical_hint(&self) -> Option<String> {
        let (column_index, column) = self.focused_visible_column()?;
        if column.column_type != ColumnType::Categorical {
            return None;
        }

        let mut unique_values = Vec::new();
        for row in &self.dataset.rows {
            let value = row
                .get(column_index)
                .map(cell_as_string)
                .unwrap_or_default();
            if value.is_empty() || unique_values.contains(&value) {
                continue;
            }
            unique_values.push(value);
        }

        if unique_values.is_empty() {
            return None;
        }

        let sample = unique_values
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if unique_values.len() > 3 { ", …" } else { "" };
        Some(format!("Values: {sample}{suffix}"))
    }

    pub fn filter_summary(&self) -> Option<String> {
        if self.filters.is_empty() {
            return None;
        }

        Some(
            self.filters
                .iter()
                .map(format_filter_expr)
                .collect::<Vec<_>>()
                .join(", "),
        )
    }

    pub fn sort_summary(&self) -> Option<String> {
        self.sort_state.as_ref().map(|(column, ascending)| {
            format!("{column} {}", if *ascending { "asc" } else { "desc" })
        })
    }

    pub fn render_state(&self) -> RenderState {
        RenderState::for_app(
            self.theme,
            self.visible_columns(),
            self.table_rows(),
            self.selected_row,
            self.focused_column,
            self.horizontal_offset,
            self.detail_fields(),
            self.filtered_columns(),
            self.sort_state.clone(),
            self.status_message(),
        )
    }

    pub fn apply_filter_str(&mut self, input: &str) -> Result<()> {
        let filter = parse_filter(input)?;
        self.filters.push(filter);
        self.recompute_visible_rows();
        Ok(())
    }

    pub fn set_search_query(&mut self, query: impl Into<String>) {
        let query = query.into();
        self.search_query = if query.trim().is_empty() {
            None
        } else {
            Some(query)
        };
        self.recompute_visible_rows();
    }

    pub fn visible_row_count(&self) -> usize {
        self.visible_row_indices.len()
    }

    pub fn total_row_count(&self) -> usize {
        self.dataset.rows.len()
    }

    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }

    pub fn hidden_column_count(&self) -> usize {
        self.hidden_columns.len()
    }

    pub fn visible_rows(&self) -> Vec<&Vec<CellValue>> {
        self.visible_row_indices
            .iter()
            .filter_map(|index| self.dataset.rows.get(*index))
            .collect()
    }

    pub fn sort_by_column(&mut self, column: &str, ascending: bool) -> Result<()> {
        self.ensure_column_exists(column)?;
        self.sort_state = Some((column.to_string(), ascending));
        self.apply_sort();
        self.invalidate_table_cache();
        self.selected_row = if self.visible_row_indices.is_empty() {
            None
        } else {
            Some(0)
        };
        Ok(())
    }

    pub fn hide_column(&mut self, column: &str) -> Result<()> {
        self.ensure_column_exists(column)?;
        self.hidden_columns.insert(column.to_string());
        self.invalidate_column_cache();
        self.invalidate_table_cache();
        self.clamp_column_state();
        Ok(())
    }

    pub fn hide_current_visible_column(&mut self) -> Result<String> {
        let visible_columns = self.visible_columns();
        let Some(column) = self
            .focused_column
            .and_then(|index| visible_columns.get(index))
            .cloned()
        else {
            anyhow::bail!("no visible columns to hide")
        };

        self.hidden_columns.insert(column.clone());
        self.invalidate_column_cache();
        self.invalidate_table_cache();
        self.clamp_column_state();

        Ok(column)
    }

    pub fn sort_focused_column_toggle(&mut self) -> Result<(String, bool)> {
        let visible_columns = self.visible_columns();
        let Some(column) = self
            .focused_column
            .and_then(|index| visible_columns.get(index))
            .cloned()
        else {
            anyhow::bail!("no focused column to sort")
        };

        let ascending =
            !matches!(self.sort_state.as_ref(), Some((current, true)) if current == &column);
        self.sort_by_column(&column, ascending)?;
        Ok((column, ascending))
    }

    pub fn export_visible_rows(&self, output: &Path) -> Result<()> {
        write_visible_rows(
            &self.dataset,
            &self.visible_row_indices,
            &self.visible_columns(),
            output,
        )
    }

    pub fn reset_view(&mut self) {
        self.filters.clear();
        self.search_query = None;
        self.hidden_columns.clear();
        self.sort_state = None;
        self.horizontal_offset = 0;
        self.invalidate_column_cache();
        self.invalidate_table_cache();
        self.focused_column = if self.dataset.columns.is_empty() {
            None
        } else {
            Some(0)
        };
        self.visible_row_indices = (0..self.dataset.rows.len()).collect();
        self.selected_row = if self.visible_row_indices.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    fn recompute_visible_rows(&mut self) {
        self.invalidate_table_cache();
        self.visible_row_indices = self
            .dataset
            .rows
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                let filters_match = self
                    .filters
                    .iter()
                    .all(|filter| matches_filter(&self.dataset, *index, filter));
                let search_match = self
                    .search_query
                    .as_ref()
                    .map(|query| matches_search(&self.dataset, *index, query))
                    .unwrap_or(true);
                filters_match && search_match
            })
            .map(|(index, _)| index)
            .collect();

        self.apply_sort();

        self.selected_row = if self.visible_row_indices.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    fn apply_sort(&mut self) {
        let Some((column, ascending)) = self.sort_state.clone() else {
            return;
        };
        let Some(column_index) = self
            .dataset
            .columns
            .iter()
            .position(|entry| entry.name == column)
        else {
            return;
        };

        self.visible_row_indices.sort_by(|left, right| {
            let left_cell = self
                .dataset
                .rows
                .get(*left)
                .and_then(|row| row.get(column_index));
            let right_cell = self
                .dataset
                .rows
                .get(*right)
                .and_then(|row| row.get(column_index));

            let ordering = compare_cells(left_cell, right_cell);
            if ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
    }

    fn visible_columns(&self) -> Vec<String> {
        if let Some(columns) = self.cached_visible_columns.borrow().as_ref() {
            return columns.clone();
        }

        let columns = self
            .dataset
            .columns
            .iter()
            .filter(|column| !self.hidden_columns.contains(&column.name))
            .map(|column| column.name.clone())
            .collect::<Vec<_>>();
        *self.cached_visible_columns.borrow_mut() = Some(columns.clone());
        columns
    }

    fn table_rows(&self) -> Arc<Vec<Vec<String>>> {
        if let Some(rows) = self.cached_table_rows.borrow().as_ref() {
            return Arc::clone(rows);
        }

        self.table_rows_build_count
            .set(self.table_rows_build_count.get() + 1);

        let visible_column_indices = self
            .dataset
            .columns
            .iter()
            .enumerate()
            .filter(|(_, column)| !self.hidden_columns.contains(&column.name))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        let rows = self
            .visible_row_indices
            .iter()
            .filter_map(|row_index| self.dataset.rows.get(*row_index))
            .map(|row| {
                visible_column_indices
                    .iter()
                    .filter_map(|column_index| row.get(*column_index))
                    .map(cell_as_string)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let rows = Arc::new(rows);
        *self.cached_table_rows.borrow_mut() = Some(Arc::clone(&rows));
        rows
    }

    fn detail_fields(&self) -> Vec<DetailField> {
        let Some(row) = self.current_row() else {
            return Vec::new();
        };

        if self
            .dataset
            .metadata
            .get("dataset_kind")
            .map(|kind| kind == "vcf")
            .unwrap_or(false)
        {
            self.vcf_detail_fields(row)
        } else {
            self.generic_detail_fields(row)
        }
    }

    fn generic_detail_fields(&self, row: &[CellValue]) -> Vec<DetailField> {
        self.dataset
            .columns
            .iter()
            .zip(row.iter())
            .map(|(column, value)| DetailField {
                label: column.name.clone(),
                value: cell_as_string(value),
            })
            .collect()
    }

    fn filtered_columns(&self) -> Vec<String> {
        let mut columns = Vec::new();
        for filter in &self.filters {
            let name = match filter {
                FilterExpr::Comparison { column, .. } | FilterExpr::InList { column, .. } => column,
            };

            if !columns.contains(name) {
                columns.push(name.clone());
            }
        }

        columns
    }

    fn vcf_detail_fields(&self, row: &[CellValue]) -> Vec<DetailField> {
        let mut details = Vec::new();
        let mut used = HashSet::new();

        for label in ["CHROM", "POS", "ID", "REF", "ALT", "FILTER"] {
            if let Some(value) = self.lookup_value(row, label) {
                details.push(DetailField {
                    label: label.to_string(),
                    value,
                });
                used.insert(label.to_string());
            }
        }

        if let Some(info_raw) = self.lookup_value(row, "INFO") {
            let parsed = parse_info_map(&info_raw);
            for label in ["DP", "AF"] {
                if let Some(value) = parsed.get(label) {
                    details.push(DetailField {
                        label: label.to_string(),
                        value: value.clone(),
                    });
                    used.insert(label.to_string());
                }
            }

            details.push(DetailField {
                label: "INFO".to_string(),
                value: info_raw,
            });
            used.insert("INFO".to_string());

            details.push(DetailField {
                label: "INFO_RAW".to_string(),
                value: self.lookup_value(row, "INFO").unwrap_or_default(),
            });
            used.insert("INFO_RAW".to_string());
        }

        for (column, value) in self.dataset.columns.iter().zip(row.iter()) {
            if used.contains(&column.name) {
                continue;
            }
            details.push(DetailField {
                label: column.name.clone(),
                value: cell_as_string(value),
            });
        }

        details
    }

    fn lookup_value(&self, row: &[CellValue], column: &str) -> Option<String> {
        let index = self
            .dataset
            .columns
            .iter()
            .position(|entry| entry.name == column)?;
        row.get(index).map(cell_as_string)
    }

    fn ensure_column_exists(&self, column: &str) -> Result<()> {
        if self
            .dataset
            .columns
            .iter()
            .any(|entry| entry.name == column)
        {
            Ok(())
        } else {
            anyhow::bail!("unknown column: {column}")
        }
    }

    fn status_message(&self) -> String {
        if !self.dataset.warnings.is_empty() {
            return format!("{} warning(s) detected", self.dataset.warnings.len());
        }

        if self.visible_row_indices.is_empty() {
            if self.search_query.is_some() || !self.filters.is_empty() {
                return "No rows match current filters/search".to_string();
            }
        }

        if !self.hidden_columns.is_empty() {
            return format!("{} column(s) hidden", self.hidden_columns.len());
        }

        "Ready".to_string()
    }

    fn clamp_column_state(&mut self) {
        let visible_count = self.visible_columns().len();
        self.focused_column = match visible_count {
            0 => None,
            count => Some(self.focused_column.unwrap_or(0).min(count - 1)),
        };
        self.horizontal_offset = match visible_count {
            0 => 0,
            count => self.horizontal_offset.min(count - 1),
        };
    }

    fn invalidate_column_cache(&self) {
        *self.cached_visible_columns.borrow_mut() = None;
    }

    fn invalidate_table_cache(&self) {
        *self.cached_table_rows.borrow_mut() = None;
    }

    fn move_selection_by(&mut self, delta: isize) {
        let Some(current) = self.selected_row else {
            return;
        };

        let last_index = self.visible_row_indices.len().saturating_sub(1) as isize;
        let next = (current as isize + delta).clamp(0, last_index) as usize;
        self.selected_row = Some(next);
    }

    fn focused_visible_column(&self) -> Option<(usize, &ColumnDef)> {
        self.dataset
            .columns
            .iter()
            .enumerate()
            .filter(|(_, column)| !self.hidden_columns.contains(&column.name))
            .nth(self.focused_column?)
    }
}

impl App {
    #[allow(dead_code)]
    pub fn debug_table_rows_build_count(&self) -> usize {
        self.table_rows_build_count.get()
    }
}

fn compare_cells(left: Option<&CellValue>, right: Option<&CellValue>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => {
            if let (Some(left_num), Some(right_num)) = (cell_as_f64(left), cell_as_f64(right)) {
                left_num.partial_cmp(&right_num).unwrap_or(Ordering::Equal)
            } else {
                cell_as_string(left).cmp(&cell_as_string(right))
            }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
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

fn parse_info_map(info: &str) -> std::collections::HashMap<String, String> {
    info.split(';')
        .filter_map(|entry| entry.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn format_filter_expr(filter: &FilterExpr) -> String {
    match filter {
        FilterExpr::Comparison { column, op, value } => {
            format!("{column} {} {value}", comparison_symbol(*op))
        }
        FilterExpr::InList { column, values } => format!("{column} in [{}]", values.join(", ")),
    }
}

fn comparison_symbol(op: crate::filter::ComparisonOp) -> &'static str {
    match op {
        crate::filter::ComparisonOp::Eq => "==",
        crate::filter::ComparisonOp::Lt => "<",
        crate::filter::ComparisonOp::Lte => "<=",
        crate::filter::ComparisonOp::Gt => ">",
        crate::filter::ComparisonOp::Gte => ">=",
    }
}
