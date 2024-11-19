use super::{Query, query::Operation, Table};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Database {
    pub(crate) name: String,
    pub(crate) file_name: String,
    pub(crate) tables: Vec<Table>,
}

impl Database {
    pub fn new(name: &str) -> Self {
        let name = name.to_string();
        let file_name = format!("{}.json", name);

        if std::path::Path::new(&file_name).exists() {
            return Database::load_from_file(&file_name).unwrap();
        } else {
            println!("Creating new database: {}", file_name);

            if let Err(e) = std::fs::write(&file_name, "{}") {
                eprintln!("Failed to create database file: {}", e);
            }
        }

        Database {
            name,
            file_name,
            tables: Vec::new(),
        }
    }

    pub fn add_table(&mut self, table: &mut Table) -> Result<(), String> {
        table.set_file_name(self.file_name.clone());
        if self.tables.iter().any(|t| t.name == table.name) {
            eprintln!("Table {} already exists, Skipping creation.", table.name);
            Ok(())
        } else {
            self.tables.push(table.clone());
            Ok(())
        }
    }

    pub fn drop_table(&mut self, table_name: &str) -> Result<(), String> {
        let mut db = Database::load_from_file(&self.file_name)
            .map_err(|e| format!("Failed to laod database from file: {:?}", e))?;

        if let Some(index) = db.tables.iter().position(|t| t.name == table_name) {
            let removed_table = db.tables.remove(index);
            println!("Table {} dropped successfully", removed_table.name);
            db.save_to_file()
                .map_err(|e| format!("Failed to save database: {:?}", e))?;

            self.tables = db.tables;
            Ok(())
        } else {
            Err(format!("Table {} not found", table_name))
        }
    }

    pub(crate) fn save_to_file(&self) -> Result<(), std::io::Error> {
        let json_data = serde_json::to_string_pretty(&self)?;
        std::fs::write(&self.file_name, json_data)?;
        println!("Database saved to file: {}", self.file_name);
        Ok(())
    }

    pub(crate) fn load_from_file(file_name: &str) -> Result<Self, std::io::Error> {
        let json_data = std::fs::read_to_string(file_name)?;
        let db: Database = serde_json::from_str(&json_data)?;
        Ok(db)
    }

    pub(crate) fn get_table_mut(&mut self, table_name: &str) -> Option<&mut Table> {
        self.tables.iter_mut().find(|t| t.name == table_name)
    }

    pub fn get_rows(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Select,
        }
    }

    pub fn get_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Select,
        }
    }

    pub fn delete_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Delete,
        }
    }

    pub fn view(&self) {
        println!("Database: {}", self.name);

        for table in &self.tables {
            println!("\nTable: {}", table.name);

            if table.columns.0.is_empty() {
                println!("No columns defined for table '{}'.", table.name);
                continue;
            }

            // Get column names and determine maximum width for each column
            let column_names: Vec<&str> = table
                .columns
                .0
                .iter()
                .map(|col| col.name.as_str())
                .collect();
            let mut column_widths: Vec<usize> =
                column_names.iter().map(|name| name.len()).collect();

            // Adjust column widths based on the content of each row
            for row in &table.rows {
                for (i, column) in table.columns.0.iter().enumerate() {
                    let value = row
                        .data
                        .get(&column.name)
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string();
                    column_widths[i] = column_widths[i].max(value.len());
                }
            }

            // Print the header row
            let header: Vec<String> = column_names
                .iter()
                .enumerate()
                .map(|(i, &name)| format!("{:<width$}", name, width = column_widths[i]))
                .collect();
            println!("{}", header.join(" | "));

            // Print a separator line
            let separator: Vec<String> = column_widths
                .iter()
                .map(|&width| "-".repeat(width))
                .collect();
            println!("{}", separator.join("-+-"));

            // Print each row of data
            for row in &table.rows {
                let row_data: Vec<String> = table
                    .columns
                    .0
                    .iter()
                    .enumerate()
                    .map(|(i, column)| {
                        let value = row
                            .data
                            .get(&column.name)
                            .unwrap_or(&serde_json::Value::Null)
                            .to_string();
                        format!("{:<width$}", value, width = column_widths[i])
                    })
                    .collect();
                println!("{}", row_data.join(" | "));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_database_new() {
        let file_name = "test.json";
        if std::path::Path::new(file_name).exists() {
            fs::remove_file(file_name).expect("Failed to remove test file");
        }

        let db = Database::new("test");
        assert_eq!(db.name, "test");
        assert_eq!(db.file_name, "test.json");
        assert_eq!(db.tables.len(), 0);

        if std::path::Path::new(file_name).exists() {
            fs::remove_file(file_name).expect("Failed to remove test file");
        }
    }

    use super::super::{Column, Columns};
    use super::Table;

    #[test]
    fn test_database_add_table() {
        let test_columns = vec![
            Column::new("id", true),
            Column::new("name", true),
            Column::new("email", true),
        ];

        let mut test_db = Database::new("test");
        let mut test_table = Table::new("test_table".to_string(), Columns::new(test_columns));

        let result = test_db.add_table(&mut test_table);
        assert_eq!(result, Ok(()));
        assert_eq!(test_db.tables.len(), 1);

        // need to figure out a better way to handle the condition where the table already exists
        let result = test_db.add_table(&mut test_table);
        assert_eq!(result, Ok(()));
        assert_eq!(test_db.tables.len(), 1);
    }
}
