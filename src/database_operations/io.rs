use std::path::Path;

use crate::Database;

impl Database {
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
}

#[cfg(test)]

mod tests {

    use super::*;
    use std::collections::HashMap;

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
}
