use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dataset {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Vec<CellValue>>,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Dataset {
    pub fn new(columns: Vec<ColumnDef>, rows: Vec<Vec<CellValue>>) -> Self {
        Self {
            columns,
            rows,
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: ColumnType,
    pub visible: bool,
    pub frozen: bool,
}

impl ColumnDef {
    pub fn new(name: impl Into<String>, column_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            column_type,
            visible: true,
            frozen: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColumnType {
    String,
    Integer,
    Float,
    Boolean,
    Categorical,
    GenomicLocus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellValue {
    String(String),
    Integer(i64),
    Float(String),
    Boolean(bool),
    Empty,
}
