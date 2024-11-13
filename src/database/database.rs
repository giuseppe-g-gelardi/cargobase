use super::tables::Table;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Database {
    name: String, // get name from command line args
    tables: Vec<Table>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            name: String::from("cargobase"),
            tables: Vec::new(),
        }
    }

    pub fn add_table(&mut self, table: Table) {
        self.tables.push(table);
    }
}
