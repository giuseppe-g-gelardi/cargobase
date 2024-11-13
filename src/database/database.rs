use super::tables::Table;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Database {
    pub name: String, // get name from command line args
    pub tables: Vec<Table>,
}

impl Database {
    pub fn new(name: String) -> Self {
        Database {
            name,
            tables: Vec::new(),
        }
    }

    pub fn add_table(&mut self, table: Table) {
        self.tables.push(table);
    }

    // pub fn get_table(&self, table_name: &str) -> Option<&Table<T>> {
    //     self.tables.iter().find(|table| table.name == table_name)
    // }
    //
    // pub fn get_table_mut(&mut self, table_name: &str) -> Option<&mut Table<T>> {
    //     self.tables
    //         .iter_mut()
    //         .find(|table| table.name == table_name)
    // }
    //
    // pub fn remove_table(&mut self, table_name: &str) {
    //     self.tables.retain(|table| table.name != table_name);
    // }
}
