use std::{borrow::Cow, collections::HashMap};
use std::path::Path;
use tracing;

use crate::{Database, DatabaseError, Operation, Query, Table, View};

impl Database {
    pub async fn new(name: Cow<'_, str>) -> Self {
        // let name = name.to_string();
        let name = name.into_owned();
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
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn get_rows(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Read,
            update_data: None,
            row_data: None,
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn get_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Read,
            update_data: None,
            row_data: None,
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn delete_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Delete,
            update_data: None,
            row_data: None,
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn update_row(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Update,
            update_data: None,
            row_data: None,
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

#[tokio::test]
async fn test_save_to_file() {

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
    use crate::setup_temp_db;
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
    struct TestData {
        id: String,
        name: String,
    }
    let mut db = setup_temp_db().await;
    let test_columns = crate::Columns::from_struct::<TestData>(true);

    let mut table = Table::new("test_table_mut".to_string(), test_columns.clone());
    db.add_table(&mut table)
        .await
        .expect("failed to add test_table_mut");

    let table = db.get_table_mut("test_table_mut");
    assert!(table.is_some());
}

} // end of tests module
