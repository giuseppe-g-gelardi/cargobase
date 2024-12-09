use serde::{Deserialize, Serialize};
use tracing;

use crate::{query::Operation, Query, Table, View};
use cargobase_core::DatabaseError;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DatabaseAsync {
    pub(crate) name: String,
    pub(crate) file_name: String,
    pub(crate) tables: Vec<Table>,
}

impl DatabaseAsync {
    pub async fn new_async(name: &str) -> Self {
        let name = name.to_string();
        let file_name = format!("{name}.json");

        if tokio::fs::metadata(&file_name).await.is_ok() {
            tracing::info!("Database already exists: {name}, loading database");

            // Load the database from the file
            match DatabaseAsync::load_from_file_async(&file_name).await {
                Ok(db) => return db,
                Err(e) => {
                    tracing::error!("Failed to load database from file: {file_name}, error: {e}");
                }
            }
        } else {
            tracing::info!("Creating new database: {file_name}");
            // Create an empty JSON file for the new database
            if let Err(e) = tokio::fs::write(&file_name, "{}").await {
                tracing::error!("Failed to create database file: {e}");
            }
        }

        DatabaseAsync {
            name,
            file_name,
            tables: Vec::new(),
        }
    }

    pub async fn drop_database_async(&self) -> Result<(), DatabaseError> {
        if tokio::fs::remove_file(&self.file_name).await.is_err() {
            tracing::error!(
                "{}",
                DatabaseError::DeleteError("Failed to delete database file".to_string())
            );
        }

        tracing::info!("Database `{}` dropped successfully", self.name);
        Ok(())
    }

    pub async fn add_table_async(&mut self, table: &mut Table) -> Result<(), DatabaseError> {
        if self.tables.iter().any(|t| t.name == table.name) {
            tracing::warn!(
                "{}",
                DatabaseError::TableAlreadyExists(table.name.to_string())
            );
            return Ok(());
        }

        self.tables.push(table.clone());
        self.save_to_file_async()
            .await
            .map_err(|e| DatabaseError::SaveError(e))?;
        Ok(())
    }

    pub async fn drop_table_async(&mut self, table_name: &str) -> Result<(), DatabaseError> {
        let mut db = DatabaseAsync::load_from_file_async(&self.file_name)
            .await
            .map_err(|e| DatabaseError::LoadError(e))?;

        if let Some(index) = db.tables.iter().position(|t| t.name == table_name) {
            let removed_table = db.tables.remove(index);
            tracing::info!("Table `{}` dropped successfully", removed_table.name);
            db.save_to_file_async()
                .await
                .map_err(|e| DatabaseError::SaveError(e))?;

            self.tables = db.tables;
            Ok(())
        } else {
            tracing::error!("{}", DatabaseError::TableNotFound(table_name.to_string()));
            Ok(())
        }
    }

    pub(crate) async fn save_to_file_async(&self) -> Result<(), tokio::io::Error> {
        let json_data = serde_json::to_string_pretty(&self)?;
        tokio::fs::write(&self.file_name, json_data).await?;
        tracing::info!("Database saved to file: {}", self.file_name);
        Ok(())
    }

    pub(crate) async fn load_from_file_async(file_name: &str) -> Result<Self, tokio::io::Error> {
        let json_data = tokio::fs::read_to_string(file_name).await?;
        let db: DatabaseAsync = serde_json::from_str(&json_data)?;
        tracing::info!("Database loaded from file: {}", file_name);
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
    use crate::{setup_temp_db_async, Table};
    use cargobase_core::Columns;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
    struct TestData {
        id: String,
        name: String,
    }

    #[tokio::test]
    async fn test_database_new_async() {
        let db = setup_temp_db_async().await;

        let db_name = &db.name.to_string();
        let fnn = format!("{db_name}.json");

        assert_eq!(db.name, db_name.to_string());
        assert_eq!(db.file_name, fnn);
        assert_eq!(db.tables.len(), 1); // the setup_temp_db function adds a table
    }

    #[tokio::test]
    async fn test_drop_database_async() {
        let db = setup_temp_db_async().await;
        let result = db.drop_database_async().await;

        assert!(result.is_ok());
        assert!(!std::path::Path::new(&db.file_name).exists());
    }

    #[tokio::test]
    async fn test_add_table_success_async() {
        // this test does not use the setup_temp_db function
        // because it needs to test the creation of a new database and table
        tokio::fs::remove_file("test_db.json").await.ok();
        let mut db = DatabaseAsync::new_async("test_db").await;
        let test_columns = Columns::from_struct::<TestData>(true);
        let mut test_table = Table::new("TestTable".to_string(), test_columns);

        let result = db.add_table_async(&mut test_table).await;

        assert!(result.is_ok());
        assert_eq!(db.tables.len(), 1);
        assert_eq!(db.tables[0].name, "TestTable");

        tokio::fs::remove_file("test_db.json").await.ok();
    }

    #[traced_test]
    #[tokio::test]
    async fn test_add_table_already_exists_async() {
        let mut db = setup_temp_db_async().await;

        // Create a duplicate table
        let columns = Columns::from_struct::<TestData>(true);
        let mut duplicate_table = Table::new("TestTable".to_string(), columns);
        let result = db.add_table_async(&mut duplicate_table).await;

        // Assert that the result is Ok(()) even when the table already exists
        assert!(result.is_ok());

        // Ensure no duplicate tables exist
        assert_eq!(db.tables.len(), 1);

        let db_error = DatabaseError::TableAlreadyExists("TestTable".to_string());
        let logs = logs_contain(&format!("{}", db_error));
        assert!(logs, "Expected warning log for existing table not found.");
    }

    #[tokio::test]
    async fn test_drop_table_success_async() {
        let mut db = setup_temp_db_async().await;
        let result = db.drop_table_async("TestTable").await;

        assert!(result.is_ok());
        assert_eq!(db.tables.len(), 0);
    }

    #[traced_test]
    #[tokio::test]
    async fn test_drop_table_not_found_async() {
        let mut db = setup_temp_db_async().await;
        let result = db.drop_table_async("NonExistentTable").await;

        assert!(result.is_ok());

        // Assert that an error is returned
        let db_error = DatabaseError::TableNotFound("NonExistentTable".to_string());
        let logs = logs_contain(&format!("{}", db_error));
        assert!(logs, "Expected error log for non-existent table not found.");

        // Ensure no tables were removed
        assert_eq!(db.tables.len(), 1);
    }

    #[tokio::test]
    async fn test_save_to_file_async() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create a temporary file");
        let db_path = temp_file.path().to_str().unwrap().to_string();

        let db = DatabaseAsync {
            name: "test_db".to_string(),
            file_name: db_path.clone(),
            tables: vec![],
        };

        db.save_to_file_async()
            .await
            .expect("Failed to save database");
        let loaded_db = DatabaseAsync::load_from_file_async(&db_path)
            .await
            .expect("Failed to load database");
        assert_eq!(db, loaded_db);
    }

    #[tokio::test]
    async fn test_load_from_file_async() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create a temporary file");
        let db_path = temp_file.path().to_str().unwrap().to_string();

        let db = DatabaseAsync {
            name: "test_db".to_string(),
            file_name: db_path.to_string(),
            tables: vec![],
        };

        db.save_to_file_async()
            .await
            .expect("Failed to save database");

        let loaded_db = DatabaseAsync::load_from_file_async(&db_path)
            .await
            .expect("Failed to load database");

        assert_eq!(db, loaded_db);
    }
}
