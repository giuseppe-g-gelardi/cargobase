use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing;

use crate::{Columns, Database, Row};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Table {
    pub(crate) name: String,
    pub rows: HashMap<String, Row>, // Row ID -> Row
    pub columns: Columns,
}

impl Table {
    pub fn new(name: String, columns: Columns) -> Self {
        Table {
            name,
            rows: HashMap::new(),
            columns,
        }
    }

    pub async fn add_row(&mut self, db: &mut Database, data: Value) {
        if let Some(table) = db.get_table_mut(&self.name) {
            match table.process_data(data) {
                Ok(_) => {
                    if let Err(e) = db.save_to_file().await {
                        tracing::error!("Failed to save to file: {}", e);
                    }
                }
                Err(err) => {
                    tracing::error!("Error adding row(s): {}", err);
                }
            }
        } else {
            tracing::error!("Table {} not found", self.name);
        }
    }

    fn process_data(&mut self, data: Value) -> Result<(), String> {
        if let Some(array) = data.as_array() {
            self.add_multiple_rows(array)?;
        } else {
            self.add_single_row(data)?;
        }
        Ok(())
    }

    /*
     * update this....
     * automatucally convert to Value
     * let _ = serde_json::to_value(&data).unwrap();
     */
    fn add_multiple_rows(&mut self, rows: &[Value]) -> Result<(), String> {
        for row in rows {
            self.add_single_row(row.clone())?;
        }
        Ok(())
    }

    fn add_single_row(&mut self, row: Value) -> Result<(), String> {
        if let Some(row_id) = row.get("id").and_then(Value::as_str) {
            if self.rows.contains_key(row_id) {
                return Err(format!("Row with id '{}' already exists", row_id));
            }
            self.rows.insert(row_id.to_string(), Row::new(row.clone()));
            Ok(())
        } else {
            Err(format!("Row is missing an 'id' field: {:?}", row))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{setup_temp_db, Column, Columns};

    use super::*;

    use serde_json::json;
    use tracing_test::traced_test;

    #[test]
    fn test_table_new() {
        let columns = Columns::new(vec![Column::new("name", true), Column::new("age", false)]);
        let table = Table::new("users".to_string(), columns.clone());
        assert_eq!(table.name, "users");
        assert_eq!(table.columns, columns);
    }

    #[tokio::test]
    async fn test_table_add_row_single() {
        let mut db = setup_temp_db().await;
        let mut table = Table::new(
            "TestTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );
        db.add_table(&mut table).await.unwrap();

        let row_data = json!({"id": "1", "name": "John Doe"});
        table.add_row(&mut db, row_data).await;

        assert_eq!(db.tables.get("TestTable").unwrap().rows.len(), 1);
        assert_eq!(
            db.tables
                .get("TestTable")
                .unwrap()
                .rows
                .get("1")
                .unwrap()
                .data,
            json!({"id": "1", "name": "John Doe"})
        );
    }

    #[tokio::test]
    async fn test_table_add_row_array() {
        let mut db = setup_temp_db().await;
        let mut table = Table::new(
            "TestTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );
        db.add_table(&mut table).await.unwrap();

        let row_data = json!([
            {"id": "1", "name": "John Doe"},
            {"id": "2", "name": "Jane Doe"}
        ]);
        table.add_row(&mut db, row_data).await;

        println!(
            "test tables: {:?}",
            db.tables.get("TestTable").unwrap().rows
        );
        println!("db.tables: {:?}", db.tables);

        assert_eq!(db.tables.get("TestTable").unwrap().rows.len(), 2);
        assert_eq!(
            db.tables
                .get("TestTable")
                .unwrap()
                .rows
                .get("1")
                .unwrap()
                .data,
            json!({"id": "1", "name": "John Doe"})
        );
        assert_eq!(
            db.tables
                .get("TestTable")
                .unwrap()
                .rows
                .get("2")
                .unwrap()
                .data,
            json!({"id": "2", "name": "Jane Doe"})
        );
    }

    #[traced_test]
    #[tokio::test]
    async fn test_table_add_row_table_now_found() {
        let mut db = setup_temp_db().await;
        let mut table = Table::new(
            "NonExistentTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );

        let row_data = json!({"id": "1", "name": "John Doe"});
        table.add_row(&mut db, row_data).await;

        assert!(logs_contain("Table NonExistentTable not found"));
        assert_eq!(db.tables.len(), 1); // Original table remains unchanged
    }

    #[traced_test]
    #[tokio::test]
    async fn test_table_add_row_save_failure() {
        let mut db = setup_temp_db().await;
        let mut table = Table::new(
            "TestTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );
        db.add_table(&mut table).await.unwrap();

        // Simulate failure in saving
        db.file_name = "/invalid/path.json".into();

        let row_data = json!({"id": "1", "name": "John Doe"});
        table.add_row(&mut db, row_data).await;

        assert!(logs_contain("Failed to save to file"));
    }
}
