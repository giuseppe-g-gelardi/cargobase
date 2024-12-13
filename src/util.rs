use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::{Columns, Database, Table};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
struct TestData {
    id: String,
    name: String,
}

pub async fn setup_temp_db() -> Database {
    let temp_file = NamedTempFile::new().expect("Failed to create a temporary file");
    let db_path = temp_file.path().to_str().unwrap().to_string();

    // Initialize the test database
    let mut db = Database::new(&db_path).await;
    let test_columns = Columns::from_struct::<TestData>(true);

    let mut table = Table::new("TestTable".to_string(), test_columns);
    db.add_table(&mut table).await.unwrap();

    db.save_to_file().await.expect("Failed to save database");

    db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_temp_db() {
        let db = setup_temp_db().await;

        assert_eq!(db.tables.len(), 1);
        assert!(db.tables.contains_key("TestTable"));

        let table = db.tables.get("TestTable").unwrap();

        assert_eq!(table.name, "TestTable");
    }

    #[tokio::test]
    async fn test_temp_file_cleanup() {
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
