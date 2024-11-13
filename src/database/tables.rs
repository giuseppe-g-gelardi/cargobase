use serde::{Deserialize, Serialize};
use super::rows::Row;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    name: String,
    rows: Vec<Row>,
    columns: Vec<String>,
}

impl Table {
    pub fn new(name: String, columns: Vec<String>) -> Self {
        Table {
            name,
            columns,
            rows: Vec::new(),
        }
    }
}
