use std::collections::HashMap;
use std::path::Path;
use tracing;

use crate::{Database, DatabaseError, Operation, Query, Table, View};

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
            file_name: file_name.into(),
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
            .map_err(DatabaseError::SaveError)?;
        Ok(())
    }

    pub async fn drop_table(&mut self, table_name: &str) -> Result<(), DatabaseError> {
        let mut db = Database::load_from_file(&self.file_name)
            .await
            .map_err(DatabaseError::LoadError)?;

        if let Some(removed_table) = db.tables.remove(table_name) {
            tracing::info!("Table `{}` dropped successfully", removed_table.name);
            db.save_to_file().await.map_err(DatabaseError::SaveError)?;

            self.tables = db.tables;
            Ok(())
        } else {
            tracing::error!("{}", DatabaseError::TableNotFound(table_name.to_string()));
            Ok(())
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

        let table = self
            .tables
            .remove(old_name)
            .ok_or_else(|| DatabaseError::TableNotFound(format!("Table {} not found", old_name)));

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

    pub fn count_rows(&self, table_name: &str) -> Result<usize, DatabaseError> {
        if let Some(table) = self.tables.get(table_name) {
            Ok(table.rows.len())
        } else {
            Err(DatabaseError::TableNotFound(format!(
                "Table {} not found",
                table_name
            )))
        }
    }

    pub fn record_exists(&self, table_name: &str, pk_value: &str) -> bool {
        if let Some(table) = self.tables.get(table_name) {
            table.rows.contains_key(pk_value)
        } else {
            false
        }
    }

    pub(crate) async fn save_to_file(&self) -> Result<(), tokio::io::Error> {
        let json_data = serde_json::to_string_pretty(&self)?;
        tokio::fs::write(&self.file_name, json_data).await?;
        tracing::info!("Database saved to file: {:?}", self.file_name);
        Ok(())
    }

    pub(crate) async fn load_from_file<P: AsRef<Path>>(
        file_name: P,
    ) -> Result<Self, tokio::io::Error> {
        let json_data = tokio::fs::read_to_string(file_name.as_ref()).await?;
        let db: Database = serde_json::from_str(&json_data)?;
        tracing::info!(
            "Database loaded from file: {:?}",
            file_name.as_ref().display()
        );
        Ok(db)
    }

    pub(crate) fn get_table_mut(&mut self, table_name: &str) -> Option<&mut Table> {
        tracing::debug!("looking for table: {}", table_name);
        let table = self.tables.get_mut(table_name);

        if table.is_some() {
            tracing::debug!("table found: {}", table_name);
        } else {
            tracing::error!("table not found: {}", table_name);
        }
        table
    }
}

impl Database {
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
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;
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
        assert_eq!(db.file_name.to_string_lossy(), fnn);
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

    #[tokio::test]
    async fn test_count_rows() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
        pub struct User {
            id: String,
            name: String,
            email: String,
        }
        let mut db = setup_temp_db().await;

        let user_columns = Columns::from_struct::<User>(true);
        //
        let mut users_table = Table::new("users".to_string(), user_columns.clone());
        db.add_table(&mut users_table)
            .await
            .expect("failed to add users table");

        let user1 = json!({
            "id": "1",
            "name": "John Doe",
            "email": "johndoe@example.com"
        });
        let user2 = json!({
            "id": "2",
            "name": "Jane Smith",
            "email": "janesmith@example.com"
        });
        let user3 = json!({
            "id": "3",
            "name": "Alice Johnson",
            "email": "alice@example.com"
        });

        let users = vec![user1, user2, user3];

        // add single rows
        // users_table.add_row(&mut db, user1).await;
        // users_table.add_row(&mut db, user2).await;
        // users_table.add_row(&mut db, user3).await;

        // add array of rows.... .into() converts Vec<serde_json::Value> to Vec<Row>???
        users_table.add_row(&mut db, users.into()).await;

        // Count rows in the table
        let row_count = db.count_rows("users").unwrap();
        assert_eq!(row_count, 3);

        // Attempt to count rows for a non-existent table
        let result = db.count_rows("NonExistentTable");
        assert!(matches!(result, Err(DatabaseError::TableNotFound(_))));
    }

    #[tokio::test]
    async fn test_foreign_key_validation() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
        struct Post {
            id: String,
            title: String,
            content: String,
            user_id: String,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
        struct User {
            id: String,
            name: String,
            email: String,
        }

        let mut db = setup_temp_db().await;

        // Set up User table
        let user_columns = Columns::from_struct::<User>(true);
        let mut users_table = Table::new("users".to_string(), user_columns.clone());
        db.add_table(&mut users_table).await.unwrap();

        let user1 = json!({
            "id": "1",
            "name": "John Doe",
            "email": "johndoe@example.com"
        });
        users_table.add_row(&mut db, user1).await;

        // Set up Post table
        let post_columns = Columns::from_struct::<Post>(true);
        let mut posts_table = Table::new("posts".to_string(), post_columns.clone());
        db.add_table(&mut posts_table).await.unwrap();

        let valid_post = json!({
            "id": "101",
            "title": "Valid Post",
            "content": "Content",
            "user_id": "1"
        });

        let invalid_post = json!({
            "id": "102",
            "title": "Invalid Post",
            "content": "Content",
            "user_id": "999"
        });

        // Valid FK
        assert!(posts_table
            .add_row_with_fk(&db, valid_post, Some(&[("users", "user_id")]))
            .is_ok());

        // Invalid FK
        assert!(posts_table
            .add_row_with_fk(&db, invalid_post, Some(&[("users", "user_id")]))
            .is_err());
    }

    #[tokio::test]
    async fn test_save_to_file() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create a temporary file");
        let db_path = temp_file.path().to_path_buf();

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
        let db_path = temp_file.path().to_path_buf();

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
    async fn test_get_table_mut() {
        let mut db = setup_temp_db().await;
        let test_columns = crate::Columns::from_struct::<TestData>(true);

        let mut table = Table::new("test_table_mut".to_string(), test_columns.clone());
        db.add_table(&mut table)
            .await
            .expect("failed to add test_table_mut");

        let table = db.get_table_mut("test_table_mut");
        assert!(table.is_some());
    }

    #[tokio::test]
    async fn test_add_row() {
        let mut db = Database {
            name: "test_db".to_string(),
            file_name: "test_db.json".into(),
            tables: HashMap::new(),
        };

        let query = db.add_row();
        assert_eq!(query.operation, Operation::Create);
    }

    #[tokio::test]
    async fn test_get_rows() {
        let db = Database {
            name: "test_db".to_string(),
            file_name: "test_db.json".into(),
            tables: HashMap::new(),
        };

        let query = db.get_rows();
        assert_eq!(query.operation, Operation::Read);
    }

    #[tokio::test]
    async fn test_get_single() {
        let db = Database {
            name: "test_db".to_string(),
            file_name: "test_db.json".into(),
            tables: HashMap::new(),
        };

        let query = db.get_single();
        assert_eq!(query.operation, Operation::Read);
    }

    #[tokio::test]
    async fn test_delete_single() {
        let db = Database {
            name: "test_db".to_string(),
            file_name: "test_db.json".into(),
            tables: HashMap::new(),
        };

        let query = db.delete_single();
        assert_eq!(query.operation, Operation::Delete);
    }

    #[tokio::test]
    async fn test_update_row() {
        let db = Database {
            name: "test_db".to_string(),
            file_name: "test_db.json".into(),
            tables: HashMap::new(),
        };

        let query = db.update_row();
        assert_eq!(query.operation, Operation::Update);
    }
}
