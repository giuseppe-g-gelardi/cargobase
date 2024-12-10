use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::{DatabaseAsync, Table};
use cargobase_core::Columns;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
struct TestData {
    id: String,
    name: String,
}

pub async fn setup_temp_db_async() -> DatabaseAsync {
    let temp_file = NamedTempFile::new().expect("Failed to create a temporary file");
    let db_path = temp_file.path().to_str().unwrap().to_string();

    // Initialize the test database
    let mut db = DatabaseAsync::new_async(&db_path).await;
    let test_columns = Columns::from_struct::<TestData>(true);

    let mut table = Table::new("TestTable".to_string(), test_columns);
    db.add_table_async(&mut table).await.unwrap();

    db.save_to_file_async()
        .await
        .expect("Failed to save database");

    db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_temp_db_async() {
        let db = setup_temp_db_async().await;
        assert_eq!(db.tables.len(), 1);
        assert_eq!(db.tables[0].name, "TestTable");
    }

    #[tokio::test]
    async fn test_temp_file_cleanup_async() {
        // Create a temporary database
        let temp_file = tempfile::Builder::new()
            .prefix("test_db")
            .suffix(".json")
            .tempfile()
            .expect("Failed to create a temporary file");

        let db_path = temp_file.path().to_str().unwrap().to_string();

        // Drop the file explicitly by dropping the `NamedTempFile` instance
        drop(temp_file);

        // Verify that the temporary file is removed
        let file_exists = tokio::fs::metadata(&db_path).await.is_ok();
        assert!(
            !file_exists,
            "Temporary file `{}` should have been removed after being dropped",
            db_path
        );
    }
}
