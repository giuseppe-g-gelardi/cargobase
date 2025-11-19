use std::{borrow::Cow, collections::HashMap};

use cargobase::{setup_temp_db, Column, Columns, Database, DatabaseError, Operation, Table};

use tracing_test::traced_test;

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
struct TestData {
    id: String,
    name: String,
}

#[tokio::test]
async fn test_database_new() {
    let db = setup_temp_db().await;

    // let db_name = &db.name.to_string();
    let db_name: &str = db.name.as_ref();
    let fnn = format!("{db_name}.json");

    // assert_eq!(db.name, db_name.to_string());
    assert_eq!(db.name, Cow::Owned(db.name.to_string()));
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
async fn test_drop_database_error_nonexistent() {
    // Create a database but don't save it (file doesn't exist)
    let db = Database {
        name: "nonexistent_db".to_string(),
        file_name: "nonexistent_db_xyz_12345.json".into(),
        tables: HashMap::new(),
    };

    // Try to drop a database file that doesn't exist
    let result = db.drop_database().await;

    // Should return an error
    assert!(result.is_err());
    match result {
        Err(DatabaseError::DeleteError(msg)) => {
            assert!(msg.contains("nonexistent_db"));
        }
        _ => panic!("Expected DeleteError"),
    }
}

#[tokio::test]
async fn test_add_table_success() {
    // this test does not use the setup_temp_db function
    // because it needs to test the creation of a new database and table
    tokio::fs::remove_file("test_db.json").await.ok();
    let mut db = Database::new(Cow::Borrowed("test_db")).await;

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

    // Should return an error when table already exists
    assert!(result.is_err());
    match result {
        Err(DatabaseError::TableAlreadyExists(name)) => {
            assert_eq!(name, "TestTable");
        }
        _ => panic!("Expected TableAlreadyExists error"),
    }

    // Ensure no duplicate tables exist
    assert_eq!(db.tables.len(), 1);

    // let db_error = DatabaseError::TableAlreadyExists("TestTable".to_string());
    // let logs = logs_contain(&format!("{}", db_error));
    // assert!(logs, "Expected warning log for existing table not found.");
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

    // Should return an error when table doesn't exist
    assert!(result.is_err());
    match result {
        Err(DatabaseError::TableNotFound(name)) => {
            assert_eq!(name, "NonExistentTable");
        }
        _ => panic!("Expected TableNotFound error"),
    }

    // // Assert that an error is returned
    // let db_error = DatabaseError::TableNotFound("NonExistentTable".to_string());
    // let logs = logs_contain(&format!("{}", db_error));
    // assert!(logs, "Expected error log for non-existent table not found.");

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
