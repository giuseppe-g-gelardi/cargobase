use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing;

use crate::{query::Operation, DatabaseError, Query, Table, View};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Database {
    pub(crate) name: String,
    pub(crate) file_name: String,
    pub(crate) tables: HashMap<String, Table>, // pub(crate) tables: Vec<Table>,
}

impl Database {
    pub async fn new(name: &str) -> Self {
        let name = name.to_string();
        let file_name = format!("{name}.json");

        if tokio::fs::metadata(&file_name).await.is_ok() {
            tracing::info!("Database already exists: {name}, loading database");

            // Load the database from the file
            match Database::load_from_file(&file_name).await {
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

        Database {
            name,
            file_name,
            tables: HashMap::new(), // tables: Vec::new(),
        }
    }

    pub async fn drop_database(&self) -> Result<(), DatabaseError> {
        if tokio::fs::remove_file(&self.file_name).await.is_err() {
            tracing::error!(
                "{}",
                DatabaseError::DeleteError("Failed to delete database file".to_string())
            );
        }

        tracing::info!("Database `{}` dropped successfully", self.name);
        Ok(())
    }

    pub async fn add_table(&mut self, table: &mut Table) -> Result<(), DatabaseError> {
        if self.tables.contains_key(&table.name) {
            tracing::warn!(
                "{}",
                DatabaseError::TableAlreadyExists(table.name.to_string())
            );
            return Ok(());
        }

        self.tables.insert(table.name.clone(), table.clone());
        self.save_to_file()
            .await
            .map_err(|e| DatabaseError::SaveError(e))?;
        Ok(())
    }

    pub async fn drop_table(&mut self, table_name: &str) -> Result<(), DatabaseError> {
        let mut db = Database::load_from_file(&self.file_name)
            .await
            .map_err(|e| DatabaseError::LoadError(e))?;

        if let Some(removed_table) = db.tables.remove(table_name) {
            tracing::info!("Table `{}` dropped successfully", removed_table.name);
            db.save_to_file()
                .await
                .map_err(|e| DatabaseError::SaveError(e))?;

            self.tables = db.tables;
            Ok(())
        } else {
            tracing::error!("{}", DatabaseError::TableNotFound(table_name.to_string()));
            Ok(())
        }
    }

    pub(crate) async fn save_to_file(&self) -> Result<(), tokio::io::Error> {
        let json_data = serde_json::to_string_pretty(&self)?;
        tokio::fs::write(&self.file_name, json_data).await?;
        tracing::info!("Database saved to file: {}", self.file_name);
        Ok(())
    }

    pub(crate) async fn load_from_file(file_name: &str) -> Result<Self, tokio::io::Error> {
        let json_data = tokio::fs::read_to_string(file_name).await?;
        let db: Database = serde_json::from_str(&json_data)?;
        tracing::info!("Database loaded from file: {}", file_name);
        Ok(db)
    }

    pub(crate) fn get_table_mut(&mut self, table_name: &str) -> Option<&mut Table> {
        // self.tables.get_mut(table_name)
        tracing::debug!("looking for table: {}", table_name);
        let table = self.tables.get_mut(table_name);

        if let Some(_) = table {
            tracing::debug!("table found: {}", table_name);
        } else {
            tracing::error!("table not found: {}", table_name);
        }

        table
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

    pub fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    pub async fn rename_table(
        &mut self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), DatabaseError> {
        if old_name == new_name {
            return Err(DatabaseError::InvalidData(
                "old name and new name are the same".to_string(),
            ));
        }

        let table = self.tables.remove(old_name).ok_or_else(|| {
            DatabaseError::TableNotFound(format!("Table {} not found", old_name.to_string()))
        });

        if self.tables.contains_key(new_name) {
            return Err(DatabaseError::TableAlreadyExists(new_name.to_string()));
        }

        let mut table = table?;
        table.name = new_name.to_string();
        self.tables.insert(new_name.to_string(), table);

        self.save_to_file()
            .await
            .map_err(DatabaseError::SaveError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;
    use crate::{setup_temp_db, Column, Columns, Table};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
    struct TestData {
        id: String,
        name: String,
    }

    #[tokio::test]
    async fn test_database_new() {
        let db = setup_temp_db().await;

        let db_name = &db.name.to_string();
        let fnn = format!("{db_name}.json");

        assert_eq!(db.name, db_name.to_string());
        assert_eq!(db.file_name, fnn);
        assert_eq!(db.tables.len(), 1); // the setup_temp_db function adds a table
    }

    #[tokio::test]
    async fn test_drop_database() {
        let db = setup_temp_db().await;
        let result = db.drop_database().await;

        assert!(result.is_ok());
        assert!(!std::path::Path::new(&db.file_name).exists());
    }

    #[tokio::test]
    async fn test_add_table_success() {
        // this test does not use the setup_temp_db function
        // because it needs to test the creation of a new database and table
        tokio::fs::remove_file("test_db.json").await.ok();
        let mut db = Database::new("test_db").await;

        let test_columns = Columns::from_struct::<TestData>(true);
        let mut test_table = Table::new("TestTable".to_string(), test_columns);

        let result = db.add_table(&mut test_table).await;

        assert!(result.is_ok());
        assert_eq!(db.tables.len(), 1);
        // assert_eq!(db.tables[0].name, "TestTable");
        assert!(db.tables.contains_key("TestTable"));

        tokio::fs::remove_file("test_db.json").await.ok();
    }

    #[traced_test]
    #[tokio::test]
    async fn test_add_table_already_exists() {
        let mut db = setup_temp_db().await;

        // Create a duplicate table
        let columns = Columns::from_struct::<TestData>(true);
        let mut duplicate_table = Table::new("TestTable".to_string(), columns);
        let result = db.add_table(&mut duplicate_table).await;

        // Assert that the result is Ok(()) even when the table already exists
        assert!(result.is_ok());

        // Ensure no duplicate tables exist
        assert_eq!(db.tables.len(), 1);

        let db_error = DatabaseError::TableAlreadyExists("TestTable".to_string());
        let logs = logs_contain(&format!("{}", db_error));
        assert!(logs, "Expected warning log for existing table not found.");
    }

    #[tokio::test]
    async fn test_drop_table_success() {
        let mut db = setup_temp_db().await;
        let result = db.drop_table("TestTable").await;

        assert!(result.is_ok());
        assert_eq!(db.tables.len(), 0);
    }

    #[traced_test]
    #[tokio::test]
    async fn test_drop_table_not_found() {
        let mut db = setup_temp_db().await;
        let result = db.drop_table("NonExistentTable").await;

        assert!(result.is_ok());

        // Assert that an error is returned
        let db_error = DatabaseError::TableNotFound("NonExistentTable".to_string());
        let logs = logs_contain(&format!("{}", db_error));
        assert!(logs, "Expected error log for non-existent table not found.");

        // Ensure no tables were removed
        assert_eq!(db.tables.len(), 1);
    }

    #[tokio::test]
    async fn test_save_to_file() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create a temporary file");
        let db_path = temp_file.path().to_str().unwrap().to_string();

        let db = Database {
            name: "test_db".to_string(),
            file_name: db_path.clone(),
            tables: HashMap::new(),
        };

        db.save_to_file().await.expect("Failed to save database");
        let loaded_db = Database::load_from_file(&db_path)
            .await
            .expect("Failed to load database");
        assert_eq!(db, loaded_db);
    }

    #[tokio::test]
    async fn test_load_from_file() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create a temporary file");
        let db_path = temp_file.path().to_str().unwrap().to_string();

        let db = Database {
            name: "test_db".to_string(),
            file_name: db_path.to_string(),
            tables: HashMap::new(),
        };

        db.save_to_file().await.expect("Failed to save database");

        let loaded_db = Database::load_from_file(&db_path)
            .await
            .expect("Failed to load database");

        assert_eq!(db, loaded_db);
    }

    #[tokio::test]
    async fn test_rename_table_success() {
        let mut db = setup_temp_db().await;

        db.rename_table("TestTable", "RenamedTable")
            .await
            .expect("Failed to rename table");

        assert!(db.tables.contains_key("RenamedTable"));
        assert!(!db.tables.contains_key("TestTable"));
    }

    #[tokio::test]
    async fn test_rename_table_already_exists() {
        let mut db = setup_temp_db().await;

        let mut another_table = Table::new(
            "AnotherTable".to_string(),
            Columns::new(vec![Column::new("id", true)]),
        );
        db.add_table(&mut another_table).await.unwrap();

        let result = db.rename_table("TestTable", "AnotherTable").await;

        assert!(matches!(result, Err(DatabaseError::TableAlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_rename_table_not_found() {
        let mut db = setup_temp_db().await;

        let result = db.rename_table("NonExistentTable", "NewTable").await;

        assert!(matches!(result, Err(DatabaseError::TableNotFound(_))));
    }
}
