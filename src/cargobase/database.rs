use serde::{Deserialize, Serialize};

use super::DatabaseError;
use super::{query::Operation, Query, Table};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Database {
    pub(crate) name: String,
    pub(crate) file_name: String,
    pub(crate) tables: Vec<Table>,
}

impl Database {
    pub fn new(name: &str) -> Self {
        let name = name.to_string();
        let file_name = format!("{name}.json");

        // find a better way of logging this information for the end user
        // -- they might not have tracing enabled

        if std::path::Path::new(&file_name).exists() {
            println!("Database already exists: {name}, loading database");

            if let Ok(db) = Database::load_from_file(&file_name) {
                return db;
            } else {
                eprintln!("Failed to load database from file: {file_name}");
            }
        } else {
            println!("Creating new database: {file_name}");
            // Create an empty JSON file for the new database
            if let Err(e) = std::fs::write(&file_name, "{}") {
                eprintln!("Failed to create database file: {e}");
            }
        }

        Database {
            name,
            file_name,
            tables: Vec::new(),
        }
    }

    pub fn add_table(&mut self, table: &mut Table) -> Result<(), DatabaseError> {
        table.set_file_name(self.file_name.clone());
        if self.tables.iter().any(|t| t.name == table.name) {
            return Err(DatabaseError::TableAlreadyExists(table.name.clone()));
        }
        // IF the table does not exist, add it to the database
        // IF the table exists:
        // -- do NOT add a duplicate to the db
        // -- let the user know that the table already exists
        // -- do NOT crash the program, just return and move on

        self.tables.push(table.clone());
        self.save_to_file()
            .map_err(|e| DatabaseError::SaveError(e))?;
        Ok(())
    }

    pub fn drop_table(&mut self, table_name: &str) -> Result<(), DatabaseError> {
        let mut db =
            Database::load_from_file(&self.file_name).map_err(|e| DatabaseError::LoadError(e))?;

        if let Some(index) = db.tables.iter().position(|t| t.name == table_name) {
            let removed_table = db.tables.remove(index);
            println!("Table `{}` dropped successfully", removed_table.name);
            db.save_to_file().map_err(|e| DatabaseError::SaveError(e))?;

            // IF the table does not exist:
            // -- let the user know that the table does not exist 
            // -- do NOT crash the program, just return and move on
            //
            // IF the table exists:
            // -- remove the table from the db 
            // -- save the db to file
            // -- let the user know that the table was removed successfully

            self.tables = db.tables;
            Ok(())
        } else {
            // eprintln!("{}", DatabaseError::TableNotFound(table_name.to_string()));
            Err(DatabaseError::TableNotFound(table_name.to_string()))
            // Ok(())
        }
    }

    pub(crate) fn save_to_file(&self) -> Result<(), std::io::Error> {
        let json_data = serde_json::to_string_pretty(&self)?;
        std::fs::write(&self.file_name, json_data)?;
        println!("Database saved to file: {}", self.file_name);
        Ok(())
    }

    // pub(crate) fn save_to_file(&self) -> Result<(), std::io::Error> {
    //     // Serialize the overall structure with pretty formatting
    //     let mut json_data = serde_json::to_string_pretty(self)?;
    //
    //     // Adjust the rows to be single-line JSON
    //     if let Some(index) = json_data.find("\"rows\": [") {
    //         let rows_start = index + "\"rows\": [".len();
    //         let rows_end = json_data[rows_start..].find(']').unwrap_or(0) + rows_start;
    //
    //         // Extract and reformat rows
    //         let rows_json = &json_data[rows_start..rows_end];
    //         let formatted_rows = self
    //             .tables
    //             .iter()
    //             .flat_map(|table| {
    //                 table
    //                     .rows
    //                     .iter()
    //                     .map(|row| serde_json::to_string(row).unwrap())
    //             })
    //             .collect::<Vec<_>>()
    //             .join(",");
    //
    //         // Replace rows in the JSON data
    //         json_data.replace_range(rows_start..rows_end, &formatted_rows);
    //     }
    //
    //     // Save to file
    //     std::fs::write(&self.file_name, json_data)?;
    //     // tracing::info!(target: "cargobase", "Database saved to file: {}", self.file_name);
    //     Ok(())
    // }

    pub(crate) fn load_from_file(file_name: &str) -> Result<Self, std::io::Error> {
        let json_data = std::fs::read_to_string(file_name)?;
        let db: Database = serde_json::from_str(&json_data)?;
        Ok(db)
    }

    pub(crate) fn get_table_mut(&mut self, table_name: &str) -> Option<&mut Table> {
        self.tables.iter_mut().find(|t| t.name == table_name)
    }

    pub fn add_row(&mut self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Create,
            update_data: None,
            row_data: None,
        }
    }

    pub fn get_rows(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Read,
            update_data: None,
            row_data: None,
        }
    }

    pub fn get_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Read,
            update_data: None,
            row_data: None,
        }
    }

    pub fn delete_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Delete,
            update_data: None,
            row_data: None,
        }
    }

    pub fn update_row(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Update,
            update_data: None,
            row_data: None,
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
    use crate::cargobase::setup_temp_db;
    use crate::{Columns, Table};

    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
    struct TestData {
        id: String,
        name: String,
    }

    #[test]
    fn test_database_new() {
        let db = setup_temp_db();

        let db_name = &db.name.to_string();
        let fnn = format!("{db_name}.json");

        assert_eq!(db.name, db_name.to_string());
        assert_eq!(db.file_name, fnn);
        assert_eq!(db.tables.len(), 1); // the setup_temp_db function adds a table
    }

    #[test]
    fn test_add_table_success() {
        // this test does not use the setup_temp_db function
        // because it needs to test the creation of a new database and table
        std::fs::remove_file("test_db.json").ok();
        let mut db = Database::new("test_db");
        let test_columns = Columns::from_struct::<TestData>(true);
        let mut test_table = Table::new("TestTable".to_string(), test_columns);

        let result = db.add_table(&mut test_table);

        assert!(result.is_ok());
        assert_eq!(db.tables.len(), 1);
        assert_eq!(db.tables[0].name, "TestTable");
        // assert_eq!(db.tables[0].file_name, Some("test_db.json".to_string()));

        // remove the test_db.json file after testing
        std::fs::remove_file("test_db.json").ok();
    }

    #[test]
    fn test_add_table_already_exists() {
        let mut db = setup_temp_db();

        let columns = Columns::from_struct::<TestData>(true);
        let mut duplicate_table = Table::new("TestTable".to_string(), columns);
        let result = db.add_table(&mut duplicate_table);

        // Assert that an error is returned
        assert!(matches!(result, Err(DatabaseError::TableAlreadyExists(_))));

        if let Err(DatabaseError::TableAlreadyExists(name)) = result {
            assert_eq!(name, "TestTable");
        }

        // Ensure no duplicate tables exist
        assert_eq!(db.tables.len(), 1);
    }

    #[test]
    fn test_drop_table_success() {
        let mut db = setup_temp_db();
        let result = db.drop_table("TestTable");

        assert!(result.is_ok());
        assert_eq!(db.tables.len(), 0);
    }

    #[test]
    fn test_drop_table_not_found() {
        let mut db = setup_temp_db();
        let result = db.drop_table("NonExistentTable");

        // Assert that an error is returned
        assert!(matches!(result, Err(DatabaseError::TableNotFound(_))));

        if let Err(DatabaseError::TableNotFound(name)) = result {
            assert_eq!(name, "NonExistentTable");
        }

        // Ensure no tables were removed
        assert_eq!(db.tables.len(), 1);
    }
}
