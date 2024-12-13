use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Database, DatabaseError, Row, Table};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum Operation {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Query {
    pub db_file_name: String,
    pub table_name: Option<String>,
    pub operation: Operation,
    pub update_data: Option<Value>,
    pub row_data: Option<Value>,
}

impl Query {
    pub fn from(mut self, table_name: &str) -> Self {
        self.table_name = Some(table_name.to_string());
        self
    }

    pub fn data(mut self, data: Value) -> Self {
        self.update_data = Some(data);
        self
    }

    pub fn data_from_struct<T: Serialize>(mut self, data: T) -> Self {
        self.row_data = Some(serde_json::to_value(data).expect("Failed to serialize data"));
        self
    }

    pub async fn where_eq<T: DeserializeOwned + Default>(
        self,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, DatabaseError> {
        let mut db = Database::load_from_file(&self.db_file_name)
            .await
            .map_err(DatabaseError::LoadError)?;
        self.handle_where_eq(&mut db, key, value).await // Shared logic
    }

    // Shared logic for where_eq
    async fn handle_where_eq<T: DeserializeOwned + Default>(
        &self,
        db: &mut Database,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, DatabaseError> {
        let table_name = self
            .table_name
            .clone()
            .ok_or_else(|| DatabaseError::TableNotFound("Table name not specified.".to_string()))?;

        let table_index = db
            .tables
            .iter()
            .position(|t| t.name == table_name)
            .ok_or_else(|| {
                DatabaseError::TableNotFound(format!("Table '{}' not found.", table_name))
            })?;

        let table = &mut db.tables[table_index];

        match self.operation {
            Operation::Read => self.execute_select(table, key, value),
            Operation::Update => {
                let result = self.execute_update(table, key, value);
                db.save_to_file().await.map_err(DatabaseError::SaveError)?;
                result
            }
            Operation::Delete => {
                let result = self.execute_delete(table, key, value);
                db.save_to_file().await.map_err(DatabaseError::SaveError)?;
                result
            }
            Operation::Create => unreachable!(),
        }
    }

    // #[cfg(feature = "async")]
    // pub async fn execute_add(self) -> Result<(), DatabaseError> {
    pub async fn execute_add(self) -> Result<(), DatabaseError> {
        let mut db = Database::load_from_file(&self.db_file_name)
            .await
            .map_err(DatabaseError::LoadError)?;
        self.handle_execute_add_sync(&mut db).await // Shared logic
    }

    async fn handle_execute_add_sync(&self, db: &mut Database) -> Result<(), DatabaseError> {
        let table_name = self
            .table_name
            .clone()
            .ok_or_else(|| DatabaseError::InvalidData("Table name not specified.".to_string()))?;

        let table = db
            .tables
            .iter_mut()
            .find(|t| t.name == table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.clone()))?;

        if let Some(row_data) = self.row_data.clone() {
            table.columns.validate(row_data.clone())?;
            table.rows.push(Row::new(row_data));
            db.save_to_file().await.map_err(DatabaseError::SaveError)?;
            Ok(())
        } else {
            Err(DatabaseError::InvalidData(
                "No data provided for the new row.".to_string(),
            ))
        }
    }

    fn execute_select<T: DeserializeOwned>(
        &self,
        table: &Table,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, DatabaseError> {
        for row in &table.rows {
            if let Some(field_value) = row.data.get(key) {
                if field_value.as_str() == Some(value) {
                    return serde_json::from_value(row.data.clone())
                        .map(Some)
                        .map_err(|e| {
                            DatabaseError::InvalidData(format!("Deserialization error: {}", e))
                        });
                }
            }
        }
        Ok(None) // No matching record found
    }

    fn execute_update<T: DeserializeOwned>(
        &self,
        table: &mut Table,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, DatabaseError> {
        // Find the row that matches the key-value pair
        if let Some(row) = self.find_matching_row(table, key, value)? {
            // Update the row with the provided data
            self.apply_update_to_row(row, &self.update_data)?;

            // Log and return the updated row
            tracing::info!("Record updated successfully.");
            return self.deserialize_row(row);
        }

        Ok(None) // No matching record found
    }

    // Helper: Find the matching row based on key-value pair
    fn find_matching_row<'a>(
        &self,
        table: &'a mut Table,
        key: &str,
        value: &str,
    ) -> Result<Option<&'a mut Row>, DatabaseError> {
        for row in &mut table.rows {
            if let Some(field_value) = row.data.get(key) {
                if field_value.as_str() == Some(value) {
                    return Ok(Some(row));
                }
            }
        }
        Ok(None)
    }

    // Helper: Apply the update data to the row
    fn apply_update_to_row(
        &self,
        row: &mut Row,
        update_data: &Option<Value>,
    ) -> Result<(), DatabaseError> {
        let update_map = match update_data {
            Some(Value::Object(map)) => map,
            Some(_) => {
                return Err(DatabaseError::InvalidData(
                    "Invalid update data format.".to_string(),
                ))
            }
            None => {
                return Err(DatabaseError::InvalidData(
                    "No update data provided.".to_string(),
                ))
            }
        };

        let row_map = row.data.as_object_mut().ok_or_else(|| {
            DatabaseError::InvalidData("Row data is not a JSON object.".to_string())
        })?;

        for (k, v) in update_map {
            row_map.insert(k.clone(), v.clone());
        }

        Ok(())
    }

    // Helper: Deserialize the updated row
    fn deserialize_row<T: DeserializeOwned>(&self, row: &Row) -> Result<Option<T>, DatabaseError> {
        serde_json::from_value(row.data.clone())
            .map(Some)
            .map_err(|e| DatabaseError::InvalidData(format!("Deserialization error: {}", e)))
    }

    fn execute_delete<T: DeserializeOwned>(
        &self,
        table: &mut Table,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, DatabaseError> {
        for (i, row) in table.rows.iter().enumerate() {
            if let Some(field_value) = row.data.get(key) {
                if field_value.as_str() == Some(value) {
                    let record = serde_json::from_value(row.data.clone())
                        .map_err(|e| DatabaseError::JSONError(e))?;

                    table.rows.remove(i);
                    tracing::info!("Record deleted successfully.");
                    return Ok(Some(record));
                }
            }
        }

        Ok(None) // No matching record found
    }

    pub async fn all<T: DeserializeOwned>(&self) -> Vec<T> {
        let db = Database::load_from_file(&self.db_file_name)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("Failed to load database from file: {}", e);
                Database {
                    name: String::new(),
                    file_name: self.db_file_name.clone(),
                    tables: Vec::new(),
                }
            });
        self.handle_all(&db) // Shared logic
    }

    fn handle_all<T: DeserializeOwned>(&self, db: &Database) -> Vec<T> {
        if let Some(table_name) = &self.table_name {
            if let Some(table) = db.tables.iter().find(|t| t.name == *table_name) {
                table
                    .rows
                    .iter()
                    .filter_map(|row| serde_json::from_value(row.data.clone()).ok())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    pub fn set(mut self, update_data: Value) -> Self {
        self.update_data = Some(update_data);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    use crate::setup_temp_db;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
    struct TestData {
        id: String,
        name: String,
    }

    #[test]
    fn test_query_from() {
        let query = Query {
            db_file_name: "test_db.json".to_string(),
            table_name: None,
            operation: Operation::Read,
            update_data: None,
            row_data: None,
        };

        let updated_query = query.from("TestTable");
        assert_eq!(updated_query.table_name, Some("TestTable".to_string()));
    }

    #[test]
    fn test_query_data() {
        let query = Query {
            db_file_name: "test_db.json".to_string(),
            table_name: Some("TestTable".to_string()),
            operation: Operation::Update,
            update_data: None,
            row_data: None,
        };

        let data = json!({ "name": "Updated Name" });
        let updated_query = query.data(data.clone());
        assert_eq!(updated_query.update_data, Some(data));
        std::fs::remove_file("test_db.json").ok();
    }

    #[test]
    fn test_query_data_from_struct() {
        std::fs::remove_file("test_db.json").ok();
        let query = Query {
            db_file_name: "test_db.json".to_string(),
            table_name: Some("TestTable".to_string()),
            operation: Operation::Create,
            update_data: None,
            row_data: None,
        };

        let test_data = TestData {
            id: "123".to_string(),
            name: "John Doe".to_string(),
        };

        let updated_query = query.data_from_struct(test_data.clone());
        let expected_data = serde_json::to_value(test_data).unwrap();

        assert_eq!(updated_query.row_data, Some(expected_data));
    }

    #[tokio::test]
    async fn test_query_all() {
        let mut db = setup_temp_db().await;

        let test_data1 = TestData {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };
        let test_data2 = TestData {
            id: "2".to_string(),
            name: "Bob".to_string(),
        };

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data1.clone())
            .execute_add()
            .await
            .expect("Failed to add row 1");

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data2.clone())
            .execute_add()
            .await
            .expect("Failed to add row 2");

        let rows: Vec<TestData> = db.get_rows().from("TestTable").all().await;

        assert_eq!(rows.len(), 2);
        assert!(rows.contains(&test_data1));
        assert!(rows.contains(&test_data2));

        // No explicit cleanup needed; tempfile will handle it automatically
    }

    #[test]
    fn test_query_set() {
        let query = Query {
            db_file_name: "test_db.json".to_string(),
            table_name: Some("TestTable".to_string()),
            operation: Operation::Update,
            update_data: None,
            row_data: None,
        };

        let data = json!({ "name": "Updated Name" });
        let updated_query = query.set(data.clone());
        assert_eq!(updated_query.update_data, Some(data));
    }

    #[tokio::test]
    async fn test_query_where_eq_no_match() {
        let mut db = setup_temp_db().await;

        let test_data = TestData {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data)
            .execute_add()
            .await
            .expect("Failed to add row");

        let result: Option<TestData> = db
            .get_rows()
            .from("TestTable")
            .where_eq("id", "999")
            .await
            .unwrap();
        assert!(result.is_none(), "Expected no matching record");
    }

    #[tokio::test]
    async fn test_query_where_eq_match() {
        let mut db = setup_temp_db().await;

        let test_data = TestData {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data.clone())
            .execute_add()
            .await
            .expect("Failed to add row");

        let result: Option<TestData> = db
            .get_rows()
            .from("TestTable")
            .where_eq("id", "1")
            .await
            .unwrap();
        assert_eq!(result, Some(test_data), "Expected matching record");
    }

    #[tokio::test]
    async fn test_query_execute_update() {
        let mut db = setup_temp_db().await;

        let test_data = TestData {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data.clone())
            .execute_add()
            .await
            .expect("Failed to add row");

        let update_data = json!({ "name": "Updated Alice" });

        let result = db
            .update_row()
            .from("TestTable")
            .data(update_data)
            .where_eq::<TestData>("id", "1")
            .await
            .unwrap();

        assert!(
            result.is_some(),
            "Expected update to return the updated record"
        );
        assert_eq!(
            result.unwrap().name,
            "Updated Alice",
            "Name was not updated"
        );
    }

    #[tokio::test]
    async fn test_query_execute_delete() {
        let mut db = setup_temp_db().await;

        let test_data = TestData {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data.clone())
            .execute_add()
            .await
            .expect("Failed to add row");

        let result = db
            .get_single()
            .from("TestTable")
            .where_eq::<TestData>("id", "1")
            .await
            .unwrap();

        assert!(result.is_some(), "Expected record to exist before deletion");

        let _ = db
            .delete_single()
            .from("TestTable")
            .where_eq::<TestData>("id", "1")
            .await
            .unwrap();

        let rows: Vec<TestData> = db.get_rows().from("TestTable").all().await;
        let deleted_record = db
            .get_single()
            .from("TestTable")
            .where_eq::<TestData>("id", "1")
            .await
            .unwrap();

        assert!(deleted_record.is_none(), "Expected record to be deleted");
        assert!(rows.is_empty(), "Expected all records to be deleted");
    }
}
