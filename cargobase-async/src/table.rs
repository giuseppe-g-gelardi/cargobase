use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing;

use crate::DatabaseAsync;
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

    pub async fn add_row_async(&mut self, db: &mut DatabaseAsync, data: Value) {
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
            match db.save_to_file_async().await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Failed to save to file: {}", e);
                }
            }
        } else {
            tracing::error!("Table {} not found", self.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::setup_temp_db_async;
    use cargobase_core::{Column, Columns};

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
    async fn test_table_add_row_single_async() {
        let mut db = setup_temp_db_async().await;
        let mut table = Table::new(
            "TestTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );
        db.add_table_async(&mut table).await.unwrap();

        let row_data = json!({"id": "1", "name": "John Doe"});
        table.add_row_async(&mut db, row_data).await;

        assert_eq!(db.tables[0].rows.len(), 1);
        assert_eq!(
            db.tables[0].rows[0].data,
            json!({"id": "1", "name": "John Doe"})
        );
    }

    #[tokio::test]
    async fn test_table_add_row_array_async() {
        let mut db = setup_temp_db_async().await;
        let mut table = Table::new(
            "TestTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );
        db.add_table_async(&mut table).await.unwrap();

        let row_data = json!([
            {"id": "1", "name": "John Doe"},
            {"id": "2", "name": "Jane Doe"}
        ]);
        table.add_row_async(&mut db, row_data).await;

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
    #[tokio::test]
    async fn test_table_add_row_table_now_found_async() {
        let mut db = setup_temp_db_async().await;
        let mut table = Table::new(
            "NonExistentTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );

        let row_data = json!({"id": "1", "name": "John Doe"});
        table.add_row_async(&mut db, row_data).await;

        assert!(logs_contain("Table NonExistentTable not found"));
        assert_eq!(db.tables.len(), 1); // Original table remains unchanged
    }

    #[traced_test]
    #[tokio::test]
    async fn test_table_add_row_save_failure_async() {
        let mut db = setup_temp_db_async().await;
        let mut table = Table::new(
            "TestTable".to_string(),
            Columns::new(vec![Column::new("id", true), Column::new("name", true)]),
        );
        db.add_table_async(&mut table).await.unwrap();

        // Simulate failure in saving
        db.file_name = "/invalid/path.json".to_string();
        let row_data = json!({"id": "1", "name": "John Doe"});
        table.add_row_async(&mut db, row_data).await;

        assert!(logs_contain("Failed to save to file"));
    }
}
