use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Database, DatabaseError, Row, Table};

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

    pub fn where_eq<T: DeserializeOwned + Default>(
        self,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, String> {
        // Load the database
        let mut db = Database::load_from_file(&self.db_file_name)
            .map_err(|e| format!("Failed to load database: {}", e))?;

        // Clone table_name to avoid moving self
        let table_name = self
            .table_name
            .clone()
            .ok_or_else(|| "Table name not specified.".to_string())?;

        // Find the index of the table
        let table_index = db
            .tables
            .iter()
            .position(|t| t.name == table_name)
            .ok_or_else(|| format!("Table '{}' not found.", table_name))?;

        // Borrow the table by index
        let table = &mut db.tables[table_index];

        // consider thiserror for the error handling for these operations
        match self.operation {
            Operation::Read => self.execute_select(table, key, value),
            Operation::Update => {
                let result = self.execute_update(table, key, value);
                db.save_to_file()
                    .map_err(|e| format!("Failed to save database: {}", e))?;
                result
            }
            Operation::Delete => {
                let result = self.execute_delete(table, key, value);
                db.save_to_file()
                    .map_err(|e| format!("Failed to save database: {}", e))?;
                result
            }
            Operation::Create => unreachable!(),
        }
    }

    // pub fn execute_add(self) -> Result<(), String> {
    pub fn execute_add(self) -> Result<(), DatabaseError> {
        let mut db = Database::load_from_file(&self.db_file_name)
            .map_err(DatabaseError::LoadError)?;
            // .map_err(|e| format!("Failed to load database: {}", e))?;

        let table_name = self
            .table_name
            .clone()
            .ok_or_else(|| DatabaseError::InvalidData("Table name not specified.".to_string()))?;
            // .ok_or_else(|| "Table name not specified.".to_string())?;

        // Find the table
        let table = db
            .tables
            .iter_mut()
            .find(|t| t.name == table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.clone()))?;
            // .ok_or_else(|| format!("Table '{}' not found.", table_name))?;

        // Validate and add the row
        if let Some(row_data) = self.row_data {
            table.columns.validate(row_data.clone())?; // optional schema validation
            table.rows.push(Row::new(row_data));

            db.save_to_file().map_err(DatabaseError::SaveError)?;
                // .map_err(|e| format!("Failed to save database: {}", e))?;
            println!("Row added successfully to '{}'.", table_name);
            Ok(())
        } else {
            // Err("No data provided for the new row.".to_string())
            Err(DatabaseError::InvalidData("No data provided for the new row.".to_string()))
        }
    }

    fn execute_select<T: DeserializeOwned>(
        &self,
        table: &Table,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, String> {
        for row in &table.rows {
            if let Some(field_value) = row.data.get(key) {
                if field_value.as_str() == Some(value) {
                    return serde_json::from_value(row.data.clone())
                        .map(Some)
                        .map_err(|e| format!("Deserialization error: {}", e));
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
    ) -> Result<Option<T>, String> {
        for row in &mut table.rows {
            if let Some(field_value) = row.data.get(key) {
                if field_value.as_str() == Some(value) {
                    if let Some(update_data) = &self.update_data {
                        if let Value::Object(update_map) = update_data {
                            if let Value::Object(row_map) = &mut row.data {
                                for (k, v) in update_map {
                                    row_map.insert(k.clone(), v.clone());
                                }
                            } else {
                                return Err("Row data is not a JSON object.".to_string());
                            }

                            println!("Record updated successfully.");
                            return serde_json::from_value(row.data.clone())
                                .map(Some)
                                .map_err(|e| format!("Deserialization error: {}", e));
                        } else {
                            return Err("Invalid update data format.".to_string());
                        }
                    } else {
                        return Err("No update data provided.".to_string());
                    }
                }
            }
        }

        Ok(None) // No matching record found
    }
    fn execute_delete<T: DeserializeOwned>(
        &self,
        table: &mut Table,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, String> {
        for (i, row) in table.rows.iter().enumerate() {
            if let Some(field_value) = row.data.get(key) {
                if field_value.as_str() == Some(value) {
                    let record = serde_json::from_value(row.data.clone())
                        .map_err(|e| format!("Deserialization error: {}", e))?;

                    table.rows.remove(i);
                    println!("Record deleted successfully.");
                    return Ok(Some(record));
                }
            }
        }

        Ok(None) // No matching record found
    }
    pub fn all<T: DeserializeOwned>(&self) -> Vec<T> {
        let db = Database::load_from_file(&self.db_file_name).unwrap_or_else(|e| {
            eprintln!("Failed to load database from file: {}", e);
            Database {
                name: String::new(),
                file_name: self.db_file_name.clone(),
                tables: Vec::new(),
            }
        });

        if let Some(table_name) = &self.table_name {
            if let Some(table) = db.tables.iter().find(|t| t.name == *table_name) {
                table
                    .rows
                    .iter()
                    .filter_map(|row| serde_json::from_value(row.data.clone()).ok())
                    .collect()
            } else {
                eprintln!("Table {} not found", table_name);
                Vec::new()
            }
        } else {
            eprintln!("Table name not provided");
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

    use crate::cargobase::setup_temp_db;

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
    fn test_query_to() {
        let query = Query {
            db_file_name: "test_db.json".to_string(),
            table_name: None,
            operation: Operation::Create,
            update_data: None,
            row_data: None,
        };

        // let updated_query = query.to("TestTable");
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

    #[test]
    fn test_query_all() {
        let mut db = setup_temp_db();

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
            .expect("Failed to add row 1");

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data2.clone())
            .execute_add()
            .expect("Failed to add row 2");

        let rows: Vec<TestData> = db.get_rows().from("TestTable").all();

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
}
