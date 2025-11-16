pub mod query;
pub mod operators;

pub use query as query_operations;
pub use operators::{ComparisonOp, Condition, ConditionGroup, LogicalOp, SortOrder};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum Operation {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Query {
    pub db_file_name: PathBuf,
    pub table_name: Option<String>,
    pub operation: Operation,
    pub update_data: Option<Value>,
    pub row_data: Option<Value>,
    pub conditions: Vec<ConditionGroup>,
    pub order_by: Vec<(String, SortOrder)>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
