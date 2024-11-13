mod database;
mod util;

use database::database::Database;
use database::rows::Row;
use database::tables::Table;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use util::util::{load_from_file, save_to_file};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub age: String,
}

const DATABASE_NAME: &str = "cargobase";
const FILE_NAME: &str = "cargobase.json";

fn main() -> Result<(), Box<dyn Error>> {
    // let new_db = create_new_database();
    println!("{}", DATABASE_NAME.to_string());
    println!("{}", FILE_NAME.to_string());

    let _database: Database = load_from_file(FILE_NAME)?;

    let user = User {
        id: Uuid::new_v4().to_string(),
        name: "Charlie".to_string(),
        age: "35".to_string(),
    };

    let user_data = json!(user);

    let row = Row::new(user_data);

    let mut table = Table::new(
        "Users".to_string(),
        vec!["id".to_string(), "name".to_string(), "age".to_string()],
        vec![row],
    );

    let mut db = Database::new(DATABASE_NAME.to_string());

    db.add_table(table);

    save_to_file(FILE_NAME, &db)?;

    Ok(())
}

// fn create_new_database() -> Result<(), Box<dyn Error>> {
//     let mut db = Database::new(DATABASE_NAME.to_string());
//     let columns = vec!["id".to_string(), "name".to_string(), "age".to_string()];
//     let mut users_table = Table::<User>::new("Users".to_string(), columns);
//
//     let user1 = User {
//         id: Uuid::new_v4().to_string(),
//         name: "Alice".to_string(),
//         age: "30".to_string(),
//     };
//
//     let user2 = User {
//         id: Uuid::new_v4().to_string(),
//         name: "Bob".to_string(),
//         age: "25".to_string(),
//     };
//
//     users_table.add_row(Row::new(user1));
//     users_table.add_row(Row::new(user2));
//
//     db.add_table(users_table);
//
//     save_to_file(FILE_NAME, &db)?;
//     println!("{:#?}", db);
//
//     Ok(())
// }
