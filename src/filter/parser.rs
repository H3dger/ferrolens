use anyhow::{anyhow, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum FilterExpr {
    Comparison {
        column: String,
        op: ComparisonOp,
        value: String,
    },
    InList {
        column: String,
        values: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparisonOp {
    Eq,
    Lt,
    Lte,
    Gt,
    Gte,
}

pub fn parse_filter(input: &str) -> Result<FilterExpr> {
    let trimmed = input.trim();

    if let Some((column, rest)) = trimmed.split_once(" in ") {
        let values = rest
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();

        if values.is_empty() {
            return Err(anyhow!("filter list cannot be empty"));
        }

        return Ok(FilterExpr::InList {
            column: column.trim().to_string(),
            values,
        });
    }

    for (symbol, op) in [
        ("<=", ComparisonOp::Lte),
        (">=", ComparisonOp::Gte),
        ("==", ComparisonOp::Eq),
        ("<", ComparisonOp::Lt),
        (">", ComparisonOp::Gt),
    ] {
        if let Some((column, value)) = trimmed.split_once(symbol) {
            return Ok(FilterExpr::Comparison {
                column: column.trim().to_string(),
                op,
                value: value.trim().to_string(),
            });
        }
    }

    Err(anyhow!("unsupported filter expression: {trimmed}"))
}
