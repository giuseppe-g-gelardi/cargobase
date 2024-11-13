use super::rows::Row;
use super::tables::Table;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Database {
    name: String, // get name from command line args
    tables: Vec<Table>,
    rows: Vec<Row>,
    data: Vec<String>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            name: String::from("cargobase"),
            tables: Vec::new(),
            rows: Vec::new(),
            data: Vec::new(),
        }
    }
}
