mod database;

use database::database::{Column, Columns, Database, Table};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use uuid::Uuid;

use once_cell::sync::Lazy;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub age: String,
}

const DATABASE_NAME: &str = "cargobase";
static FILE_NAME: Lazy<String> = Lazy::new(|| format!("{}.json", DATABASE_NAME));

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::new(DATABASE_NAME.to_string());

    let columns = Columns::new(vec![
        Column::new("id", true),
        Column::new("name", true),
        Column::new("age", true),
    ]);

    let mut users_table = Table::new("Users".to_string(), columns);

    db.add_table(&mut users_table);

    let user1 = User {
        id: Uuid::new_v4().to_string(),
        name: "John Doe".to_string(),
        age: "30".to_string(),
    };

    let user2 = User {
        id: Uuid::new_v4().to_string(),
        name: "Jane Doe".to_string(),
        age: "25".to_string(),
    };

    let users = vec![user1, user2];

    for user in users {
        if let Err(e) = db.add_row("Users", json!(user), &FILE_NAME) {
            println!("Failed to add row for user {}: {}", user.name, e);
        } else {
            ()
        }
    }

    println!("db: {:#?}", db);


    let user3 = User {
        id: Uuid::new_v4().to_string(),
        name: "apple banana".to_string(),
        age: "420".to_string(),
    };

    if let Err(e) = db.add_row("Users", json!(user3), &FILE_NAME) {
        println!("Failed to add row for user {}: {}", user3.name, e);
    } else {
        ()
    }

    println!("db: {:#?}", db);
    Ok(())
}
