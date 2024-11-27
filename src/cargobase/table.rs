use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Columns, Database, Row};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Table {
    pub(crate) name: String,
    pub rows: Vec<Row>,
    pub columns: Columns,
}

impl Table {
    pub fn new(name: String, columns: Columns) -> Self {
        Table {
            name,
            rows: Vec::new(),
            columns,
        }
    }

    // consider removing this. need to check what it is doing after removing the file name field
    // pub(crate) fn set_file_name(&mut self, file_name: String) {
    //     println!("File name set to: {}", file_name);
    // }

    pub fn add_row(&mut self, db: &mut Database, data: Value) {
        if let Some(table) = db.get_table_mut(&self.name) {
            if data.is_array() {
                if let Some(array) = data.as_array() {
                    for item in array {
                        table.rows.push(Row::new(item.clone()))
                    }
                }
            } else {
                table.rows.push(Row::new(data))
            }
            let _ = db.save_to_file().map_err(|e| {
                println!("Failed to save to file: {}", e);
            });
        } else {
            println!("Table {} not found", self.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Column;
    use super::*;

    #[test]
    fn test_table_new() {
        let columns = Columns::new(vec![Column::new("name", true), Column::new("age", false)]);
        let table = Table::new("users".to_string(), columns.clone());
        assert_eq!(table.name, "users");
        assert_eq!(table.columns, columns);
    }

    #[test]
    fn test_table_set_file_name() {
        let columns = Columns::new(vec![Column::new("name", true), Column::new("age", false)]);
        let table = Table::new("users".to_string(), columns.clone());
        // table.set_file_name("db.json".to_string());
        assert_eq!(table.name, "users");
    }
}
