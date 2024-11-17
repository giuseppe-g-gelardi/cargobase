use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::Database;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Query {
    pub db_file_name: String,
    pub table_name: Option<String>,
    pub delete: bool,
}

impl Query {
    pub fn from(mut self, table_name: &str) -> Self {
        self.table_name = Some(table_name.to_string());
        self
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

    /// Fetch or delete a single row by a specific key-value pair
    pub fn where_eq<T: DeserializeOwned>(self, key: &str, value: &str) -> Result<T, String> {
        // Load the latest state of the database from the file
        let mut db = Database::load_from_file(&self.db_file_name)
            .map_err(|e| format!("Failed to load database from file: {}", e))?;

        if let Some(table_name) = &self.table_name {
            if let Some(table) = db.tables.iter_mut().find(|t| t.name.as_str() == table_name) {
                for i in 0..table.rows.len() {
                    let row = &table.rows[i];
                    if let Some(field_value) = row.data.get(key) {
                        if field_value.as_str() == Some(value) {
                            // Deserialize the matching record
                            let record: T = serde_json::from_value(row.data.clone())
                                .map_err(|e| format!("Deserialization error: {}", e))?;

                            // Check if the operation is "delete"
                            if self.delete {
                                table.rows.remove(i);
                                db.save_to_file()
                                    .map_err(|e| format!("Failed to save database: {}", e))?;
                                println!("Deleted record from table '{}'.", table_name);
                            }

                            // Return the found or deleted record
                            return Ok(record);
                        }
                    }
                }
                Err(format!(
                    "No matching record found where '{}' == '{}'.",
                    key, value
                ))
            } else {
                Err(format!("Table '{}' not found.", table_name))
            }
        } else {
            Err("Table name not specified.".to_string())
        }
    }
}
