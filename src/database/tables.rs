use super::rows::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub rows: Vec<Row>,
    pub columns: Vec<String>,
}

impl Table {
    pub fn new(name: String, columns: Vec<String>, rows: Vec<Row>) -> Self {
        Table {
            name,
            rows,
            columns,
        }
    }

    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }

    // pub fn add_row(&mut self, row: Row<T>) {
    //     self.rows.push(row);
    // }

    // pub fn get_row(&self, row_id: &str) -> Option<&Row<T>> {
    //     self.rows.iter().find(|row| row._id == row_id)
    // }
    //
    // pub fn remove_row(&mut self, row_id: &str) {
    //     self.rows.retain(|row| row._id != row_id);
    // }
}
