use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Columns, Database, Row};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Table {
    pub name: String,
    pub rows: Vec<Row>,
    pub columns: Columns,
    pub file_name: Option<String>, // reference to the db file_name
}

impl Table {
    pub fn new(name: String, columns: Columns) -> Self {
        Table {
            name,
            rows: Vec::new(),
            columns,
            file_name: None,
        }
    }

    pub fn set_file_name(&mut self, file_name: String) {
        self.file_name = Some(file_name);
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
                println!("Failed to save to file: {}", e);
            });
        } else {
            println!("Table {} not found", self.name);
        }
    }

    fn validate_row(&self, data: &Value) -> Result<(), String> {
        if let Some(obj) = data.as_object() {
            for column in &self.columns.0 {
                if column.required && !obj.contains_key(&column.name) {
                    return Err(format!("Missing required column: {}", column.name));
                }
            }

            for key in obj.keys() {
                if !self.columns.0.iter().any(|col| col.name == *key) {
                    return Err(format!("Invalid column name: {}", key));
                }
            }
            Ok(())
        } else {
            Err("Invalid data format: expected a JSON object.".to_string())
        }
    }
}

// impl Table {
//     /// Add a new column to the table schema and update existing rows
//     pub fn add_column(&mut self, column: Column, db: &mut Database) -> Result<(), String> {
//         // Check if the column already exists
//         if self.columns.0.iter().any(|col| col.name == column.name) {
//             return Err(format!("Column '{}' already exists in table '{}'.", column.name, self.name));
//         }
//
//         // Add the new column to the schema
//         self.columns.0.push(column.clone());
//         println!("Column '{}' added to table '{}'.", column.name, self.name);
//
//         // Update existing rows by adding the new column with an empty value
//         for row in &mut self.rows {
//             if !row.data.as_object().unwrap().contains_key(&column.name) {
//                 row.data[&column.name] = serde_json::Value::Null; // Default to `null` or `Value::String("")`
//             }
//         }
//
//         // Save the updated table back to the database file
//         if let Some(file_name) = &self.file_name {
//             db.save_to_file().map_err(|e| format!("Failed to save table: {}", e))?;
//         }
//
//         Ok(())
//     }
// }
//
