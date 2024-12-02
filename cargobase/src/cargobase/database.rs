use serde::{Deserialize, Serialize};
use tracing;

use super::view::View;
use super::{query::Operation, Query, Table};

use cargobase_core::DatabaseError;

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

        if std::path::Path::new(&file_name).exists() {
            tracing::info!("Database already exists: {name}, loading database");

            // Load the database from the file
            if let Ok(db) = Database::load_from_file(&file_name) {
                return db;
            } else {
                tracing::error!("Failed to load database from file: {file_name}");
            }
        } else {
            tracing::info!("Creating new database: {file_name}");
            // Create an empty JSON file for the new database
            if let Err(e) = std::fs::write(&file_name, "{}") {
                tracing::error!("Failed to create database file: {e}");
            }
        }

        Database {
            name,
            file_name,
            tables: Vec::new(),
        }
    }

    // #[cfg(feature = "async")]
    // pub async fn new_async(name: &str) -> Self {
    //     let name = name.to_string();
    //     let file_name = format!("{name}.json");
    //
    //     if tokio::fs::metadata(&file_name).await.is_ok() {
    //         tracing::info!("Database already exists: {name}, loading database");
    //
    //         // Load the database from the file
    //         match Database::load_from_file_async(&file_name).await {
    //             Ok(db) => return db,
    //             Err(e) => {
    //                 tracing::error!("Failed to load database from file: {file_name}, error: {e}");
    //             }
    //         }
    //     } else {
    //         tracing::info!("Creating new database: {file_name}");
    //         // Create an empty JSON file for the new database
    //         if let Err(e) = tokio::fs::write(&file_name, "{}").await {
    //             tracing::error!("Failed to create database file: {e}");
    //         }
    //     }
    //
    //     Database {
    //         name,
    //         file_name,
    //         tables: Vec::new(),
    //     }
    // }

    pub fn drop_database(&self) -> Result<(), DatabaseError> {
        if std::fs::remove_file(&self.file_name).is_err() {
            tracing::error!(
                "{}",
                DatabaseError::DeleteError("Failed to delete database file".to_string())
            );
        }

        tracing::info!("Database `{}` dropped successfully", self.name);
        Ok(())
    }

    pub fn add_table(&mut self, table: &mut Table) -> Result<(), DatabaseError> {
        if self.tables.iter().any(|t| t.name == table.name) {
            tracing::warn!(
                "{}",
                DatabaseError::TableAlreadyExists(table.name.to_string())
            );
            return Ok(());
        }

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
            tracing::info!("Table `{}` dropped successfully", removed_table.name);
            db.save_to_file().map_err(|e| DatabaseError::SaveError(e))?;

            self.tables = db.tables;
            Ok(())
        } else {
            tracing::error!("{}", DatabaseError::TableNotFound(table_name.to_string()));
            Ok(())
        }
    }

    pub(crate) fn save_to_file(&self) -> Result<(), std::io::Error> {
        let json_data = serde_json::to_string_pretty(&self)?;
        std::fs::write(&self.file_name, json_data)?;
        tracing::info!("Database saved to file: {}", self.file_name);
        Ok(())
    }

    // #[cfg(feature = "async")]
    // pub(crate) async fn save_to_file_async(&self) -> Result<(), tokio::io::Error> {
    //     let json_data = serde_json::to_string_pretty(&self)?;
    //     tokio::fs::write(&self.file_name, json_data).await?;
    //     tracing::info!("Database saved to file: {}", self.file_name);
    //     Ok(())
    // }

    pub(crate) fn load_from_file(file_name: &str) -> Result<Self, std::io::Error> {
        let json_data = std::fs::read_to_string(file_name)?;
        let db: Database = serde_json::from_str(&json_data)?;
        tracing::info!("Database loaded from file: {}", file_name);
        Ok(db)
    }

    // #[cfg(feature = "async")]
    // pub(crate) async fn load_from_file_async(file_name: &str) -> Result<Self, tokio::io::Error> {
    //     let json_data = tokio::fs::read_to_string(file_name).await?;
    //     let db: Database = serde_json::from_str(&json_data)?;
    //     tracing::info!("Database loaded from file: {}", file_name);
    //     Ok(db)
    // }

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
        let view = View::new(self);
        view.all_tables();
    }

    pub fn view_table(&self, table_name: &str) {
        let view = View::new(self);
        view.single_table(table_name);
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;
    use crate::cargobase::setup_temp_db;
    use crate::Table;

    use cargobase_core::Columns;

    // #[cfg(feature = "async")]
    // use crate::cargobase::setup_temp_db_async;

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

    // #[cfg(feature = "async")]
    // #[tokio::test]
    // async fn test_database_new_async() {
    //     let db = setup_temp_db_async().await;
    //
    //     let db_name = &db.name.to_string();
    //     let fnn = format!("{db_name}.json");
    //
    //     assert_eq!(db.name, db_name.to_string());
    //     assert_eq!(db.file_name, fnn);
    //     assert_eq!(db.tables.len(), 1); // the setup_temp_db function adds a table
    // }

    #[test]
    fn test_drop_database() {
        let db = setup_temp_db();
        let result = db.drop_database();

        assert!(result.is_ok());
        assert!(!std::path::Path::new(&db.file_name).exists());
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

    #[traced_test]
    #[test]
    fn test_add_table_already_exists() {
        let mut db = setup_temp_db();

        // Create a duplicate table
        let columns = Columns::from_struct::<TestData>(true);
        let mut duplicate_table = Table::new("TestTable".to_string(), columns);
        let result = db.add_table(&mut duplicate_table);

        // Assert that the result is Ok(()) even when the table already exists
        assert!(result.is_ok());

        // Ensure no duplicate tables exist
        assert_eq!(db.tables.len(), 1);

        let db_error = DatabaseError::TableAlreadyExists("TestTable".to_string());
        let logs = logs_contain(&format!("{}", db_error));
        assert!(logs, "Expected warning log for existing table not found.");
    }

    #[test]
    fn test_drop_table_success() {
        let mut db = setup_temp_db();
        let result = db.drop_table("TestTable");

        assert!(result.is_ok());
        assert_eq!(db.tables.len(), 0);
    }

    #[traced_test]
    #[test]
    fn test_drop_table_not_found() {
        let mut db = setup_temp_db();
        let result = db.drop_table("NonExistentTable");

        assert!(result.is_ok());

        // Assert that an error is returned
        let db_error = DatabaseError::TableNotFound("NonExistentTable".to_string());
        let logs = logs_contain(&format!("{}", db_error));
        assert!(logs, "Expected error log for non-existent table not found.");

        // Ensure no tables were removed
        assert_eq!(db.tables.len(), 1);
    }

    #[test]
    fn test_save_to_file() {
        let db = setup_temp_db();
        let result = db.save_to_file();

        assert!(result.is_ok());
        assert!(std::path::Path::new(&db.file_name).exists());
    }

    #[test]
    fn test_load_from_file() {
        let db = setup_temp_db();
        let loaded_db = Database::load_from_file(&db.file_name).expect("Failed to load database");

        assert_eq!(db, loaded_db);
    }

    // #[cfg(feature = "async")]
    // #[tokio::test]
    // async fn test_save_to_file_async() {
    //     use tempfile::NamedTempFile;
    //
    //     let temp_file = NamedTempFile::new().expect("Failed to create a temporary file");
    //     let db_path = temp_file.path().to_str().unwrap().to_string();
    //
    //     let db = Database {
    //         name: "test_db".to_string(),
    //         file_name: db_path.clone(),
    //         tables: vec![],
    //     };
    //
    //     db.save_to_file_async()
    //         .await
    //         .expect("Failed to save database");
    //     let loaded_db = Database::load_from_file(&db_path).expect("Failed to load database");
    //     assert_eq!(db, loaded_db);
    // }

    // #[cfg(feature = "async")]
    // #[tokio::test]
    // async fn test_load_from_file_async() {
    //     use tempfile::NamedTempFile;
    //
    //     let temp_file = NamedTempFile::new().expect("Failed to create a temporary file");
    //     let db_path = temp_file.path().to_str().unwrap().to_string();
    //
    //     let db = Database {
    //         name: "test_db".to_string(),
    //         file_name: db_path.to_string(),
    //         tables: vec![],
    //     };
    //
    //     db.save_to_file_async()
    //         .await
    //         .expect("Failed to save database");
    //
    //     let loaded_db = Database::load_from_file_async(&db_path)
    //         .await
    //         .expect("Failed to load database");
    //
    //     assert_eq!(db, loaded_db);
    // }
}
