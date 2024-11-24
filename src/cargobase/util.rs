use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use super::{Columns, Database, Table};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
struct TestData {
    id: String,
    name: String,
}

pub fn setup_temp_db() -> Database {
    // Create a temporary file
    let temp_file = NamedTempFile::new().expect("Failed to create a temporary file");
    let db_path = temp_file.path().to_str().unwrap().to_string();

    // Initialize the test database
    let mut db = Database::new(&db_path);
    let test_columns = Columns::from_struct::<TestData>(true);

    let mut table = Table::new("TestTable".to_string(), test_columns);
    db.add_table(&mut table).unwrap();

    db
}
