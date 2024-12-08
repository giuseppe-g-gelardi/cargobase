use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing;

use crate::Database;
use cargobase_core::{Columns, Row};

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
                tracing::error!("Failed to save to file: {}", e);
            });
        } else {
            tracing::error!("Table {} not found", self.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::setup_temp_db;
    use cargobase_core::{Column, Columns};

    use serde_json::json;
    use tracing_test::traced_test;

    #[test]
    fn test_table_new() {
        let columns = Columns::new(vec![Column::new("name", true), Column::new("age", false)]);
        let table = Table::new("users".to_string(), columns.clone());
        assert_eq!(table.name, "users");
        assert_eq!(table.columns, columns);
    }

    #[test]
    fn test_table_add_row_single() {
        let mut db = setup_temp_db();
        let mut table = Table::new(
            "TestTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );
        db.add_table(&mut table).unwrap();

        let row_data = json!({"id": "1", "name": "John Doe"});
        table.add_row(&mut db, row_data);

        assert_eq!(db.tables[0].rows.len(), 1);
        assert_eq!(
            db.tables[0].rows[0].data,
            json!({"id": "1", "name": "John Doe"})
        );
    }

    #[test]
    fn test_table_add_row_array() {
        let mut db = setup_temp_db();
        let mut table = Table::new(
            "TestTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );
        db.add_table(&mut table).unwrap();

        let row_data = json!([
            {"id": "1", "name": "John Doe"},
            {"id": "2", "name": "Jane Doe"}
        ]);
        table.add_row(&mut db, row_data);

        assert_eq!(db.tables[0].rows.len(), 2);
        assert_eq!(
            db.tables[0].rows[0].data,
            json!({"id": "1", "name": "John Doe"})
        );
        assert_eq!(
            db.tables[0].rows[1].data,
            json!({"id": "2", "name": "Jane Doe"})
        );
    }

    #[traced_test]
    #[test]
    fn test_table_add_row_table_not_found() {
        let mut db = setup_temp_db();
        let mut table = Table::new(
            "NonExistentTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );

        let row_data = json!({"id": "1", "name": "John Doe"});
        table.add_row(&mut db, row_data);

        assert!(logs_contain("Table NonExistentTable not found"));
        assert_eq!(db.tables.len(), 1); // Original table remains unchanged
    }

    #[traced_test]
    #[test]
    fn test_table_add_row_save_failure() {
        let mut db = setup_temp_db();
        let mut table = Table::new(
            "TestTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );
        db.add_table(&mut table).unwrap();

        // Simulate failure in saving
        db.file_name = "/invalid/path.json".to_string();
        let row_data = json!({"id": "1", "name": "John Doe"});
        table.add_row(&mut db, row_data);

        assert!(logs_contain("Failed to save to file"));
    }
}
